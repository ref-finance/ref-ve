use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{serde_json, PromiseOrValue};

/// Message parameters to receive via ft function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum FTokenReceiverMessage {
    Reward { proposal_id: u32, incentive_type: IncentiveType }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount: u128 = amount.into();
        let token_id = env::predecessor_account_id();
        let message =
            serde_json::from_str::<FTokenReceiverMessage>(&msg).expect(E500_INVALID_MSG);
        match message {
            FTokenReceiverMessage::Reward { proposal_id, incentive_type } => {

                let (total_amount, start_at) = self.internal_deposit_reward(proposal_id, incentive_type, &token_id, amount);

                Event::RewardDeposit {
                    caller_id: &sender_id,
                    proposal_id,
                    token_id: &token_id,
                    deposit_amount: &U128(amount),
                    total_amount: &U128(total_amount),
                    start_at,
                }
                .emit();
            }
        }
        PromiseOrValue::Value(U128(0))
    }
}

pub trait MFTTokenReceiver {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

/// Message parameters to receive via mft function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum MFTokenReceiverMessage {
    Lock { duration_sec: u32 },
    Append { append_duration_sec: u32 }
}

#[near_bindgen]
impl MFTTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let (amount, refund) = self.real_amount_and_refund(amount.into());
        require!(amount > 0, E101_INSUFFICIENT_BALANCE);

        require!(token_id == self.data().lptoken_id, E600_MFT_INVALID_LPTOKEN_ID);
        require!(env::predecessor_account_id() == self.data().lptoken_contract_id, E601_MFT_INVALID_LPTOKEN_CONTRACT);
        
        let message =
            serde_json::from_str::<MFTokenReceiverMessage>(&msg).expect(E500_INVALID_MSG);
        match message {
            MFTokenReceiverMessage::Lock { duration_sec } => {
                self.lock_lpt(&sender_id, amount, duration_sec);
            },
            MFTokenReceiverMessage::Append { append_duration_sec } => {
                self.append_lpt(&sender_id, amount, append_duration_sec);
            },
        }
        PromiseOrValue::Value(U128(refund))
    }
}

impl Contract {

    pub fn lock_lpt(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        duration_sec: u32,
    ) {
        let mut account = self.internal_unwrap_or_default_account(account_id);
        let config = self.internal_config();
        require!(duration_sec >= config.min_locking_duration_sec, E302_INVALID_DURATION);
        require!(duration_sec <= config.max_locking_duration_sec, E302_INVALID_DURATION);

        let increased_ve_lpt = account.lock_lpt(amount, duration_sec, &config, self.data().lptoken_decimals);
        require!(increased_ve_lpt > 0, E101_INSUFFICIENT_BALANCE);
        self.mint_love_token(account_id, increased_ve_lpt);

        self.data_mut().cur_lock_lpt += amount;
        self.data_mut().cur_total_ve_lpt += increased_ve_lpt;

        self.update_impacted_proposals(&mut account, increased_ve_lpt, true);

        self.internal_set_account(account_id, account);

        Event::LptLock {
            caller_id: account_id,
            deposit_amount: &U128(amount),
            increased_ve_lpt: &U128(increased_ve_lpt),
            duration: duration_sec,
        }
        .emit();
    }

    pub fn append_lpt(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        append_duration_sec: u32,
    ) {
        let mut account = self.internal_unwrap_or_default_account(account_id);
        require!(account.unlock_timestamp != 0, E105_ACC_NOT_LOCKED);
        let timestamp = env::block_timestamp();
        let duration_sec = nano_to_sec(account.unlock_timestamp - timestamp) + append_duration_sec;

        let config = self.internal_config();
        require!(duration_sec >= config.min_locking_duration_sec, E302_INVALID_DURATION);
        require!(duration_sec <= config.max_locking_duration_sec, E302_INVALID_DURATION);

        let increased_ve_lpt = account.lock_lpt(amount, duration_sec, &config, self.data().lptoken_decimals);
        require!(increased_ve_lpt > 0, E101_INSUFFICIENT_BALANCE);
        self.mint_love_token(account_id, increased_ve_lpt);

        self.data_mut().cur_lock_lpt += amount;
        self.data_mut().cur_total_ve_lpt += increased_ve_lpt;

        self.update_impacted_proposals(&mut account, increased_ve_lpt, true);

        self.internal_set_account(account_id, account);

        Event::LptAppend {
            caller_id: account_id,
            deposit_amount: &U128(amount),
            increased_ve_lpt: &U128(increased_ve_lpt),
            duration: duration_sec,
        }
        .emit();
    }

    pub fn real_amount_and_refund(&self, amount: Balance) -> (Balance, Balance) {
        if self.data().lptoken_decimals > LOVE_DECIMAL {
            let decimals_diff = 10u128.pow((self.data().lptoken_decimals - LOVE_DECIMAL) as u32);
            (amount / decimals_diff * decimals_diff, amount % decimals_diff)
        } else {
            (amount, 0)
        }
    }
}