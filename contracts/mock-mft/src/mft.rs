use crate::*;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{env, ext_contract, PromiseOrValue, assert_one_yocto, Gas, PromiseResult, log, Promise};

pub const STORAGE_BALANCE_MIN_BOUND: u128 = 1_250_000_000_000_000_000_000;
pub const NO_DEPOSIT: Balance = 0;
pub const TGAS: u64 = 1_000_000_000_000;
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10 * TGAS);
const GAS_FOR_MFT_TRANSFER_CALL: Gas = Gas(30 * TGAS);

pub(crate) fn parse_token_id(token_id: &String) -> String {
    require!(token_id.starts_with(":"), "ILLEGAL_TOKEN_ID");
    token_id[1..token_id.len()].to_string()
}

#[ext_contract(ext_self)]
trait MFTTokenResolver {
    fn mft_resolve_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

#[ext_contract(ext_mft_receiver)]
pub trait MFTTokenReceiver {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn mft_register(&mut self, token_id: String, account_id: AccountId) {
        let amount = env::attached_deposit();
        require!(amount >= STORAGE_BALANCE_MIN_BOUND, "ERR11_INSUFFICIENT_STORAGE");

        let inner_id = parse_token_id(&token_id);
        let mut token = self.tokens.get(&inner_id).expect("ERR_TOKEN_NOT_EXIST");

        let refund = if token.accounts.contains_key(&account_id) {
            amount
        } else {
            token.accounts.insert(&account_id, &0_u128);
            self.tokens.insert(&inner_id, &token);
            amount - STORAGE_BALANCE_MIN_BOUND
        };

        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }

    /// with force is true, would burn all remaining balance in the account
    #[payable]
    pub fn mft_unregister(&mut self, token_id: String, force: Option<bool>) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let inner_id = parse_token_id(&token_id);
        let force = force.unwrap_or(false);
        let mut token = self.tokens.get(&inner_id).expect("ERR_TOKEN_NOT_EXIST");
        if let Some(amount) = token.accounts.remove(&account_id) {
            if !force {
                require!(amount==0, "ERR_NON_ZERO_BALANCE");
            }
            token.total_supply -= amount;
            self.tokens.insert(&inner_id, &token);
        }
    }

    pub fn mft_balance_of(&self, token_id: String, account_id: AccountId) -> U128 {
        let inner_id = parse_token_id(&token_id);
        self.internal_mft_balance(&inner_id, &account_id).into()
    }

    pub fn mft_total_supply(&self, token_id: String) -> U128 {
        let inner_id = parse_token_id(&token_id);
        let token = self.tokens.get(&inner_id).unwrap_or_else(|| {
            Token {
                accounts: LookupMap::new(StorageKey::Accounts {
                    inner_id: inner_id.clone(),
                }),
                total_supply: 0,
            }
        });
        token.total_supply.into()
    }

    pub fn mft_metadata(&self, token_id: String) -> FungibleTokenMetadata {
        let inner_id = parse_token_id(&token_id);
        FungibleTokenMetadata {
            spec: "mft-1.0.0".to_string(),
            name: format!("mock-mft-{}", inner_id),
            symbol: format!("MOCK-MFT-{}", inner_id),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: self.decimals,
        }
    }

    #[payable]
    pub fn mft_transfer(
        &mut self,
        token_id: String,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let inner_id = parse_token_id(&token_id);
        self.internal_mft_transfer(&inner_id, &sender_id, &receiver_id, amount.0, memo);
    }

    #[payable]
    pub fn mft_transfer_call(
        &mut self,
        token_id: String,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let inner_id = parse_token_id(&token_id);
        self.internal_mft_transfer(&inner_id, &sender_id, &receiver_id, amount.0, memo);

        ext_mft_receiver::mft_on_transfer(
            token_id.clone(),
            sender_id.clone(),
            amount,
            msg,
            receiver_id.clone(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_MFT_TRANSFER_CALL - GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::mft_resolve_transfer(
            token_id,
            sender_id,
            receiver_id,
            amount,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    /// Returns how much was refunded back to the sender.
    #[private]
    pub fn mft_resolve_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount.0, unused_amount.0)
                } else {
                    amount.0
                }
            }
            PromiseResult::Failed => amount.0,
        };
        if unused_amount > 0 {
            let inner_id = parse_token_id(&token_id);
            let receiver_balance = self.internal_mft_balance(&inner_id, &receiver_id);
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                self.internal_mft_transfer(&inner_id, &receiver_id, &sender_id, refund_amount, None);
            }
        }
        U128(unused_amount)
    }

}

impl Contract {
    fn internal_mft_balance(&self, inner_id: &String, account_id: &AccountId) -> u128 {
        let token = self.tokens.get(&inner_id).expect("ERR_TOKEN_NOT_EXIST");
        token.accounts.get(&account_id).unwrap_or_default()
    }

    fn internal_mft_transfer(&mut self, inner_id: &String, sender_id: &AccountId, receiver_id: &AccountId, amount: u128, memo: Option<String>) {
        let mut token = self.tokens.get(&inner_id).expect("ERR_TOKEN_NOT_EXIST");
        let prev_sender_amount = token.accounts.get(&sender_id).expect("ERR_SENDER_NOT_REGISTERED");
        require!(prev_sender_amount >= amount, "NOT_ENOUGH_BALANCE");
        let prev_receiver_amount = token.accounts.get(&receiver_id).expect("ERR_RECEIVER_NOT_REGISTERED");

        token.accounts.insert(&sender_id, &(prev_sender_amount - amount));
        token.accounts.insert(&receiver_id, &(prev_receiver_amount + amount));
        
        self.tokens.insert(&inner_id, &token);

        if let Some(content) = memo {
            log!("mft_transfer memo: {}", content);
        }
    }
}