use crate::*;

impl Env {
    pub fn set_owner(
        &self, 
        operator: &UserAccount,
        new_owner: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.set_owner(
                    new_owner.account_id(),
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn extend_operators(
        &self, 
        operator: &UserAccount,
        operators: Vec<&UserAccount>,
        deposit: u128
    ) -> ExecutionResult {
        let ops = operators.iter().map(|v| v.account_id()).collect::<Vec<_>>();
        operator
        .function_call(
            self.ve_contract.contract.extend_operators(
                ops,
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn remove_operators(
        &self, 
        operator: &UserAccount,
        operators: Vec<&UserAccount>,
        deposit: u128
    ) -> ExecutionResult {
        let ops = operators.iter().map(|v| v.account_id()).collect::<Vec<_>>();
        operator
        .function_call(
            self.ve_contract.contract.remove_operators(
                ops,
            ),
            MAX_GAS.0,
            deposit,
        )
    }
}