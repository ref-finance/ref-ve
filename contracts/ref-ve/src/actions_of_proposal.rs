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
        require!(self.data().whitelisted_accounts.contains(&proposer) , E002_NOT_ALLOWED);
        
        self.internal_unwrap_account(&proposer);

        let config = self.internal_config();

        require!(start_at - env::block_timestamp() >= config.min_proposal_start_vote_offset, E402_INVALID_START_TIME);

        let votes = match &kind {
            ProposalKind::FarmingReward{ farm_list, .. } => {
                require!(incentive_detail.is_none() , E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
                vec![0; farm_list.len()]
            },
            ProposalKind::Poll{ descriptions } => {
                vec![0; descriptions.len()]
            },
            ProposalKind::Common{ .. } => {
                require!(incentive_detail.is_none() , E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
                vec![0; 4]
            }
        };

        let mut proposal = Proposal{
            proposer: proposer.clone(),
            kind: kind.clone(),
            votes,
            incentive: None,
            start_at,
            end_at: start_at + to_nano(duration_sec),
            participants: 0,
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
