use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    /// Vote to approve given proposal
    VoteApprove,
    /// Vote to reject given proposal
    VoteReject,
    /// Vote to nonsense given proposal(because it's spam).
    VoteNonsense,
    /// Vote to farm id given proposal
    VoteFarm { farm_id: usize },
    /// Vote to poll id given proposal
    VotePoll { poll_id: usize }
}

impl Action {

    pub fn get_index(&self) -> usize {
        match self {
            Action::VoteFarm { farm_id } => {
                *farm_id
            },
            Action::VotePoll { poll_id } => {
                *poll_id
            },
            Action::VoteApprove => Vote::Approve as usize,
            Action::VoteReject => Vote::Reject as usize,
            Action::VoteNonsense => Vote::Nonsense as usize,
        }
    }
    
}

/// Votes recorded in the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum Vote {
    Approve = 0x0,
    Reject = 0x1,
    Nonsense = 0x2,
}

impl From<&Action> for Vote {
    fn from(action: &Action) -> Self {
        match action {
            Action::VoteApprove => Vote::Approve,
            Action::VoteReject => Vote::Reject,
            Action::VoteNonsense => Vote::Nonsense,
            Action::VoteFarm { .. } => env::panic_str(E201_INVALID_VOTE),
            Action::VotePoll { .. } => env::panic_str(E201_INVALID_VOTE),
        }
    }
}

impl Proposal {
    pub fn update_votes(
        &mut self,
        action: &Action,
        amount: Balance,
        total: Balance,
        is_increased: bool
    ) {
        let index = action.get_index();
        match &self.kind {
            ProposalKind::FarmingReward { farm_list, .. } => {
                require!(index < farm_list.len(), E201_INVALID_VOTE);
                if is_increased {
                    self.votes[index].total_ballots += amount;
                } else {
                    self.votes[index].total_ballots -= amount;
                }
                self.votes[farm_list.len()].total_ballots = total;
            },
            ProposalKind::Poll { options } => {
                require!(index < options.len(), E201_INVALID_VOTE);
                if is_increased {
                    self.votes[index].total_ballots += amount;
                } else {
                    self.votes[index].total_ballots -= amount;
                }
                self.votes[options.len()].total_ballots = total;
            },
            ProposalKind::Common { .. } => {
                if is_increased {
                    self.votes[index].total_ballots += amount;
                } else {
                    self.votes[index].total_ballots -= amount;
                }
                self.votes[3].total_ballots = total;
            }
        }
    }
}

impl Contract {
    pub fn internal_append_vote(
        &mut self,
        proposal_id: u32,
        action: &Action,
        amount: Balance,
    ) {
        let mut proposal = self.internal_unwrap_proposal(proposal_id);
        require!(action != &Action::VoteNonsense, E201_INVALID_VOTE);
        
        // check proposal is inprogress
        match proposal.status {
            Some(ProposalStatus::InProgress) => {
                // update proposal result
                proposal.update_votes(
                    action,
                    amount,
                    self.data().cur_total_ve_lpt,
                    true
                );
                proposal.votes[action.get_index()].participants += 1;
                proposal.participants += 1;
                
                self.data_mut()
                    .proposals
                    .insert(&proposal_id, &proposal.into());
            },
            _ => env::panic_str(E205_NOT_VOTABLE)
        }
    }

    pub fn internal_cancel_vote(
        &mut self,
        proposal_id: u32,
        action: &Action,
        amount: Balance,
    ) {
        let mut proposal = self.internal_unwrap_proposal(proposal_id);
        
        // check proposal is inprogress
        match proposal.status {
            Some(ProposalStatus::InProgress) => {
                // update proposal result
                proposal.update_votes(
                    action,
                    amount,
                    self.data().cur_total_ve_lpt,
                    false
                );
                proposal.votes[action.get_index()].participants -= 1;
                proposal.participants -= 1;
                
                self.data_mut()
                    .proposals
                    .insert(&proposal_id, &proposal.into());
            },
            _ => env::panic_str(E204_VOTE_CAN_NOT_CANCEL)
        }
    }
}