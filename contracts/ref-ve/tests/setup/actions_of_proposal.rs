use crate::*;
use near_sdk::{AccountId, Balance};

impl Env {
    pub fn create_proposal(
        &self,
        operator: &UserAccount,
        kind: ProposalKind,
        start_at: u64,
        duration_sec: u32,
        incentive_detail: Option<(AccountId, IncentiveType)>,
        deposit: Balance
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.create_proposal(
                    kind, start_at, duration_sec, incentive_detail
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn remove_proposal(
        &self,
        operator: &UserAccount,
        proposal_id: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.remove_proposal(
                    proposal_id
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn redeem_near_in_expired_proposal(
        &self,
        operator: &UserAccount,
        proposal_id: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.redeem_near_in_expired_proposal(
                    proposal_id
                ),
                MAX_GAS.0,
                0,
            )
    }

    pub fn action_proposal(
        &self,
        operator: &UserAccount,
        proposal_id: u32, action: Action, memo: Option<String>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.action_proposal(
                    proposal_id, action, memo
                ),
                MAX_GAS.0,
                0,
            )
    }

    pub fn action_cancel(
        &self,
        operator: &UserAccount,
        proposal_id: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.action_cancel(
                    proposal_id
                ),
                MAX_GAS.0,
                1,
            )
    }
}