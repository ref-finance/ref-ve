use crate::*;
use near_sdk::AccountId;

impl Env {
    pub fn extend_whitelisted_accounts(
        &self,
        operator: &UserAccount,
        accounts: Vec<AccountId>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.extend_whitelisted_accounts(
                    accounts
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn remove_whitelisted_accounts(
        &self,
        operator: &UserAccount,
        accounts: Vec<AccountId>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.remove_whitelisted_accounts(
                    accounts
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn extend_whitelisted_incentive_tokens(
        &self,
        operator: &UserAccount,
        tokens: Vec<AccountId>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.extend_whitelisted_incentive_tokens(
                    tokens
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn remove_whitelisted_incentive_tokens(
        &self,
        operator: &UserAccount,
        tokens: Vec<AccountId>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.remove_whitelisted_incentive_tokens(
                    tokens
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_min_start_vote_offset_sec(
        &self,
        operator: &UserAccount,
        min_start_vote_offset: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_min_start_vote_offset_sec(
                    min_start_vote_offset
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_locking_policy(
        &self,
        operator: &UserAccount,
        min_duration: u32, max_duration: u32, max_ratio: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_locking_policy(
                    min_duration, max_duration, max_ratio
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

    pub fn return_removed_proposal_assets(
        &self,
        operator: &UserAccount,
        account: &UserAccount, token: &UserAccount, amount: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.return_removed_proposal_assets(
                    account.account_id(), token.account_id(), amount.into()
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn modify_voting_duration_limit(
        &self,
        operator: &UserAccount,
        min_voting_duration_sec: u32, max_voting_duration_sec: u32
    ) -> ExecutionResult {
        operator
            .function_call(
                self.ve_contract.contract.modify_voting_duration_limit(
                    min_voting_duration_sec, max_voting_duration_sec
                ),
                MAX_GAS.0,
                1,
            )
    }
}