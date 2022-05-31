use crate::*;
use near_sdk::json_types::U128;

impl Env {
    pub fn claim_and_withdraw_all(
        &self,
        operator: &UserAccount
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.claim_and_withdraw_all(),
                MAX_GAS.0,
                0,
            )
    }

    pub fn claim_reward(
        &self,
        operator: &UserAccount, 
        proposal_id: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.claim_reward(
                    proposal_id
                ),
                MAX_GAS.0,
                0,
            )
    }

    pub fn withdraw_reward(
        &self,
        operator: &UserAccount, 
        token_id: &UserAccount, amount: Option<u128>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.withdraw_reward(
                    token_id.account_id(), 
                    if let Some(amount) = amount { Some(U128(amount)) } else { None }
                ),
                MAX_GAS.0,
                0,
            )
    }
}