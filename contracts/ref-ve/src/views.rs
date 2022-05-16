use crate::*;
use near_sdk::json_types::U64;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Debug))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub dao_id: AccountId,
    pub operators: Vec<AccountId>,
    pub account_count: U64,
    pub proposal_count: U64,
    pub cur_total_ve_lpt: U128,
    pub cur_lock_lpt: U128,
    pub lostfound: U128,
    pub slashed: U128,
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
            dao_id: self.data().dao_id.clone(),
            operators: self.data().operators.to_vec(),
            account_count: self.data().account_count.into(),
            proposal_count: self.data().proposals.len().into(),
            cur_total_ve_lpt: self.data().cur_total_ve_lpt.into(),
            cur_lock_lpt: self.data().cur_lock_lpt.into(),
            lostfound: self.data().lostfound.into(),
            slashed: self.data().slashed.into(),
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
    ) -> Option<Account> {
        self.internal_get_account(&account_id)
    }
}