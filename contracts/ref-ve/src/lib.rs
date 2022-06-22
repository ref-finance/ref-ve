/*!
*
* lib.rs is the main entry point.
*
*/
mod owner;
mod account;
mod management;
mod actions_of_account;
mod actions_of_proposal;
mod actions_of_reward;
mod proposals;
mod proposals_action;
mod proposals_incentive;
mod token_receiver;
mod storage_impl;
mod errors;
mod events;
mod utils;
mod views;

pub use crate::owner::*;
pub use crate::account::*;
pub use crate::management::*;
pub use crate::actions_of_account::*;
pub use crate::actions_of_proposal::*;
pub use crate::actions_of_reward::*;
pub use crate::proposals::*;
pub use crate::proposals_action::*;
pub use crate::proposals_incentive::*;
pub use crate::token_receiver::*;
pub use crate::storage_impl::*;
pub use crate::errors::*;
pub use crate::events::*;
pub use crate::utils::*;
pub use crate::views::*;

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::BorshStorageKey;
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, Balance, PanicOnDefault, Promise, PromiseOrValue,
    PromiseResult, Timestamp, log
};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKeys {
    Operator,
    Config,
    Accounts,
    WhitelistedAccounts,
    Proposals,
    AccountProposalHistory { account_id: AccountId },
    RemovedProposalAssets
}

/// Contract config
#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub min_proposal_start_vote_offset_sec: u32,
    /// The min duration to stake LPT in seconds.
    pub min_locking_duration_sec: DurationSec,
    /// The max duration to stake LPT in seconds.
    pub max_locking_duration_sec: DurationSec,
    /// The rate of veLPT for the amount of LPT given for the maximum locking duration.
    /// Assuming the 100% multiplier at the 0 duration. Should be no less than 100%.
    /// E.g. 20000 means 200% multiplier (or 2X).
    pub max_locking_multiplier: u32,
}

impl Config {
    pub fn assert_valid(&self) {
        require!(
            self.max_locking_multiplier > MIN_LOCKING_REWARD_RATIO,
            E301_INVALID_RATIO
        );
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            min_proposal_start_vote_offset_sec: DEFAULT_MIN_PROPOSAL_START_VOTE_OFFSET_SEC,
            min_locking_duration_sec: DEFAULT_MIN_LOCKING_DURATION_SEC,
            max_locking_duration_sec: DEFAULT_MAX_LOCKING_DURATION_SEC,
            max_locking_multiplier: DEFAULT_MAX_LOCKING_REWARD_RATIO,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractData {
    pub owner_id: AccountId,
    pub operators: UnorderedSet<AccountId>,
    pub whitelisted_accounts: UnorderedSet<AccountId>,
    pub whitelisted_incentive_tokens: HashSet<AccountId>,

    pub config: LazyOption<Config>,

    // love token symbol
    pub symbol: String,
    // where lptoken_id from
    pub lptoken_contract_id: AccountId,
    // which lptoken used for locking
    pub lptoken_id: String,
    pub lptoken_decimals: u8,
    
    /// Last available id for the proposals.
    pub last_proposal_id: u32,
    /// Proposal map from ID to proposal information.
    pub proposals: UnorderedMap<u32, VProposal>,

    pub accounts: LookupMap<AccountId, VAccount>,
    pub account_count: u64,

    // total ve lpt amount
    pub cur_total_ve_lpt: Balance,
    // total lock lpt amount
    pub cur_lock_lpt: Balance,

    // if withdraw lpt encounter error, the lpt would go to here
    pub lostfound: Balance,

    pub removed_proposal_assets: UnorderedMap<AccountId, Balance>
}

/// Versioned contract data. Allows to easily upgrade contracts.
#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedContractData {
    V0100(ContractData),
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    ft: FungibleToken,
    data: VersionedContractData,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, symbol: String, lptoken_contract_id: AccountId, lptoken_id: String, lptoken_decimals: u8) -> Self {
        require!(!env::state_exists(), E000_ALREADY_INIT);
        Self {
            ft: FungibleToken::new(b"a".to_vec()),
            data: VersionedContractData::V0100(ContractData {
                owner_id,
                operators: UnorderedSet::new(StorageKeys::Operator),
                whitelisted_accounts: UnorderedSet::new(StorageKeys::WhitelistedAccounts),
                whitelisted_incentive_tokens: HashSet::new(),
                config: LazyOption::new(StorageKeys::Config, Some(&Config::default())),
                symbol,
                lptoken_contract_id,
                lptoken_id,
                lptoken_decimals,
                last_proposal_id: 0,
                proposals: UnorderedMap::new(StorageKeys::Proposals),
                accounts: LookupMap::new(StorageKeys::Accounts),
                account_count: 0,
                cur_total_ve_lpt: 0,
                cur_lock_lpt: 0,
                lostfound: 0,
                removed_proposal_assets: UnorderedMap::new(StorageKeys::RemovedProposalAssets),
            }),
        }
    }
}

impl Contract {
    pub fn internal_config(&self) -> Config {
        self.data().config.get().unwrap()
    }

    #[allow(unreachable_patterns)]
    fn data(&self) -> &ContractData {
        match &self.data {
            VersionedContractData::V0100(data) => data,
            _ => unimplemented!(),
        }
    }

    #[allow(unreachable_patterns)]
    fn data_mut(&mut self) -> &mut ContractData {
        match &mut self.data {
            VersionedContractData::V0100(data) => data,
            _ => unimplemented!(),
        }
    }

    fn is_owner_or_operators(&self) -> bool {
        env::predecessor_account_id() == self.data().owner_id
            || self
                .data()
                .operators
                .contains(&env::predecessor_account_id())
    }

    fn mint_love_token(&mut self, account_id: &AccountId, amount: Balance){
        if !self.ft.accounts.contains_key(account_id){
            self.ft.internal_register_account(account_id);
        }
        self.ft.internal_deposit(account_id, amount);
    }

    fn burn_love_token(&mut self, account_id: &AccountId, amount: Balance){
        self.ft.internal_withdraw(account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, ft);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        let data_url = "data:image/svg+xml;base64,\
        PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPCEtLSBHZW5l\
        cmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDIxLjAuMCwgU1ZHIEV4cG9ydCBQbHVn\
        LUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPgo8c3ZnIHZlcnNp\
        b249IjEuMSIgaWQ9IkxheWVyXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8y\
        MDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxp\
        bmsiIHg9IjBweCIgeT0iMHB4IgoJIHZpZXdCb3g9IjAgMCAyODggMzI0IiBzdHls\
        ZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAyODggMzI0OyIgeG1sOnNwYWNl\
        PSJwcmVzZXJ2ZSI+CjxzdHlsZSB0eXBlPSJ0ZXh0L2NzcyI+Cgkuc3Qwe2ZpbGw6\
        IzAwQzA4Qjt9Cjwvc3R5bGU+CjxnPgoJPHBhdGggZD0iTTE3My40LDE5MS40VjI2\
        OEgyNTBMMTczLjQsMTkxLjR6IE0xMDcuMiwxMjUuMmwzMCwzMGwzMC4zLTMwLjNW\
        NjkuMmgtNjAuNFYxMjUuMnogTTEwNy4yLDE1Mi4zVjI2OGg2MC40VjE1MmwtMzAu\
        MywzMC4zCgkJTDEwNy4yLDE1Mi4zeiBNMTc3LjEsNjkuMmgtMy43VjExOUwyMTIs\
        ODAuNUMyMDEuOCw3My4yLDE4OS42LDY5LjIsMTc3LjEsNjkuMnogTTM4LDE3NS41\
        VjI2OGg2My4zVjE0Ni40bC0xNy4xLTE3LjFMMzgsMTc1LjV6CgkJIE0zOCwxNDgu\
        NWw0Ni4yLTQ2LjJsMTcuMSwxNy4xVjY5LjJIMzhWMTQ4LjV6IE0yMzYuOCwxMjgu\
        OUwyMzYuOCwxMjguOWMwLTEyLjUtMy45LTI0LjctMTEuMi0zNC44bC01Mi4xLDUy\
        djQyLjRoMy43CgkJQzIxMC4xLDE4OC41LDIzNi44LDE2MS44LDIzNi44LDEyOC45\
        eiIvPgoJPHBvbHlnb24gY2xhc3M9InN0MCIgcG9pbnRzPSIyMTAuMiw1NiAyNTAs\
        OTUuOCAyNTAsNTYgCSIvPgo8L2c+Cjwvc3ZnPgo=";

        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: format!("{} Finance Token", self.data().symbol.clone()),
            symbol: self.data().symbol.clone(),
            icon: Some(String::from(data_url)),
            reference: None,
            reference_hash: None,
            decimals: LOVE_DECIMAL,
        }
    }
}
