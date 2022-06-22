use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    FarmingReward { 
        farm_list: Vec<String>,
        total_reward: u32
    },
    Poll {
        options: Vec<String>,
    },
    Common,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    WarmUp,
    InProgress,
    /// Expired after period of time.
    Expired,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct FarmingReward {
    #[serde(with = "u128_dec_format")]
    pub price: u128,
    pub portion_list: Vec<(String, u32)>
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct VoteInfo {
    #[serde(with = "u128_dec_format")]
    pub total_ballots: u128,
    pub participants: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub id: u32,
    pub description: String,
    /// Original proposer.
    pub proposer: AccountId,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Result of proposal with relevant information.
    pub votes: Vec<VoteInfo>,
    #[serde(with = "u128_dec_format")]
    pub ve_amount_at_last_action: u128,
    /// the nano seconds of voting begin time,
    /// before this time, proposer can remove this immediately.
    #[serde(with = "u64_dec_format")]
    pub start_at: Timestamp,
    /// the nano seconds of voting end time,
    /// An inprogress poposal would change to expired after it.
    #[serde(with = "u64_dec_format")]
    pub end_at: Timestamp,
    #[serde(with = "u64_dec_format")]
    pub participants: u64,

    /// Incentive of proposal with relevant information.   
    pub incentive: HashMap<u32, ProposalIncentive>,
    #[borsh_skip]
    pub status: Option<ProposalStatus>,
    #[borsh_skip] 
    pub is_nonsense: Option<bool>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VProposal {
    Current(Proposal),
}

impl From<VProposal> for Proposal {
    fn from(v: VProposal) -> Self {
        match v {
            VProposal::Current(c) => c,
        }
    }
}

impl From<Proposal> for VProposal {
    fn from(c: Proposal) -> Self {
        VProposal::Current(c)
    }
}

impl Proposal {

    pub fn update_result(&mut self){
        if let ProposalKind::Common { .. } = &self.kind {
            if self.votes[0].total_ballots + self.votes[1].total_ballots < self.votes[2].total_ballots {
                self.is_nonsense = Some(true);
            } else {
                self.is_nonsense = Some(false);
            }
        }
    }

    pub fn update_status(&mut self) {
        let now = env::block_timestamp(); 
        if now < self.start_at {
            self.status = Some(ProposalStatus::WarmUp);
        } else if now >= self.start_at && now < self.end_at {
            self.status = Some(ProposalStatus::InProgress);
        } else {
            self.status = Some(ProposalStatus::Expired);
            self.update_result();
        }
    }
}


impl Contract {
    pub fn internal_unwrap_proposal(&self, proposal_id: u32) -> Proposal {
        let mut proposal = self.internal_get_proposal(proposal_id).expect(E404_PROPOSAL_NOT_EXIST);
        proposal.update_status();
        proposal
    }

    pub fn internal_get_proposal(&self, proposal_id: u32) -> Option<Proposal> {
        self.data().proposals.get(&proposal_id).map(|o| o.into())
    }

    pub fn internal_set_proposal(&mut self, proposal_id: u32, proposal: Proposal) {
        self.data_mut().proposals.insert(&proposal_id, &proposal.into());
    }
}