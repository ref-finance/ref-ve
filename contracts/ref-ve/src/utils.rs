use std::collections::HashSet;

// use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance, Timestamp, Gas, ext_contract};
use near_sdk::json_types::U128;

use crate::errors::{E501_INVALID_FARM_INFO, E502_INVALID_TOKEN_ID};

uint::construct_uint!(
    pub struct U256(4);
);

pub type DurationSec = u32;

pub const LOVE_DECIMAL: u8 = 18;

pub const DAY_SEC: DurationSec = 60 * 60 * 24;
pub const DEFAULT_MIN_VOTING_DURATION_SEC: DurationSec = DAY_SEC * 3;
pub const DEFAULT_MAX_VOTING_DURATION_SEC: DurationSec = DAY_SEC * 30;
pub const DEFAULT_MIN_PROPOSAL_START_VOTE_OFFSET_SEC: u32 = DAY_SEC;
pub const DEFAULT_MIN_LOCKING_DURATION_SEC: DurationSec = DAY_SEC * 30;
pub const DEFAULT_MAX_LOCKING_DURATION_SEC: DurationSec = DAY_SEC * 30 * 12; 
pub const DEFAULT_MAX_LOCKING_REWARD_RATIO: u32 = 20000;
pub const MIN_LOCKING_REWARD_RATIO: u32 = 10000;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_LPT_TRANSFER: Gas = Gas(20 * TGAS);
pub const GAS_FOR_RESOLVE_LPT_TRANSFER: Gas = Gas(10 * TGAS);
pub const GAS_FOR_REWARD_TRANSFER: Gas = Gas(20 * TGAS);
pub const GAS_FOR_RESOLVE_REWARD_TRANSFER: Gas = Gas(10 * TGAS);
pub const GAS_FOR_REMOVED_PROPOSAL_ASSETS: Gas = Gas(20 * TGAS);
pub const GAS_FOR_RESOLVE_REMOVED_PROPOSAL_ASSETS: Gas = Gas(10 * TGAS);

pub const DESCRIPTION_LIMIT: usize = 2048;
pub const MIN_FIRST_LOCK: u128 = 10u128.pow(22);
pub const STORAGE_BALANCE_MIN_BOUND: u128 = 1_250_000_000_000_000_000_000;

pub mod u64_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub mod u128_vec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::ser::Serialize;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &Vec<u128>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut res = Vec::with_capacity(num.len());
        for value in num {
            res.push(value.to_string());
        }
        Vec::serialize(&res, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u128>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_vec: Vec<String> = Vec::deserialize(deserializer)?;
        let mut res = Vec::with_capacity(str_vec.len());
        for s in str_vec.into_iter() {
            let item: u128 = s.parse().map_err(de::Error::custom)?;
            res.push(item);
        }
        Ok(res)
    }
}

pub mod u128_map_format {
    use near_sdk::serde::de;
    use near_sdk::serde::ser::Serialize;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};
    use near_sdk::{AccountId, Balance};
    use std::collections::HashMap;


    pub fn serialize<S>(info: &HashMap<AccountId, Balance>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut res = Vec::with_capacity(info.len());
        for (account_id, balance) in info {
            res.push((account_id, balance.to_string()));
        }
        Vec::serialize(&res, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<AccountId, Balance>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_vec: Vec<(AccountId, String)> = Vec::deserialize(deserializer)?;
        let mut res = HashMap::new();
        for (account_id, balance_str) in str_vec.into_iter() {
            let balance: u128 = balance_str.parse().map_err(de::Error::custom)?;
            res.insert(account_id, balance);
        }
        Ok(res)
    }
}

pub fn to_nano(sec: u32) -> Timestamp {
    Timestamp::from(sec) * 10u64.pow(9)
}

pub fn nano_to_sec(nano: u64) -> u32 {
    (nano / 10u64.pow(9)) as u32
}

pub(crate) fn u128_ratio(a: u128, num: u128, denom: u128) -> Balance {
    (U256::from(a) * U256::from(num) / U256::from(denom)).as_u128()
}

pub fn extra_incentive_tokens(farm_info: String) -> HashSet<AccountId> {
    let (farm_tokens_str, _) = farm_info.split_once('&').unwrap_or_else(|| env::panic_str(E501_INVALID_FARM_INFO));
    farm_tokens_str.split('|').into_iter().map(|a| a.parse().unwrap_or_else(|_| env::panic_str(E502_INVALID_TOKEN_ID))).collect()
}

#[ext_contract(ext_multi_fungible_token)]
pub trait MultiFungibleToken {
    fn mft_transfer(
        &mut self,
        token_id: String,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
}

#[ext_contract(ext_self)]
pub trait TokenPostActions {
    fn callback_post_withdraw_reward(
        &mut self, token_id: AccountId, sender_id: AccountId, amount: U128,
    );

    fn callback_removed_proposal_assets(
        &mut self, token_id: AccountId, receiver_id: AccountId, amount: U128,
    );

    fn callback_withdraw_lpt(&mut self, sender_id: AccountId, amount: U128);

    fn callback_withdraw_lpt_lostfound(&mut self, receiver_id: AccountId, amount: U128);

    fn callback_withdraw_lpt_slashed(&mut self, sender_id: AccountId, amount: U128);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn test_extra_incentive_tokens() {
        assert_eq!(HashSet::from(
            ["noct.near".parse().unwrap(), "nref.near".parse().unwrap()]), 
            extra_incentive_tokens("noct.near|nref.near&2657".to_string()));
        assert_eq!(HashSet::from(
            ["nusdt.near".parse().unwrap(), "nusdc.near".parse().unwrap(), "ndai.near".parse().unwrap()]), 
            extra_incentive_tokens("nusdt.near|nusdc.near|ndai.near&1910".to_string()));
    }
}