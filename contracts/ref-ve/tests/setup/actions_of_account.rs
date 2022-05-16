use crate::*;
use near_sdk::json_types::U128;

impl Env {
    pub fn withdraw_lpt(
        &self,
        operator: &UserAccount,
        amount: Option<u128>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.withdraw_lpt(
                    if let Some(amount) = amount { Some(U128(amount)) } else { None },
                ),
                MAX_GAS.0,
                1,
            )
    }
}