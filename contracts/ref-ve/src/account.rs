use crate::*;
use std::cmp::Ordering;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct VoteDetail {
    pub action: Action,
    #[serde(with = "u128_dec_format")]
    pub amount: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    pub sponsor_id: AccountId,
    /// The amount of LPT locked
    #[serde(with = "u128_dec_format")]
    pub lpt_amount: Balance,
    /// The amount of veLPT the account holds
    #[serde(with = "u128_dec_format")]
    pub ve_lpt_amount: Balance,
    /// When the locking token can be unlocked without slash in nanoseconds.
    #[serde(with = "u64_dec_format")]
    pub unlock_timestamp: u64,
    /// The duration of current locking in seconds.
    pub duration_sec: u32,
    /// Record voting info
    #[serde(skip_serializing)]
    pub proposals: HashMap<u32, VoteDetail>,
    /// Record expired proposal voting info
    #[serde(skip_serializing)]
    pub proposals_history: UnorderedMap<u32, VoteDetail>,
    #[serde(with = "u128_map_format")]
    pub rewards: HashMap<AccountId, Balance>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAccount {
    Current(Account),
}

impl From<VAccount> for Account {
    fn from(v: VAccount) -> Self {
        match v {
            VAccount::Current(c) => c,
        }
    }
}

impl From<Account> for VAccount {
    fn from(c: Account) -> Self {
        VAccount::Current(c)
    }
}

impl Account {
    pub fn new(account_id: &AccountId, sponsor_id: &AccountId) -> Self {
        Account {
            sponsor_id: sponsor_id.clone(),
            lpt_amount: 0,
            ve_lpt_amount: 0,
            unlock_timestamp: 0,
            duration_sec: 0,
            proposals: HashMap::new(),
            proposals_history: UnorderedMap::new(StorageKeys::AccountProposalHistory { account_id: account_id.clone() }),
            rewards: HashMap::new()
        }
    }

    pub fn add_rewards(&mut self, rewards: &HashMap<AccountId, Balance>) {
        for (reward_token, reward) in rewards {
            self.rewards.insert(
                reward_token.clone(),
                reward + self.rewards.get(reward_token).unwrap_or(&0_u128),
            );
        }
    }

    pub fn add_history(&mut self, history: &HashMap<u32, VoteDetail>){
        for (proposal_id, vote_detail) in history {
            self.proposals_history.insert(proposal_id, vote_detail);
        }
    }

    pub fn sub_reward(&mut self, token_id: &AccountId, amount: Balance) {
        if let Some(prev) = self.rewards.remove(token_id) {
            require!(amount <= prev, E101_INSUFFICIENT_BALANCE);
            let remain = prev - amount;
            if remain > 0 {
                self.rewards.insert(token_id.clone(), remain);
            }
        }
    }

    pub fn lock_lpt(&mut self, amount: Balance, duration_sec: u32, config: &Config, lptoken_decimals: u8) -> Balance {
        let prev = self.ve_lpt_amount;

        let timestamp = env::block_timestamp();
        let new_unlock_timestamp = timestamp + to_nano(duration_sec);

        if self.unlock_timestamp > 0 && self.unlock_timestamp > timestamp {
            // exist lpt locked need relock
            require!(nano_to_sec(self.unlock_timestamp) <= nano_to_sec(new_unlock_timestamp), E304_CAUSE_PRE_UNLOCK);
            let relocked_ve = compute_ve_lpt_amount(config, self.lpt_amount, duration_sec, lptoken_decimals);
            self.ve_lpt_amount = std::cmp::max(self.ve_lpt_amount, relocked_ve);
            let extra_x = compute_ve_lpt_amount(config, amount, duration_sec, lptoken_decimals);
            self.ve_lpt_amount += extra_x;
        } else {
            self.ve_lpt_amount = compute_ve_lpt_amount(config, self.lpt_amount + amount, duration_sec, lptoken_decimals);
        }
        self.unlock_timestamp = new_unlock_timestamp;
        self.lpt_amount += amount;
        self.duration_sec = duration_sec;

        self.ve_lpt_amount - prev
    }

    pub fn withdraw_lpt(&mut self, amount: u128) -> Balance {
        let prev = self.ve_lpt_amount;

        let timestamp = env::block_timestamp();
        require!(timestamp >= self.unlock_timestamp, E305_STILL_IN_LOCK);
        require!(amount <= self.lpt_amount && amount != 0, E101_INSUFFICIENT_BALANCE);

        if amount < self.lpt_amount {
            let new_ve = u128_ratio(self.ve_lpt_amount, self.lpt_amount - amount, self.lpt_amount);
            self.ve_lpt_amount = new_ve;
        } else {
            self.ve_lpt_amount = 0;
            self.unlock_timestamp = 0;
            self.duration_sec = 0;
        }
        self.lpt_amount -= amount;

        prev - self.ve_lpt_amount
    }
}

impl Contract {
    pub fn update_impacted_proposals(&mut self, account: &mut Account, diff_ve_lpt_amount: Balance, is_increased: bool){
        let mut rewards = HashMap::new();
        let mut history = HashMap::new();
        account.proposals.retain(|proposal_id, vote_detail| {
            let mut proposal = self.internal_unwrap_proposal(*proposal_id);
            if proposal.status == Some(ProposalStatus::Expired) {
                if let Some(reward_details) = proposal.claim_reward(vote_detail) {
                    reward_details.into_iter().for_each(|(reward_token, reward_amount)| {
                        rewards.insert(reward_token.clone(), reward_amount + rewards.get(&reward_token).unwrap_or(&0_u128));
                    });
                }
                self.internal_set_proposal(*proposal_id, proposal.into());
                history.insert(*proposal_id, vote_detail.clone());
                false
            } else {
                let mut is_retain = true;
                if diff_ve_lpt_amount > 0 {
                    proposal.update_votes(&vote_detail.action, diff_ve_lpt_amount, is_increased);
                    if is_increased {
                        vote_detail.amount += diff_ve_lpt_amount;
                    } else if vote_detail.amount == diff_ve_lpt_amount {
                        proposal.votes[vote_detail.action.get_index()].participants -= 1;
                        proposal.participants -= 1;
                        is_retain = false
                    } else {
                        vote_detail.amount -= diff_ve_lpt_amount;
                    }
                    proposal.ve_amount_at_last_action = self.data().cur_total_ve_lpt;
                    self.internal_set_proposal(*proposal_id, proposal.into());
                }
                is_retain
            }
        });
        account.add_rewards(&rewards);
        account.add_history(&history);
    }

    pub fn internal_account_vote(
        &mut self,
        voter: &AccountId,
        proposal_id: u32,
        action: &Action,
    ) -> Balance {
        let mut account = self.internal_unwrap_account(voter);
        let ve_lpt_amount = account.ve_lpt_amount;
        require!(ve_lpt_amount > 0, E303_INSUFFICIENT_VE_LPT);
        require!(!account.proposals.contains_key(&proposal_id), E200_ALREADY_VOTED);
        account.proposals.insert(proposal_id, VoteDetail{
            action: action.clone(),
            amount: ve_lpt_amount,
        });
        self.internal_claim_all(&mut account);
        self.internal_set_account(voter, account.into());
        ve_lpt_amount
    }

    pub fn internal_account_cancel_vote(
        &mut self,
        voter: &AccountId,
        proposal_id: u32,
    ) -> VoteDetail {
        let mut account = self.internal_unwrap_account(voter);
        require!(account.proposals.contains_key(&proposal_id), E206_NO_VOTED);
        let action = account.proposals.remove(&proposal_id).unwrap();
        self.internal_claim_all(&mut account);
        self.internal_set_account(voter, account.into());
        action
    }
}

impl Contract {
    pub fn internal_get_account(&self, account_id: &AccountId) -> Option<Account> {
        self.data().accounts.get(account_id).map(|o| o.into())
    }

    pub fn internal_unwrap_account(&self, account_id: &AccountId) -> Account {
        self.internal_get_account(account_id)
            .expect(E100_ACC_NOT_REGISTERED)
    }

    pub fn internal_set_account(&mut self, account_id: &AccountId, account: Account) {
        self.data_mut().accounts.insert(account_id, &account.into());
    }

    pub fn internal_unwrap_or_default_account(&mut self, account_id: &AccountId) -> Account {
        if let Some(account) = self.internal_get_account(account_id) {
            account
        } else {
            self.data_mut().account_count += 1;
            Account::new(account_id, &env::current_account_id())
        }
    }

    pub fn internal_remove_account(&mut self, account_id: &AccountId) {
        self.ft.accounts.remove(account_id);
        self.data_mut().accounts.remove(account_id);
        self.data_mut().account_count -= 1;
    }
}

fn compute_ve_lpt_amount(config: &Config, amount: u128, duration_sec: u32, lptoken_decimals: u8) -> u128 {
    let amount = match lptoken_decimals.cmp(&LOVE_DECIMAL) {
        Ordering::Greater => amount / 10u128.pow((lptoken_decimals - LOVE_DECIMAL) as u32),
        Ordering::Less => amount * 10u128.pow((LOVE_DECIMAL - lptoken_decimals) as u32),
        Ordering::Equal => amount,
    };
    amount
        + u128_ratio(
            amount,
            u128::from(config.max_locking_multiplier - MIN_LOCKING_REWARD_RATIO) * u128::from(to_nano(duration_sec)),
            u128::from(to_nano(config.max_locking_duration_sec)) * MIN_LOCKING_REWARD_RATIO as u128,
        )
}