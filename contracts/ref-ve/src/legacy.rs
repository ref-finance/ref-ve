use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ConfigV0100 {
    pub min_proposal_start_vote_offset_sec: u32,
    pub min_locking_duration_sec: DurationSec,
    pub max_locking_duration_sec: DurationSec,
    pub max_locking_multiplier: u32,
}

impl From<ConfigV0100> for Config {
    fn from(a: ConfigV0100) -> Self {
        Self { 
            min_proposal_start_vote_offset_sec: a.min_proposal_start_vote_offset_sec,
            min_locking_duration_sec: a.min_locking_duration_sec,
            max_locking_duration_sec: a.max_locking_duration_sec,
            max_locking_multiplier: a.max_locking_multiplier,
            min_voting_duration_sec: DEFAULT_MIN_VOTING_DURATION_SEC,
            max_voting_duration_sec: DEFAULT_MAX_VOTING_DURATION_SEC,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractDataV0100 {
    pub owner_id: AccountId,
    pub operators: UnorderedSet<AccountId>,
    pub whitelisted_accounts: UnorderedSet<AccountId>,
    pub whitelisted_incentive_tokens: HashSet<AccountId>,
    pub config: LazyOption<ConfigV0100>,
    pub symbol: String,
    pub lptoken_contract_id: AccountId,
    pub lptoken_id: String,
    pub lptoken_decimals: u8,
    pub last_proposal_id: u32,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub accounts: LookupMap<AccountId, VAccount>,
    pub account_count: u64,
    pub cur_total_ve_lpt: Balance,
    pub cur_lock_lpt: Balance,
    pub lostfound: Balance,
    pub removed_proposal_assets: UnorderedMap<AccountId, Balance>
}

impl From<ContractDataV0100> for ContractData {
    fn from(a: ContractDataV0100) -> Self {
        let ContractDataV0100 {
            owner_id,
            operators,
            whitelisted_accounts,
            whitelisted_incentive_tokens,
            config,
            symbol,
            lptoken_contract_id,
            lptoken_id,
            lptoken_decimals,
            last_proposal_id,
            proposals,
            accounts,
            account_count,
            cur_total_ve_lpt,
            cur_lock_lpt,
            lostfound,
            removed_proposal_assets
        } = a;
        Self {
            owner_id,
            operators,
            whitelisted_accounts,
            whitelisted_incentive_tokens,
            config: LazyOption::new(StorageKeys::Config, Some(&config.get().unwrap().into())),
            symbol,
            lptoken_contract_id,
            lptoken_id,
            lptoken_decimals,
            last_proposal_id,
            proposals,
            accounts,
            account_count,
            cur_total_ve_lpt,
            cur_lock_lpt,
            lostfound,
            removed_proposal_assets
        }
    }
}