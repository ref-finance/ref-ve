use crate::*;
use near_sdk::Balance;

impl Env {
    pub fn create_proposal(
        &self,
        operator: &UserAccount,
        kind: ProposalKind,
        description: String,
        start_at: u32,
        duration_sec: u32,
        deposit: Balance
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.create_proposal(
                    kind, description, start_at, duration_sec
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
                1,
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