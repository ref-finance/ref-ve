use crate::*;
use near_sdk::serde_json::json;
use near_sdk::json_types::U128;
use near_sdk::Balance;

impl Env {
    pub fn deposit_reward(
        &self,
        token: &UserAccount,
        user: &UserAccount,
        amount: Balance,
        proposal_id: u32,
        incentive_key: u32,
        incentive_type: String,
    ) -> ExecutionResult {
        user.call(
            token.account_id.clone(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.ve_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": format!("{{\"Reward\": {{\"proposal_id\": {}, \"incentive_key\": {}, \"incentive_type\": \"{}\"}}}}", proposal_id, incentive_key, incentive_type),
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn lock_lpt(
        &self,
        user: &UserAccount,
        amount: Balance,
        duration_sec: u32,
    ) -> ExecutionResult {
        user.call(
            self.lptoken_contract.account_id(),
            "mft_transfer_call",
            &json!({
                "token_id": &lpt_id(),
                "receiver_id": self.ve_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": format!("{{\"Lock\": {{\"duration_sec\": {}}}}}", duration_sec),
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn append_lpt(
        &self,
        user: &UserAccount,
        amount: Balance,
        append_duration_sec: u32,
    ) -> ExecutionResult {
        user.call(
            self.lptoken_contract.account_id(),
            "mft_transfer_call",
            &json!({
                "token_id": &lpt_id(),
                "receiver_id": self.ve_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": format!("{{\"Append\": {{\"append_duration_sec\": {}}}}}", append_duration_sec),
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }
}