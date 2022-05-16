use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn modify_voting_period_range(&mut self, min_voting_period: u32, max_voting_period: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
        config.min_proposal_voting_period = min_voting_period;
        config.max_proposal_voting_period = max_voting_period;
        
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_min_start_vote_offset(&mut self, min_start_vote_offset: Timestamp) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
        config.min_proposal_start_vote_offset = min_start_vote_offset;
        
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_lock_near_per_proposal(&mut self, amount: U128) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
        config.lock_near_per_proposal = amount.0;
        
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_min_per_lock_lpt_amount(&mut self, amount: U128) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
        config.min_per_lock_lpt_amount = amount.0;
        
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_locking_policy(&mut self, max_duration: DurationSec, max_ratio: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        
        let mut config =  self.data().config.get().unwrap();
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

    /// Owner retrieve those slashed lpt
    #[payable]
    pub fn withdraw_lpt_slashed(&mut self) -> Promise {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);

        // update inner state
        let amount = self.data().slashed;
        require!(amount > 0, E101_INSUFFICIENT_BALANCE);
        self.data_mut().slashed = 0;

        self.transfer_lpt_slashed(&env::predecessor_account_id(), amount)
    }

    #[private]
    pub fn callback_withdraw_lpt_lostfound(&mut self, sender_id: AccountId, amount: U128) {
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
                    proposer_id: &sender_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::LptWithdrawLostfound {
                    proposer_id: &sender_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }

    #[private]
    pub fn callback_withdraw_lpt_slashed(&mut self, sender_id: AccountId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // all seed amount go to lostfound
                self.data_mut().slashed += amount;

                Event::LptWithdrawSlashed {
                    owner_id: &sender_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::LptWithdrawSlashed {
                    owner_id: &sender_id,
                    withdraw_amount: &U128(amount),
                    success: true,
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

    fn transfer_lpt_slashed(&mut self, account_id: &AccountId, amount: Balance) -> Promise {
        ext_multi_fungible_token::mft_transfer(
            self.data().lptoken_id.clone(),
            account_id.clone(),
            amount.into(),
            None,
            self.data().lptoken_contract_id.clone(),
            1, // one yocto near
            GAS_FOR_LPT_TRANSFER,
        )
        .then(ext_self::callback_withdraw_lpt_slashed(
            account_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_LPT_TRANSFER,
        ))
    }
}