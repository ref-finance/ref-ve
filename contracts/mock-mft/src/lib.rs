use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    json_types::U128,
    near_bindgen, Balance, AccountId, PanicOnDefault, BorshStorageKey, require,
};

mod mft;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Tokens,
    Accounts {inner_id: String},
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Token {
    pub accounts: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    tokens: LookupMap<String, Token>,
    name: String,
    symbol: String,
    decimals: u8,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        Self {
            tokens: LookupMap::new(StorageKey::Tokens),
            name,
            symbol,
            decimals,
        }
    }

    pub fn mint(&mut self, inner_id: String, account_id: AccountId, amount: U128) {
        let mut token = self.tokens.get(&inner_id).unwrap_or_else(|| {
            Token {
                accounts: LookupMap::new(StorageKey::Accounts {
                    inner_id: inner_id.clone(),
                }),
                total_supply: 0,
            }
        });

        let new_amount = amount.0 + token.accounts.get(&account_id).unwrap_or_default();
        token.accounts.insert(&account_id, &(new_amount));
        token.total_supply += amount.0;

        self.tokens.insert(&inner_id, &token);
    }

    pub fn burn(&mut self, inner_id: String, account_id: AccountId, amount: U128) {
        let mut token = self.tokens.get(&inner_id).unwrap_or_else(|| {
            Token {
                accounts: LookupMap::new(StorageKey::Accounts {
                    inner_id: inner_id.clone(),
                }),
                total_supply: 0,
            }
        });
        let total = token.accounts.get(&account_id).unwrap_or_default();
        require!(total >= amount.0, "NOT_ENOUGH_BALANCE");

        token.accounts.insert(&account_id, &(total - amount.0));
        token.total_supply -= amount.0;

        self.tokens.insert(&inner_id, &token);
    }

}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Contract::new(String::from("TBD"), String::from("TBD"), 24);
        
        contract.mint(String::from("0"), accounts(0), 1_000_000.into());
        assert_eq!(contract.mft_balance_of(String::from(":0"), accounts(0)), 1_000_000.into());

        testing_env!(context
            .attached_deposit(1250000000000000000000u128)
            .predecessor_account_id(accounts(0))
            .build());
        contract.mft_register(String::from(":0"), accounts(1));
        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.mft_transfer(String::from(":0"), accounts(1), 1_000.into(), None);
        assert_eq!(contract.mft_balance_of(String::from(":0"), accounts(1)), 1_000.into());

        contract.burn(String::from("0"), accounts(1), 500.into());
        assert_eq!(contract.mft_balance_of(String::from(":0"), accounts(1)), 500.into());
    }
}