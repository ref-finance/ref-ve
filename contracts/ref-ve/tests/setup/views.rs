use crate::*;
use near_sdk::AccountId;
use std::collections::HashMap;
use near_sdk::json_types::U128;


impl Env {
    pub fn get_metadata(&self) -> Metadata{
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_metadata()
        ).unwrap_json::<Metadata>()
    }

    pub fn get_config(&self) -> Config{
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_config()
        ).unwrap_json::<Config>()
    }

    pub fn list_proposals(&self, from_index: Option<u64>, limit: Option<u64>,) -> Vec<Proposal>{
        self.owner
        .view_method_call(
            self.ve_contract.contract.list_proposals(from_index, limit)
        ).unwrap_json::<Vec<Proposal>>()
    }

    pub fn get_proposal(&self, proposal_id: u32) -> Option<Proposal>{
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_proposal(proposal_id)
        ).unwrap_json::<Option<Proposal>>()
    }

    pub fn get_account_info(&self, user: &UserAccount) -> Option<AccountInfo>{
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_account_info(user.account_id())
        ).unwrap_json::<Option<AccountInfo>>()
    }

    pub fn get_vote_detail(&self, user: &UserAccount) -> HashMap<u32, VoteDetail> {
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_vote_detail(user.account_id())
        ).unwrap_json::<HashMap<u32, VoteDetail>>()
    }

    pub fn get_vote_detail_history(&self, user: &UserAccount) -> HashMap<u32, VoteDetail> {
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_vote_detail_history(user.account_id())
        ).unwrap_json::<HashMap<u32, VoteDetail>>()
    }

    pub fn get_unclaimed_proposal(&self, user: &UserAccount) -> HashMap<u32, VoteDetail> {
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_unclaimed_proposal(user.account_id())
        ).unwrap_json::<HashMap<u32, VoteDetail>>()
    }

    pub fn get_unclaimed_rewards(&self, user: &UserAccount) -> HashMap<AccountId, U128> {
        self.owner
        .view_method_call(
            self.ve_contract.contract.get_unclaimed_rewards(user.account_id())
        ).unwrap_json::<HashMap<AccountId, U128>>()
    }
}