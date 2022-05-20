use crate::*;

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn withdraw_lpt(&mut self, amount: Option<U128>)  -> Promise {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&account_id);
        let prev_ve_lpt_amount = account.ve_lpt_amount;
        let amount = if let Some(request) = amount {
            request.0
        } else {
            account.lpt_amount
        };
        let decreased_ve_lpt = account.withdraw_lpt(amount);
        self.burn_love_token(&account_id, decreased_ve_lpt);

        self.data_mut().cur_lock_lpt -= amount;
        self.data_mut().cur_total_ve_lpt -= decreased_ve_lpt;

        self.update_impacted_proposals(&mut account, prev_ve_lpt_amount, decreased_ve_lpt, false);
        
        self.internal_set_account(&account_id, account);

        self.transfer_lpt_token(&account_id, amount)
    }


    #[private]
    pub fn callback_withdraw_lpt(&mut self, sender_id: AccountId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // all token amount go to lostfound
                self.data_mut().lostfound += amount;

                Event::LptWithdraw {
                    proposer_id: &sender_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            }
            PromiseResult::Successful(_) => {
                Event::LptWithdraw {
                    proposer_id: &sender_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }
}

impl Contract {
    fn transfer_lpt_token(
        &self,
        account_id: &AccountId,
        amount: Balance,
    ) -> Promise {
        ext_multi_fungible_token::mft_transfer(
            self.data().lptoken_id.clone(),
            account_id.clone(),
            amount.into(),
            None,
            self.data().lptoken_contract_id.clone(),
            1, // one yocto near
            GAS_FOR_LPT_TRANSFER,
        )
        .then(ext_self::callback_withdraw_lpt(
            account_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_LPT_TRANSFER,
        ))
    }
}