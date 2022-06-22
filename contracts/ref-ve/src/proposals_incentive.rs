use crate::*;
#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalIncentive{
    pub incentive_token_ids: Vec<AccountId>,
    #[serde(with = "u128_vec_format")]
    pub incentive_amounts: Vec<Balance>,
    #[serde(with = "u128_vec_format")]
    pub claimed_amounts: Vec<Balance>,
}

impl ProposalIncentive {
    pub fn calc_reward(&self, account_votes_amount: u128, votes_total_amount: Balance) -> Vec<(AccountId, Balance)> {
        let mut claimed_list = vec![];
        for (index, token_id) in self.incentive_token_ids.iter().enumerate() {
            let reward = u128_ratio(self.incentive_amounts[index], account_votes_amount, votes_total_amount);
            claimed_list.push((token_id.clone(), reward));
        }
        claimed_list
    }
}

impl Proposal {
    
    pub fn get_votes_total_amount_for_reward_calc(&self, incentive_key: u32) -> u128{
        match self.kind{
            ProposalKind::FarmingReward { .. } => {
                self.votes[incentive_key as usize].total_ballots
            },
            _ => {
                self.votes.iter().map(|item| item.total_ballots).sum()
            }
        }
    }

    pub fn claim_reward(&mut self, vote_detail: &VoteDetail) -> Option<Vec<(AccountId, Balance)>> {
        let incentive_key = if let ProposalKind::FarmingReward { .. } = self.kind {
            vote_detail.action.get_index() as u32
        } else {
            0
        };
        let votes_total_amount = self.get_votes_total_amount_for_reward_calc(incentive_key);
        if let Some(incentive) = self.incentive.get_mut(&incentive_key) {
            let res = incentive.calc_reward(vote_detail.amount, votes_total_amount);
            incentive.claimed_amounts = res.iter().zip(incentive.claimed_amounts.iter()).map(|(new, old)| new.1 + old).collect();
            Some(res)
        } else {
            None
        }
    }

    pub fn deposit_reward(&mut self, incentive_key: u32, token_id: &AccountId, amount: Balance) -> Balance {
        let proposal_incentive = self.incentive.entry(incentive_key).or_insert(ProposalIncentive{
            incentive_token_ids: vec![token_id.clone()],
            incentive_amounts: vec![0u128],
            claimed_amounts: vec![0u128],
        });
        let index = match proposal_incentive.incentive_token_ids.iter().position(|incentive_token_id| incentive_token_id == token_id){
            Some(index) => index,
            None => {
                proposal_incentive.incentive_token_ids.push(token_id.clone());
                proposal_incentive.incentive_amounts.push(0);
                proposal_incentive.claimed_amounts.push(0);
                proposal_incentive.incentive_token_ids.len() - 1
            }
        };
        proposal_incentive.incentive_amounts[index] += amount;
        proposal_incentive.incentive_amounts[index]
    }
}

impl Contract {
    pub fn internal_deposit_reward(&mut self, proposal_id: u32, incentive_key: u32, token_id: &AccountId, amount: Balance) -> (Balance, Timestamp) {
        let mut proposal = self.internal_unwrap_proposal(proposal_id);
        match proposal.status {
            Some(ProposalStatus::WarmUp) | Some(ProposalStatus::InProgress) => {
                match &proposal.kind {
                    ProposalKind::Poll { .. } => {
                        require!(incentive_key == 0, E207_INVALID_INCENTIVE_KEY);
                        require!(self.data().whitelisted_incentive_tokens.contains(token_id), E203_INVALID_INCENTIVE_TOKEN);
                    },
                    ProposalKind::FarmingReward { farm_list, .. } => {
                        require!(incentive_key < farm_list.len() as u32, E207_INVALID_INCENTIVE_KEY);
                        let farm_tokens: Vec<AccountId> = farm_list[incentive_key as usize].split('|').into_iter().map(|a| a.parse().unwrap()).collect();
                        require!(
                            self.data().whitelisted_incentive_tokens.contains(token_id) || farm_tokens.contains(token_id)
                            , E203_INVALID_INCENTIVE_TOKEN);
                    },
                    _ => {
                        env::panic_str(E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
                    }
                }
                
                let total_reward = proposal.deposit_reward(incentive_key, token_id, amount);
                let start_at = proposal.start_at;
                self.data_mut().proposals.insert(&proposal_id, &proposal.into());
                (total_reward, start_at)
            },
            _ => {
                env::panic_str(E406_EXPIRED_PROPOSAL);
            }
        }
    }
}