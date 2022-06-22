use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn extend_whitelisted_accounts(&mut self, accounts: Vec<AccountId>) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        for account in accounts {
            self.data_mut().whitelisted_accounts.insert(&account);
        }
    }

    #[payable]
    pub fn remove_whitelisted_accounts(&mut self, accounts: Vec<AccountId>) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        for account in accounts {
            self.data_mut().whitelisted_accounts.remove(&account);
        }
    }

    #[payable]
    pub fn extend_whitelisted_incentive_tokens(&mut self, tokens: Vec<AccountId>) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        for token in tokens {
            self.data_mut().whitelisted_incentive_tokens.insert(token);
        }
    }

    #[payable]
    pub fn remove_whitelisted_incentive_tokens(&mut self, tokens: Vec<AccountId>) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        for token in tokens {
            self.data_mut().whitelisted_incentive_tokens.remove(&token);
        }
    }

    #[payable]
    pub fn modify_min_start_vote_offset_sec(&mut self, min_start_vote_offset_sec: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
        config.min_proposal_start_vote_offset_sec = min_start_vote_offset_sec;
        
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_locking_policy(&mut self, min_duration: DurationSec, max_duration: DurationSec, max_ratio: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
        config.min_locking_duration_sec = min_duration;
        config.max_locking_duration_sec = max_duration;
        config.max_locking_multiplier = max_ratio;
        
        config.assert_valid();
        self.data_mut().config.set(&config);
    }

    /// owner help to return those who lost lpt when withdraw,
    /// It's owner's responsibility to verify amount and token id before calling
    #[payable]
    pub fn return_lpt_lostfound(&mut self, account_id: AccountId, amount: U128) -> Promise {
        assert_one_yocto();
        self.assert_owner();

        // update inner state
        let max_amount = self.data().lostfound;
        require!(amount.0 <= max_amount, E101_INSUFFICIENT_BALANCE);
        self.data_mut().lostfound -= amount.0;

        self.transfer_lpt_lostfound(&account_id, amount.0)
    }

    #[payable]
    pub fn return_removed_proposal_assets(&mut self, account_id: AccountId, token_id: AccountId, amount: U128) -> Promise {
        assert_one_yocto();
        self.assert_owner();

        let max_amount = self.data().removed_proposal_assets.get(&token_id).unwrap_or(0_u128);
        require!(amount.0 <= max_amount, E101_INSUFFICIENT_BALANCE);
        self.data_mut().removed_proposal_assets.insert(&token_id, &(max_amount - amount.0));

        self.transfer_removed_proposal_assets(&token_id, &account_id, amount.0)
    }

    #[private]
    pub fn callback_withdraw_lpt_lostfound(&mut self, receiver_id: AccountId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // all seed amount go to lostfound
                self.data_mut().lostfound += amount;

                Event::LptWithdrawLostfound {
                    receiver_id: &receiver_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::LptWithdrawLostfound {
                    receiver_id: &receiver_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }

    #[private]
    pub fn callback_removed_proposal_assets(
        &mut self,
        token_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                Event::RemovedProposalAssets {
                    receiver_id: &receiver_id,
                    token_id: &token_id,
                    amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                let current_amount = self.data().removed_proposal_assets.get(&token_id).unwrap_or(0_u128);
                self.data_mut().removed_proposal_assets.insert(
                    &token_id,
                    &(amount + current_amount),
                );

                Event::RemovedProposalAssets {
                    receiver_id: &receiver_id,
                    token_id: &token_id,
                    amount: &U128(amount),
                    success: false,
                }
                .emit();
            }
        }
    }
}

impl Contract {
    fn transfer_lpt_lostfound(&mut self, account_id: &AccountId, amount: Balance) -> Promise {
        ext_multi_fungible_token::mft_transfer(
            self.data().lptoken_id.clone(),
            account_id.clone(),
            amount.into(),
            None,
            self.data().lptoken_contract_id.clone(),
            1, // one yocto near
            GAS_FOR_LPT_TRANSFER,
        )
        .then(ext_self::callback_withdraw_lpt_lostfound(
            account_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_LPT_TRANSFER,
        ))
    }

    fn transfer_removed_proposal_assets(&mut self, token_id: &AccountId, account_id: &AccountId, amount: Balance) -> Promise {
        ext_fungible_token::ft_transfer(
            account_id.clone(),
            amount.into(),
            None,
            token_id.clone(),
            1,
            GAS_FOR_REMOVED_PROPOSAL_ASSETS,
        )
        .then(ext_self::callback_removed_proposal_assets(
            token_id.clone(),
            account_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_REMOVED_PROPOSAL_ASSETS,
        ))
    }
}