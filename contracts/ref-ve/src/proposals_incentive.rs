use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum IncentiveType{
    Evenly,
    Proportion
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalIncentive{
    pub incentive_type: IncentiveType,
    pub incentive_token_id: AccountId,
    #[serde(with = "u128_dec_format")]
    pub incentive_amount: Balance,
    #[serde(with = "u128_dec_format")]
    pub claimed_amount: Balance,
}

impl ProposalIncentive {
    pub fn calc_reward(&self, participants: u64, account_votes_amount: u128, votes_total_amount: Balance) -> (AccountId, Balance) {
        let reward = match self.incentive_type {
            IncentiveType::Evenly => {
                self.incentive_amount / participants as u128
            },
            IncentiveType::Proportion => {
                u128_ratio(self.incentive_amount, account_votes_amount, votes_total_amount)
            }
        };
        (self.incentive_token_id.clone(), reward)
    }
}

impl Proposal {
    pub fn get_votes_total_amount(&self) -> u128{
        match self.kind{
            ProposalKind::FarmingReward { .. } => {
                self.votes.iter().sum()
            },
            ProposalKind::Poll { .. } => {
                self.votes.iter().sum()
            },
            ProposalKind::Common { .. } => {
                self.votes.iter().take(3).sum()
            }
        }
    }

    pub fn claim_reward(&mut self, account_votes_amount: Balance) -> Option<(AccountId, Balance)> {
        let votes_total_amount = self.get_votes_total_amount();
        if let Some(incentive) = &mut self.incentive {
            let (account_id, claimed_amount) = incentive.calc_reward(self.participants, account_votes_amount, votes_total_amount);
            incentive.claimed_amount += claimed_amount;
            Some((account_id, claimed_amount))
        } else {
            None
        }
    }

    pub fn deposit_reward(&mut self, token_id: &AccountId, amount: Balance) -> Balance {
        if let Some(incentive) = &mut self.incentive {
            require!(&incentive.incentive_token_id == token_id, E203_INVALID_INCENTIVE_TOKEN);
            incentive.incentive_amount += amount;
            incentive.incentive_amount
        } else {
            env::panic_str(E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
        }
    }
}

impl Contract {
    pub fn internal_deposit_reward(&mut self, proposal_id: u32, incentive_type: IncentiveType, token_id: &AccountId, amount: Balance) -> (Balance, Timestamp) {
        let mut proposal = self.internal_unwrap_proposal(proposal_id);
        match proposal.status {
            Some(ProposalStatus::WarmUp) | Some(ProposalStatus::InProgress) => {
                match proposal.kind {
                    ProposalKind::Poll { .. } => {
                        if proposal.incentive.is_none() {
                            proposal.incentive = Some(ProposalIncentive{
                                incentive_type,
                                incentive_token_id: token_id.clone(),
                                incentive_amount: 0u128,
                                claimed_amount: 0u128,
                            })
                        }
                        let total_reward = proposal.deposit_reward(token_id, amount);
                        let start_at = proposal.start_at;
                        self.data_mut().proposals.insert(&proposal_id, &proposal.into());
                        (total_reward, start_at)
                    },
                    _ => {
                        env::panic_str(E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
                    }
                }
            },
            _ => {
                env::panic_str(E406_EXPIRED_PROPOSAL);
            }
        }
    }
}