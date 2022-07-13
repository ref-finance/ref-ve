use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_proposal(
        &mut self,
        kind: ProposalKind,
        description: String,
        start_at: u32,
        duration_sec: u32,
    ) -> u32 {
        assert_one_yocto();

        let proposer = env::predecessor_account_id();
        require!(self.data().whitelisted_accounts.contains(&proposer) , E002_NOT_ALLOWED);
        require!(description.len() <= DESCRIPTION_LIMIT , E208_DESCRIPTION_TOO_LONG);
        
        self.internal_unwrap_account(&proposer);

        let config = self.internal_config();

        require!(duration_sec >= config.min_voting_duration_sec && duration_sec <= config.max_voting_duration_sec
            , E302_INVALID_DURATION);
        require!(start_at - nano_to_sec(env::block_timestamp()) >= config.min_proposal_start_vote_offset_sec, E402_INVALID_START_TIME);

        let votes: Vec<VoteInfo> = match &kind {
            ProposalKind::FarmingReward{ farm_list, .. } => {
                vec![Default::default(); farm_list.len()]
            },
            ProposalKind::Poll{ options, .. } => {
                vec![Default::default(); options.len()]
            },
            ProposalKind::Common{ .. } => {
                vec![Default::default(); 3]
            }
        };

        let id = self.data().last_proposal_id;
        let proposal = Proposal{
            id,
            description,
            proposer: proposer.clone(),
            kind: kind.clone(),
            votes,
            ve_amount_at_last_action: self.data().cur_total_ve_lpt,
            incentive: HashMap::new(),
            start_at: to_nano(start_at),
            end_at: to_nano(start_at + duration_sec),
            participants: 0,
            status: None,
            is_nonsense: None
        };
        self.internal_set_proposal(id, proposal.into());

        Event::ProposalCreate {
            proposer_id: &proposer,
            proposal_id: id,
            kind: &format!("{:?}", kind),
            start_at: to_nano(start_at),
            duration_sec
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

                for item in proposal.incentive.values() {
                    for index in 0..item.incentive_token_ids.len() {
                        let current_amount = self.data().removed_proposal_assets.get(&item.incentive_token_ids[index]).unwrap_or(0_u128);
                        self.data_mut().removed_proposal_assets.insert(
                            &item.incentive_token_ids[index],
                            &(item.incentive_amounts[index] + current_amount),
                        );
                    }
                }

                Event::ProposalRemove {
                    proposer_id: &proposer,
                    proposal_id,
                }
                .emit();

                true
            }
            _ => false,
        }
    }

    #[payable]
    pub fn action_proposal(&mut self, proposal_id: u32, action: Action, memo: Option<String>) -> U128 {
        assert_one_yocto();

        let voter = env::predecessor_account_id();

        let ve_lpt_amount = self.internal_account_vote(&voter, proposal_id, &action);

        self.internal_append_vote(proposal_id, &action, ve_lpt_amount);

        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }

        Event::ActionProposal {
            voter_id: &voter,
            proposal_id,
            action: &format!("{:?}", action)
        }
        .emit();

        ve_lpt_amount.into()
    }

    #[payable]
    pub fn action_cancel(&mut self, proposal_id: u32) -> U128 {
        assert_one_yocto();
        let voter = env::predecessor_account_id();

        let vote_detail = self.internal_account_cancel_vote(&voter, proposal_id);

        self.internal_cancel_vote(proposal_id, &vote_detail);

        Event::ActionCancel {
            voter_id: &voter,
            proposal_id,
            action: &format!("{:?}", vote_detail.action)
        }
        .emit();

        vote_detail.amount.into()
    }
}
