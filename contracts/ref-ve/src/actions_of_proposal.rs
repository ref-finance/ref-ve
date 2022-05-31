use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_proposal(
        &mut self,
        kind: ProposalKind,
        start_at: Timestamp,
        duration_sec: u32,
        incentive_detail: Option<(AccountId, IncentiveType)>
    ) -> u32 {
        let proposer = env::predecessor_account_id();
        
        let is_dao = proposer == self.data().dao_id;

        if !is_dao {
            self.internal_unwrap_account(&proposer);
        }

        let config = self.internal_config();
        let lock_amount = check_proposal_attached(&proposer, &config, is_dao);

        require!(start_at - env::block_timestamp() >= config.min_proposal_start_vote_offset, E402_INVALID_START_TIME);

        let votes = match &kind {
            ProposalKind::FarmingReward{ farm_list, .. } => {
                require!(is_dao , E002_NOT_ALLOWED);
                require!(incentive_detail.is_none() , E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
                vec![0; farm_list.len()]
            },
            ProposalKind::Poll{ descriptions } => {
                require!(is_dao , E002_NOT_ALLOWED);
                vec![0; descriptions.len()]
            },
            ProposalKind::Common{ .. } => {
                require!(incentive_detail.is_none() , E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
                if !is_dao {
                    require!(duration_sec >= config.min_proposal_voting_period &&
                        duration_sec <= config.max_proposal_voting_period, E403_INVALID_VOTING_PERIOD);
                }
                vec![0; 4]
            }
        };

        let mut proposal = Proposal{
            proposer: proposer.clone(),
            lock_amount,
            kind: kind.clone(),
            votes,
            incentive: None,
            start_at,
            end_at: start_at + to_nano(duration_sec),
            participants: 0,
            farming_reward: None,
            status: None,
            is_nonsense: None
        };

        if let Some((incentive_token_id, incentive_type)) = incentive_detail.clone() {
            proposal.incentive = Some(ProposalIncentive{
                incentive_type,
                incentive_token_id,
                incentive_amount: 0u128,
                claimed_amount: 0u128
            });
        }

        let id = self.data().last_proposal_id;
        self.data_mut().proposals.insert(&id, &proposal.into());

        Event::ProposalCreate {
            proposer_id: &proposer,
            proposal_id: id,
            lock_near: &U128(lock_amount),
            kind: &format!("{:?}", kind),
            start_at,
            duration_sec,
            incentive_detail: &format!("{:?}", incentive_detail),
        }
        .emit();
        
        self.data_mut().last_proposal_id += 1;
        id
    }

    #[payable]
    pub fn remove_proposal(&mut self, proposal_id: u32) -> bool {
        assert_one_yocto();
       
        let proposer = env::predecessor_account_id();

        let proposal = self.internal_unwrap_proposal(proposal_id);
        require!(proposal.proposer == proposer, E002_NOT_ALLOWED);

        match proposal.status.unwrap() {
            ProposalStatus::WarmUp => {
                if proposal.lock_amount > 0 {
                    Promise::new(proposer.clone()).transfer(proposal.lock_amount);
                }
                self.data_mut().proposals.remove(&proposal_id);

                Event::ProposalRemote {
                    proposer_id: &proposer,
                    proposal_id,
                }
                .emit();

                true
            }
            _ => false,
        }
    }

    pub fn redeem_near_in_expired_proposal(&mut self, proposal_id: u32) -> bool {
        let proposer = env::predecessor_account_id();

        let mut proposal = self.internal_unwrap_proposal(proposal_id);
        require!(proposal.proposer == proposer, E002_NOT_ALLOWED);

        if proposal.status == Some(ProposalStatus::Expired) {
            self.internal_redeem_near(&mut proposal);
            self.data_mut().proposals.insert(&proposal_id, &proposal.into());
            true
        } else {
            false
        }
    }

    pub fn action_proposal(&mut self, proposal_id: u32, action: Action, memo: Option<String>) -> U128 {

        let proposer = env::predecessor_account_id();

        let ve_lpt_amount = self.internal_account_vote(&proposer, proposal_id, &action);

        self.internal_append_vote(proposal_id, &action, ve_lpt_amount);

        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }

        Event::ActionProposal {
            proposer_id: &proposer,
            proposal_id,
            action: &format!("{:?}", action)
        }
        .emit();

        ve_lpt_amount.into()
    }

    #[payable]
    pub fn action_cancel(&mut self, proposal_id: u32) -> U128 {
        assert_one_yocto();
        let proposer = env::predecessor_account_id();

        let vote_detail = self.internal_account_cancel_vote(&proposer, proposal_id);

        self.internal_cancel_vote(proposal_id, &vote_detail.action, vote_detail.amount);

        Event::ActionCancel {
            proposer_id: &proposer,
            proposal_id,
            action: &format!("{:?}", vote_detail.action)
        }
        .emit();

        vote_detail.amount.into()
    }
}

pub fn check_proposal_attached(proposer:&AccountId, config: &Config, is_dao: bool) -> Balance{
    let mut deposit_amount = env::attached_deposit();
    if is_dao {
        if deposit_amount > 0 {
            Promise::new(proposer.clone()).transfer(deposit_amount);
        }
        deposit_amount = 0;
    } else {
        require!(deposit_amount >= config.lock_near_per_proposal, E401_NOT_ENOUGH_LOCK_NEAR);
        let refund = deposit_amount - config.lock_near_per_proposal;
        if refund > 0 {
            Promise::new(proposer.clone()).transfer(refund);
        }
        deposit_amount = config.lock_near_per_proposal;
    }
    deposit_amount
}
