use std::iter::FromIterator;

use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

#[near_bindgen]
impl Contract {

    pub fn claim_and_withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&account_id);
        self.internal_claim_all(&mut account);
        account.rewards.retain(|token_id, amount|{
            self.transfer_reward(token_id, &account_id, *amount);
            false
        });
        self.internal_set_account(&account_id, account);
    }

    pub fn claim_reward(&mut self, proposal_id: u32) {
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&account_id);
        let mut proposal = self.internal_unwrap_proposal(proposal_id);
        if proposal.status == Some(ProposalStatus::Expired) {
            if let Some(vote_detail) = account.proposals.remove(&proposal_id) {
                if let Some(reward_details) = proposal.claim_reward(&vote_detail) {
                    account.add_rewards(&HashMap::from_iter(reward_details));
                }
                self.internal_set_proposal(proposal_id, proposal.into());
                account.proposals_history.insert(&proposal_id, &vote_detail);
                self.internal_set_account(&account_id, account);
            }
        }
    }

    /// Withdraws given reward token of given user.
    /// when amount is None, withdraw all balance of the token.
    pub fn withdraw_reward(&mut self, token_id: AccountId, amount: Option<U128>) {
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&account_id);

        let total = account.rewards.get(&token_id).unwrap_or(&0_u128);
        let amount: u128 = amount.map(|v| v.into()).unwrap_or(*total);

        if amount > 0 {
            // Note: subtraction, will be reverted if the promise fails.
            account.sub_reward(&token_id, amount);
            self.internal_set_account(&account_id, account);
            self.transfer_reward(&token_id, &account_id, amount);
        }
    }

    #[private]
    pub fn callback_post_withdraw_reward(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                Event::RewardWithdraw {
                    caller_id: &sender_id,
                    token_id: &token_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                if let Some(mut account) = self.internal_get_account(&sender_id) {
                    account.add_rewards(&HashMap::from([(token_id.clone(), amount)]));
                    self.internal_set_account(&sender_id, account);

                    Event::RewardWithdraw {
                        caller_id: &sender_id,
                        token_id: &token_id,
                        withdraw_amount: &U128(amount),
                        success: false,
                    }
                    .emit();
                } else {
                    Event::RewardLostfound {
                        caller_id: &sender_id,
                        token_id: &token_id,
                        withdraw_amount: &U128(amount),
                    }
                    .emit();
                }
            }
        }
    }
}

impl Contract {

    fn transfer_reward(&self, token_id: &AccountId, account_id: &AccountId, amount: Balance){
        ext_fungible_token::ft_transfer(
            account_id.clone(),
            amount.into(),
            None,
            token_id.clone(),
            1,
            GAS_FOR_REWARD_TRANSFER,
        )
        .then(ext_self::callback_post_withdraw_reward(
            token_id.clone(),
            account_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_REWARD_TRANSFER,
        ));
    }

    pub fn internal_claim_all(&mut self, account: &mut Account) {
        let mut rewards = HashMap::new();
        let mut history = HashMap::new();
        account.proposals.retain(|proposal_id, vote_detail| {
            let mut proposal = self.internal_unwrap_proposal(*proposal_id);
            if proposal.status == Some(ProposalStatus::Expired) {
                if let Some(reward_details) = proposal.claim_reward(vote_detail){
                    reward_details.into_iter().for_each(|(reward_token, reward_amount)| {
                        rewards.insert(reward_token.clone(), reward_amount + rewards.get(&reward_token).unwrap_or(&0_u128));
                    });
                }
                history.insert(*proposal_id, vote_detail.clone());
                self.internal_set_proposal(*proposal_id, proposal.into());
                false
            } else {
                true
            }
        });
        account.add_rewards(&rewards);
        account.add_history(&history);
    }


    pub fn internal_calc_account_unclaim_rewards(&self, account_id: &AccountId) -> HashMap<AccountId, Balance> {
        let account = self.internal_unwrap_account(account_id);
        let mut rewards = HashMap::new();
        for (proposal_id, vote_detail) in account.proposals {
            let proposal = self.internal_unwrap_proposal(proposal_id);
            if proposal.status == Some(ProposalStatus::Expired) {
                let incentive_key = if let ProposalKind::FarmingReward { .. } = proposal.kind {
                    vote_detail.action.get_index() as u32
                } else {
                    0
                };
                if let Some(incentive) = proposal.incentive.get(&incentive_key) {
                    let votes_total_amount = proposal.get_votes_total_amount_for_reward_calc(incentive_key);
                    let reward_details = incentive.calc_reward(vote_detail.amount, votes_total_amount);
                    reward_details.into_iter().for_each(|(reward_token, reward_amount)| {
                        rewards.insert(reward_token.clone(), reward_amount + rewards.get(&reward_token).unwrap_or(&0_u128));
                    });
                }
            }
        }
        rewards
    }
}