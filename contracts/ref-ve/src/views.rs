use crate::*;
use near_sdk::json_types::U64;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Debug))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub operators: Vec<AccountId>,
    pub whitelisted_accounts: Vec<AccountId>,
    pub whitelisted_incentive_tokens: Vec<AccountId>,
    pub lptoken_contract_id: AccountId,
    pub lptoken_id: String,
    pub lptoken_decimals: u8,
    pub account_count: U64,
    pub proposal_count: U64,
    pub cur_total_ve_lpt: U128,
    pub cur_lock_lpt: U128,
    pub lostfound: U128,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Debug))]
pub struct AccountInfo {
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
    #[serde(with = "u128_map_format")]
    pub rewards: HashMap<AccountId, Balance>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Deserialize, Clone))]
pub struct StorageReport {
    pub storage: U64,
    pub locking_near: U128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.data().owner_id.clone(),
            operators: self.data().operators.to_vec(),
            whitelisted_accounts: self.data().whitelisted_accounts.to_vec(),
            whitelisted_incentive_tokens: self.data().whitelisted_incentive_tokens.iter().cloned().collect(),
            lptoken_contract_id: self.data().lptoken_contract_id.clone(),
            lptoken_id: self.data().lptoken_id.clone(),
            lptoken_decimals: self.data().lptoken_decimals,
            account_count: self.data().account_count.into(),
            proposal_count: self.data().proposals.len().into(),
            cur_total_ve_lpt: self.data().cur_total_ve_lpt.into(),
            cur_lock_lpt: self.data().cur_lock_lpt.into(),
            lostfound: self.data().lostfound.into(),
        }
    }

    pub fn get_config(&self) -> Config {
        self.internal_config()
    }

    pub fn get_contract_storage_report(&self) -> StorageReport {
        let su = env::storage_usage();
        StorageReport {
            storage: U64(su),
            locking_near: U128(su as Balance * env::storage_byte_cost()),
        }
    }

    pub fn list_proposals(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<Proposal> {
        let values = self.data().proposals.values_as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(values.len());
        (from_index..std::cmp::min(values.len(), from_index + limit))
            .map(|index| {
                let mut proposal: Proposal = values.get(index).unwrap().into();
                proposal.update_status();
                proposal
            })
            .collect()
    }

    pub fn get_proposal(&self, proposal_id: u32) -> Option<Proposal> {
        if let Some(mut proposal) = self.internal_get_proposal(proposal_id) {
            proposal.update_status();
            Some(proposal)
        } else {
            None
        }
    }

    pub fn get_unclaimed_rewards(
        &self,
        account_id: AccountId,
    ) -> HashMap<AccountId, U128> {
        let rewards = self.internal_calc_account_unclaim_rewards(&account_id);
        rewards
            .into_iter()
            .map(|(key, val)| (key, val.into()))
            .collect()
    }

    pub fn get_account_info(
        &self,
        account_id: AccountId
    ) -> Option<AccountInfo> {
        if let Some(account) = self.internal_get_account(&account_id) {
            Some(AccountInfo {
                sponsor_id: account.sponsor_id,
                lpt_amount: account.lpt_amount,
                ve_lpt_amount: account.ve_lpt_amount,
                unlock_timestamp: account.unlock_timestamp,
                duration_sec: account.duration_sec,
                rewards: account.rewards,
            })
        } else {
            None
        }
    }

    pub fn get_vote_detail(
        &self,
        account_id: AccountId
    ) -> HashMap<u32, VoteDetail> {
        if let Some(account) = self.internal_get_account(&account_id) {
            let mut result = HashMap::new();
            for (proposal_id, vote_detail) in account.proposals {
                let proposal = self.internal_unwrap_proposal(proposal_id);
                if proposal.status != Some(ProposalStatus::Expired) {
                    result.insert(proposal_id, vote_detail.clone());
                }
            }
            result
        } else {
            HashMap::new()
        }
    }

    pub fn get_vote_detail_history(
        &self,
        account_id: AccountId
    ) -> HashMap<u32, VoteDetail> {
        if let Some(account) = self.internal_get_account(&account_id) {
            let mut result = HashMap::new();
            for (proposal_id, vote_detail) in account.proposals {
                let proposal = self.internal_unwrap_proposal(proposal_id);
                if proposal.status == Some(ProposalStatus::Expired) {
                    result.insert(proposal_id, vote_detail.clone());
                }
            }
            for (k, v) in account.proposals_history.iter() {
                result.insert(k, v);
            }
            result
        } else {
            HashMap::new()
        }
    }

    pub fn get_unclaimed_proposal(
        &self,
        account_id: AccountId
    ) -> HashMap<u32, VoteDetail> {
        if let Some(account) = self.internal_get_account(&account_id) {
            let mut result = HashMap::new();
            for (proposal_id, vote_detail) in account.proposals {
                let proposal = self.internal_unwrap_proposal(proposal_id);
                if proposal.status == Some(ProposalStatus::Expired) {
                    match proposal.kind {
                        ProposalKind::Poll { .. } | ProposalKind::FarmingReward { .. } => {
                            if !proposal.incentive.is_empty() {
                                result.insert(proposal_id, vote_detail.clone());
                            }
                        },
                        _ => {}
                    }
                }
            }
            result
        } else {
            HashMap::new()
        }
    }

    pub fn list_removed_proposal_assets(&self, from_index: Option<u64>, limit: Option<u64>) -> HashMap<AccountId, U128> {
        let keys = self.data().removed_proposal_assets.keys_as_vector();

        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(keys.len());

        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                (
                    keys.get(index).unwrap(),
                    self.data().removed_proposal_assets.get(&keys.get(index).unwrap()).unwrap().into()
                )
            })
            .collect()
    }
}