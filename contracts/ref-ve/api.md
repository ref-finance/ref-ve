# API for REF-VE

## Logic
### User Roles
There are three user roles:
* User
    * Can lock lptoken to got veToken and loveToken,
    * Ve holders can participate in the referendum,
    * Can claim voting reward to inner account,
    * Can withdraw assets from inner account to user wallet,
* Operator (also could be a user)
    * adjust locking policy,
    * adjust the minimum start voting offset time,
    * manage whitelisted accounts,
* Owner (mostly is a DAO)
    * set owner to another account,
    * manage operators,
    * refund from seed lostfound,
    * upgrade the contract,

### Locking Policy
Every User can lock their lptoken with a specific period to got veToken/loveToken.

$$
X = A + A \frac{D(M_{max} - M_{min})}{D_{max} M_{min}}
$$

Where,  
$X$ is vetoken/loveToken amount,  
$A$ is locking lptoken amount,  
$D$ is user requested locking period,  
$D_{max}$ is the maximum locking duration, such as 360 days,  
$M_{min}$ is base BP ratio litterally is 10000,  
$M_{max}$ is multiple BP ratio related to $D_{max}$, say 20000,  

**Example:**  
Alice locking 30 lptoken for 60 days,  
$$
X = 30 + 30* \frac{60*(2.0 - 1.0)}{360*1.0} = 35
$$


**Append to Current Locking**  
Require:
- The new unlock timestamp must be later than current one;

Algorithm:  
1. Using the new duration to re-lock current locking balance, if new $X$ is larger, then update it, $X = Max(X, X_{new})$
2. Lock the append amount, got the extra $X_{append}$
3. $X = X + X_{append}$

## Interface
### User Register
This contract obeys NEP-145 to manage storage, but choose a fixed storage fee policy in this contract. Each user only needs deposit to lock a fixed 0.00125 NEAR as storage cost.

Detailed interface description could be found at [NEP-145](https://nomicon.io/Standards/StorageManagement.html).

Here we only list some common-use interfaces:

* `storage_deposit`, to register a user,
* `storage_unregister`, to unregister caller self and get 0.00125 NEAR back,
* `storage_balance_of`, to get given user storage balance,
* `storage_balance_bounds`, to get storage policy.

Note: 
- To sucessfully unregister, user should withdraw all his lptoken and reward tokens before calling `storage_unregister`.
- Support having a sponsor to deposit storage for user, in that case, when `storage_unregister`, the fixed 0.00125 near would transfer back to that sponsor. Can use `get_account_info(account_id)` to check it.

### User Lock/Append/Withdraw
```rust
enum MFTokenReceiverMessage {
    Lock { duration_sec: u32 },
    Append { append_duration_sec: u32 }
}
```
**Lock**  
are executed by calling lptoken's `mft_transfer_call ` with the following msg:
Eg:
```bash
near call $MFT mft_transfer_call '{"receiver_id": "'$VE'", "token_id": ":0", "amount": "1'$ZERO24'", "msg": "{\"Lock\":{\"duration_sec\":5184000}}"}' --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```
**Append** 
are executed by calling lptoken's `mft_transfer_call ` with the following msg:
Eg:
```bash
near call $MFT mft_transfer_call '{"receiver_id": "'$VE'", "token_id": ":0", "amount": "1'$ZERO24'", "msg": "{\"Append\":{\"append_duration_sec\":0}}"}' --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```
**Withdraw**  
are unified into one interface `withdraw_lpt`:
```rust
pub fn withdraw_lpt(&mut self, amount: Option<U128>)  -> Promise 
```
Eg:
```bash
near call $VE withdraw_lpt --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```
Note: 
1. If amount is not given, withdraw all balance.

### Deposit Reward to Proposal
are executed by calling reward token's `ft_transfer_call ` with the following msg:
```rust
enum FTokenReceiverMessage {
    Reward { proposal_id: u32, incentive_key: u32, incentive_type: IncentiveType }
}
```
Eg:
```bash
near call ref.$FT ft_transfer_call '{"receiver_id": "'$VE'", "amount": "36'$ZERO18'", "msg": "{\"Reward\":{\"proposal_id\":0, \"incentive_key\": 0, \"incentive_type\": \"Proportion\"}}"}' --account_id=u1.testnet --depositYocto=1 --gas=100$TGAS || true
```
### Proposal

**Create Proposal**  
```rust
pub enum ProposalKind {
    FarmingReward { 
        farm_list: Vec<String>,
        total_reward: u32
    },
    Poll {
        options: Vec<String>,
    },
    Common,
}
pub enum IncentiveType{
    Evenly,
    Proportion
}
pub fn create_proposal(
        &mut self,
        kind: ProposalKind,
        description: String,
        start_at: u32,
        duration_sec: u32,
        incentive_detail: Option<(AccountId, IncentiveType)>
    ) -> u32
```
Eg:

create farming reward proposal
```bash
near call $VE create_proposal '{"kind": {"FarmingReward":{"farm_list":["ref<>celo", "usn<>usdt"],"total_reward": 200000}}, "description": "FarmingReward Proposal", "start_at": 1655736586, "duration_sec": 86400 }' --account_id=u1.testnet 
```
create common proposal
```bash
near call $VE create_proposal '{"kind": "Common", "description": "Common Proposal", "start_at": 1655736586, "duration_sec": 5184000 }' --account_id=u1.testnet 
```
create poll
```bash
near call $VE create_proposal '{"kind": {"Poll":{ "options":["topic1", "topic2"]}}, "description": "Poll Proposal", "start_at": 1655736586, "duration_sec": 5184000 }' --account_id=u1.testnet 
```
**Remove Proposal** 
```rust
pub fn remove_proposal(&mut self, proposal_id: u32) -> bool
```
```bash
near call $VE remove_proposal '{"proposal_id":4}' --account_id=u1.testnet --depositYocto=1
```
**Action Proposal**
```rust
pub fn action_proposal(&mut self, proposal_id: u32, action: Action, memo: Option<String>) -> U128
```
voting farming reward
```bash
near call $VE action_proposal '{"proposal_id":0, "action": {"VoteFarm": {"farm_id": 0}}}' --account_id=u1.testnet
```
voting poll
```bash
near call $VE action_proposal '{"proposal_id":1, "action": {"VotePoll": {"poll_id": 0}}}' --account_id=u1.testnet
```
voting common
```bash
near call $VE action_proposal '{"proposal_id":1, "action": "VoteApprove"}' --account_id=u1.testnet
near call $VE action_proposal '{"proposal_id":1, "action": "VoteReject"}' --account_id=u1.testnet
```
**Action Cancel**
```rust
pub fn action_cancel(&mut self, proposal_id: u32) -> U128
```
```bash
near call $VE action_cancel '{"proposal_id":0}' --account_id=u1.testnet  --depositYocto=1
```
### Reward Related
**Claim And Withdraw**
```rust
pub fn claim_and_withdraw_all(&mut self)
```
```bash
near call $VE claim_and_withdraw_all --account_id=u1.testnet 
```
**Claim Reward**
```rust
pub fn claim_reward(&mut self, proposal_id: u32) 
```
```bash
near call $VE claim_reward '{"proposal_id":0}' --account_id=u1.testnet 
```
**Withdraw Reward**
```rust
pub fn withdraw_reward(&mut self, token_id: AccountId, amount: Option<U128>) 
```
```bash
near call $VE withdraw_reward '{"token_id":"xx"}' --account_id=u1.testnet 
```
Note: 
1. If amount is not given, withdraw all balance.
### Management Related
```rust
pub fn extend_whitelisted_accounts(&mut self, accounts: Vec<AccountId>);
pub fn remove_whitelisted_accounts(&mut self, accounts: Vec<AccountId>);

pub fn modify_min_start_vote_offset_sec(&mut self, min_start_vote_offset_sec: u32);
pub fn modify_locking_policy(&mut self, min_duration: DurationSec, max_duration: DurationSec, max_ratio: u32);

pub fn return_lpt_lostfound(&mut self, account_id: AccountId, amount: U128) -> Promise;
```

### All Views
**Contract Info**
```bash
near view $VE get_metadata
{
  version: '0.0.1',
  owner_id: 'ref-ve.testnet',
  operators: [],
  whitelisted_accounts: [],
  lptoken_contract_id: 'exchange.ref-dev.testnet',
  lptoken_id: ':269',
  lptoken_decimals: 24,
  account_count: '2',
  proposal_count: '0',
  cur_total_ve_lpt: '200000000000000000000000000',
  cur_lock_lpt: '100000000000000000000',
  lostfound: '0'
}

near view $VE get_config
{
  min_proposal_start_vote_offset_sec: '86400',
  min_locking_duration_sec: 2592000,
  max_locking_duration_sec: 31104000,
  max_locking_multiplier: 20000
}

near view $VE get_contract_storage_report
{ storage: '559993', locking_near: '5599930000000000000000000' }

near view $VE list_proposals
[
  {
    id: 0,
    proposer: 'user_account_id',
    kind: {
      FarmingReward: { farm_list: [ 'ref<>celo', 'usn<>usdt' ], total_reward: 200000 }
    },
    description: "FarmingReward Proposal",
    votes: [
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 }
    ],
    start_at: '1654650000000000000',
    end_at: '1654736400000000000',
    participants: '0',
    incentive: {
      '0': {
        incentive_type: 'Proportion',
        incentive_token_id: 'token_id',
        incentive_amount: '100000000000000000000',
        claimed_amount: '0'
      },
      '1': {
        incentive_type: 'Proportion',
        incentive_token_id: 'token_id',
        incentive_amount: '100000000000000000000',
        claimed_amount: '0'
      }
    },
    status: 'WarmUp',
    is_nonsense: null
  },
  {
    id: 1,
    proposer: 'user_account_id',
    kind: 'Common',
    description: "Common Proposal",
    votes: [
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 }
    ],
    start_at: '1654650000000000000',
    end_at: '1659834000000000000',
    participants: '0',
    incentive: {},
    status: 'WarmUp',
    is_nonsense: null
  },
  {
    id: 2,
    proposer: 'user_account_id',
    kind: { Poll: { options: [ 'topic1', 'topic2' ] } },
    description: "Poll Proposal",
    votes: [
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 }
    ],
    start_at: '1654650000000000000',
    end_at: '1659834000000000000',
    participants: '0',
    incentive: {},
    status: 'WarmUp',
    is_nonsense: null
  },
  {
    id: 3,
    proposer: 'user_account_id',
    kind: { Poll: { options: [ 'topic1', 'topic2' ] } },
    description: "Poll Proposal",
    votes: [
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 }
    ],
    start_at: '1654660800000000000',
    end_at: '1659844800000000000',
    participants: '0',
    incentive: {
      '0': {
        incentive_type: 'Proportion',
        incentive_token_id: 'token_id',
        incentive_amount: '100000000000000000000',
        claimed_amount: '0'
      }
    },
    status: 'WarmUp',
    is_nonsense: null
  }
]

near view $VE get_proposal '{"proposal_id": 0}'
{
    id: 0,
    proposer: 'user_account_id',
    kind: {
      FarmingReward: { farm_list: [ 'ref<>celo', 'usn<>usdt' ], total_reward: 200000 }
    },
    description: "FarmingReward Proposal",
    votes: [
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 },
      { total_ballots: '0', participants: 0 }
    ],
    start_at: '1654650000000000000',
    end_at: '1654736400000000000',
    participants: '0',
    incentive: {
      '0': {
        incentive_type: 'Proportion',
        incentive_token_id: 'token_id',
        incentive_amount: '100000000000000000000',
        claimed_amount: '0'
      },
      '1': {
        incentive_type: 'Proportion',
        incentive_token_id: 'token_id',
        incentive_amount: '100000000000000000000',
        claimed_amount: '0'
      }
    },
    status: 'WarmUp',
    is_nonsense: null
  }

near view $VE get_account_info '{"account_id": "xxx"}'
{
  sponsor_id: 'user_account_id',
  lpt_amount: '100000000000000000000000000',
  ve_lpt_amount: '200000000000000000000',
  unlock_timestamp: '1685625923349461711',
  duration_sec: 31104000,
  rewards: []
}

near view $VE get_unclaimed_rewards '{"account_id": "xxx"}'
{ 'token_id': '100000000000000000000' }

near view $VE get_vote_detail '{"account_id": "xxx"}'
{
  '7': {
    action: { VoteFarm: { farm_id: 0 } },
    amount: '200000000000000000000'
  }
}

near view $VE get_vote_detail_history '{"account_id": "xxx"}'
{
  '7': {
    action: { VoteFarm: { farm_id: 0 } },
    amount: '200000000000000000000'
  },
  '9': {
    action: { VotePoll: { poll_id: 0 } },
    amount: '200000000000000000000'
  },
  { '10': { 
    action: 'VoteApprove', 
    amount: '200000000000000000000'
  }
}

near view $VE get_unclaimed_proposal '{"account_id": "xxx"}'
{
  '9': {
    action: { VotePoll: { poll_id: 0 } },
    amount: '200000000000000000000'
  }
}

near view $VE list_removed_proposal_asserts
{ 'token_id': '200000000000000000000' }
```

**Storage**
```bash
near view $VE storage_balance_bounds
{ min: '1250000000000000000000', max: '1250000000000000000000' }

near view $VE storage_balance_of '{"account_id": "xxx"}'
{ total: '1250000000000000000000', available: '0' }
```