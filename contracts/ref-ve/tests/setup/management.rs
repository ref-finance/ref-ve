use crate::*;

impl Env {
    pub fn modify_voting_period_range(
        &self,
        operator: &UserAccount,
        min_voting_period: u32, max_voting_period: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_voting_period_range(
                    min_voting_period,
                    max_voting_period
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_min_start_vote_offset(
        &self,
        operator: &UserAccount,
        min_start_vote_offset: u64
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_min_start_vote_offset(
                    min_start_vote_offset
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_lock_near_per_proposal(
        &self,
        operator: &UserAccount,
        amount: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_lock_near_per_proposal(
                    amount.into()
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_min_per_lock_lpt_amount(
        &self,
        operator: &UserAccount,
        amount: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_min_per_lock_lpt_amount(
                    amount.into()
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_locking_policy(
        &self,
        operator: &UserAccount,
        max_duration: u32, max_ratio: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_locking_policy(
                    max_duration, max_ratio
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn return_lpt_lostfound(
        &self,
        operator: &UserAccount,
        account: &UserAccount, amount: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.return_lpt_lostfound(
                    account.account_id(), amount.into()
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn withdraw_lpt_slashed(
        &self,
        operator: &UserAccount,
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.withdraw_lpt_slashed(),
                MAX_GAS.0,
                1,
            )
    }
}