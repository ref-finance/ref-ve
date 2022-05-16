use crate::*;
impl Env {
    pub fn storage_deposit (
        &self,
        operator: &UserAccount,
        user: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.storage_deposit(
                    Some(user.account_id()),
                    None,
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn storage_withdraw(
        &self,
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.storage_withdraw(
                    None,
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn storage_unregister(
        &self,
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.storage_unregister(
                    None,
                ),
                MAX_GAS.0,
                deposit,
            )
    }
}