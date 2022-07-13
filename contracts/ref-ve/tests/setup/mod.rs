#![allow(dead_code)]

use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Balance, Gas, Timestamp};
use near_sdk_sim::runtime::GenesisConfig;
pub use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};

use mock_mft::ContractContract as MockMultiFungibleToken;

pub use ref_ve::{ContractContract as VeContract,
    Metadata, Proposal, ProposalKind, Action, Account, Config, VoteDetail, AccountInfo, VoteInfo
};

pub use ref_ve::{
    DAY_SEC,
    DEFAULT_MIN_PROPOSAL_START_VOTE_OFFSET_SEC, DEFAULT_MAX_LOCKING_DURATION_SEC, DEFAULT_MAX_LOCKING_REWARD_RATIO,
    DEFAULT_MIN_LOCKING_DURATION_SEC, DEFAULT_MIN_VOTING_DURATION_SEC, DEFAULT_MAX_VOTING_DURATION_SEC
};

pub use ref_ve::{
    E002_NOT_ALLOWED, 
    E100_ACC_NOT_REGISTERED, E101_INSUFFICIENT_BALANCE, E102_INSUFFICIENT_STORAGE, E103_STILL_HAS_REWARD, E104_STILL_HAS_LPT,
    E200_ALREADY_VOTED, E201_INVALID_VOTE, E203_INVALID_INCENTIVE_TOKEN, E204_VOTE_CAN_NOT_CANCEL, E205_NOT_VOTABLE, E206_NO_VOTED, E207_INVALID_INCENTIVE_KEY, E208_DESCRIPTION_TOO_LONG,
    E301_INVALID_RATIO, E302_INVALID_DURATION, E305_STILL_IN_LOCK, E306_INVALID_LOCK_DURATION_LIMIT, E307_INVALID_VOTING_DURATION_LIMIT,
    E402_INVALID_START_TIME, E404_PROPOSAL_NOT_EXIST, E406_EXPIRED_PROPOSAL,
    E503_FIRST_LOCK_TOO_FEW
};

mod users;
pub use users::*;
mod tokens;
pub use tokens::*;

mod owner;
pub use owner::*;
mod storage_impl;
pub use storage_impl::*;

mod actions_of_account;
pub use actions_of_account::*;
mod actions_of_proposal;
pub use actions_of_proposal::*;
mod actions_of_reward;
pub use actions_of_reward::*;
mod management;
pub use management::*;
mod token_receiver;
pub use token_receiver::*;
mod views;
pub use views::*;


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_REF_VE_WASM_BYTES => "../../res/ref_ve_v0100.wasm",
    REF_VE_WASM_BYTES => "../../res/ref_ve.wasm",

    FUNGIBLE_TOKEN_WASM_BYTES => "../../res/mock_ft.wasm",
    MULTI_FUNGIBLE_TOKEN_WASM_BYTES => "../../res/mock_mft.wasm",
}

pub fn previous_ref_ve_wasm_bytes() -> &'static [u8] {
    &PREV_REF_VE_WASM_BYTES
}

pub fn ref_ve_wasm_bytes() -> &'static [u8] {
    &REF_VE_WASM_BYTES
}

pub const NEAR: &str = "near";
pub const REF_VE_ID: &str = "ref_ve.near";
pub const FUNGIBLE_TOKEN_ID: &str = "token.near";
pub const MULTI_FUNGIBLE_TOKEN_ID: &str = "mutlitoken.near";
pub const OWNER_ID: &str = "owner.near";

pub const DAY_TS: Timestamp = 60 * 60 * 24 * 1_000_000_000;
pub const DEFAULT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);
pub const TOKEN_DECIMALS: u8 = 24;
pub const TOKEN_TOTAL_SUPPLY: Balance =
    1_000_000_000 * 10u128.pow(TOKEN_DECIMALS as _);

pub const GENESIS_TIMESTAMP: u64 = 1_600_000_000 * 10u64.pow(9);
pub struct Env {
    pub root: UserAccount,
    pub near: UserAccount,
    pub owner: UserAccount,
    pub ve_contract: ContractAccount<VeContract>,
    pub lptoken_contract: ContractAccount<MockMultiFungibleToken>,
}

pub fn lpt_id() -> String{
    ":0".to_string()
}

pub fn lpt_inner_id() -> String{
    "0".to_string()
}

pub fn to_nano(timestamp: u32) -> Timestamp {
    Timestamp::from(timestamp) * 10u64.pow(9)
}

pub fn to_sec(timestamp: Timestamp) -> u32 {
    (timestamp / 10u64.pow(9)) as u32
}

pub fn init_env() -> Env {
    Env::init_with_contract(&REF_VE_WASM_BYTES)
}

impl Env {
    pub fn init_with_contract(contract_bytes: &[u8]) -> Self {
        let mut genesis_config = GenesisConfig::default();
        genesis_config.genesis_time = GENESIS_TIMESTAMP;
        genesis_config.block_prod_time = 0;

        let root = init_simulator(Some(genesis_config));
        let near = root.create_user(
            AccountId::new_unchecked(NEAR.to_string()),
            to_yocto("1000000"),
        );
        let owner = near.create_user(
            AccountId::new_unchecked(OWNER_ID.to_string()),
            to_yocto("10000"),
        );

        let lptoken_contract = deploy!(
            contract: MockMultiFungibleToken,
            contract_id: MULTI_FUNGIBLE_TOKEN_ID.to_string(),
            bytes: &MULTI_FUNGIBLE_TOKEN_WASM_BYTES,
            signer_account: near,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(
                "Multi Fungible Token".to_string(),
                "MFT".to_string(),
                TOKEN_DECIMALS
            )
        );

        let ve_contract = deploy!(
            contract: VeContract,
            contract_id: REF_VE_ID.to_string(),
            bytes: &contract_bytes,
            signer_account: near,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(
                owner.account_id(),
                "loveRef".to_string(),
                lptoken_contract.account_id(),
                lpt_id(),
                24
            )
        );

        Self {
            root,
            near,
            owner,
            ve_contract,
            lptoken_contract
        }
    }

    pub fn upgrade_contract(&self, user: &UserAccount, contract_bytes: &[u8]) -> ExecutionResult {
        user
            .create_transaction(account_id(REF_VE_ID))
            .function_call("upgrade".to_string(), contract_bytes.to_vec(), MAX_GAS.0, 0)
            .submit()
    }

    pub fn skip_time(&self, seconds: u32) {
        self.near.borrow_runtime_mut().cur_block.block_timestamp += to_nano(seconds);
    }

    pub fn current_time(&self) -> u64{
        self.near.borrow_runtime().cur_block.block_timestamp
    }
}

pub fn d(value: Balance, decimals: u8) -> Balance {
    value * 10u128.pow(decimals as _)
}
pub fn account_id(account_id: &str) -> AccountId {
    AccountId::new_unchecked(account_id.to_string())
}

pub fn init_token(e: &Env, token_account_id: &AccountId, decimals: u8) -> UserAccount {
    let token = e.near.deploy_and_init(
        &FUNGIBLE_TOKEN_WASM_BYTES,
        token_account_id.clone(),
        "new",
        &json!({
            "name": token_account_id.to_string(),
            "symbol": token_account_id.to_string(),
            "decimals": decimals
        })
        .to_string()
        .into_bytes(),
        to_yocto("10"),
        DEFAULT_GAS.0,
    );

    e.owner.call(
        token_account_id.clone(),
        "storage_deposit",
        &json!({ "account_id": e.ve_contract.account_id() }).to_string().into_bytes(),
        DEFAULT_GAS.0,
        125 * env::STORAGE_PRICE_PER_BYTE,
    )
    .assert_success();
    token
}

#[macro_export]
macro_rules! assert_err{
    (print $exec_func: expr)=>{
        println!("{:?}", $exec_func.promise_errors()[0].as_ref().unwrap().status());
    };
    ($exec_func: expr, $err_info: expr)=>{
        assert!(format!("{:?}", $exec_func.promise_errors()[0].as_ref().unwrap().status()).contains($err_info));
    };
}

pub fn to_ve_token(value: &str) -> u128 {
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(18);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(18 - power);
        part1 + part2
    } else {
        part1
    }
}