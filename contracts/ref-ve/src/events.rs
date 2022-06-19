use crate::*;
use near_sdk::serde_json::json;

const EVENT_STANDARD: &str = "ref-ve";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    ProposalCreate {
        proposer_id: &'a AccountId,
        proposal_id: u32,
        kind: &'a String,
        start_at: u64,
        duration_sec: u32,
    },
    ProposalRemove {
        proposer_id: &'a AccountId,
        proposal_id: u32,
    },
    RemovedProposalAssets {
        receiver_id: &'a AccountId,
        token_id: &'a AccountId,
        amount: &'a U128,
        success: bool,
    },
    ActionProposal {
        voter_id: &'a AccountId,
        proposal_id: u32,
        action: &'a String,
    },
    ActionCancel {
        voter_id: &'a AccountId,
        proposal_id: u32,
        action: &'a String,
    },
    LptWithdraw {
        caller_id: &'a AccountId,
        withdraw_amount: &'a U128,
        success: bool,
    },
    LptWithdrawLostfound {
        receiver_id: &'a AccountId,
        withdraw_amount: &'a U128,
        success: bool,
    },
    RewardWithdraw {
        caller_id: &'a AccountId,
        token_id: &'a AccountId,
        withdraw_amount: &'a U128,
        success: bool,
    },
    
    RewardDeposit {
        caller_id: &'a AccountId,
        proposal_id: u32,
        token_id: &'a AccountId,
        deposit_amount: &'a U128,
        total_amount: &'a U128,
        start_at: u64,
    },
    LptLock {
        caller_id: &'a AccountId,
        deposit_amount: &'a U128,
        increased_ve_lpt: &'a U128,
        duration: u32,
    },
    LptAppend {
        caller_id: &'a AccountId,
        deposit_amount: &'a U128,
        increased_ve_lpt: &'a U128,
        duration: u32,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        emit_event(&self);
    }
}

// Emit event that follows NEP-297 standard: https://nomicon.io/Standards/EventsFormat
// Arguments
// * `standard`: name of standard, e.g. nep171
// * `version`: e.g. 1.0.0
// * `event`: type of the event, e.g. nft_mint
// * `data`: associate event data. Strictly typed for each set {standard, version, event} inside corresponding NEP
pub (crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": EVENT_STANDARD_VERSION,
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    log!(format!("EVENT_JSON:{}", event_json));
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils, AccountId};

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice".to_string())
    }

    fn token_id() -> AccountId {
        AccountId::new_unchecked("ref".to_string())
    }

    #[test]
    fn event_proposal_create() {
        let proposer_id = &alice();
        let proposal_id = 0;
        let kind = &format!("{:?}", ProposalKind::FarmingReward{ farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string()], total_reward: 2});
        let start_at = 1000_u64;
        let duration_sec = 500_u32;
        Event::ProposalCreate { proposer_id, proposal_id, kind, start_at, duration_sec }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"proposal_create","data":[{"proposer_id":"alice","proposal_id":0,"kind":"FarmingReward { farm_list: [\"ref<>celo\", \"usn<>usdt\"], total_reward: 2 }","start_at":1000,"duration_sec":500}]}"#
        );
    }

    #[test]
    fn event_proposal_remove() {
        let proposer_id = &alice();
        let proposal_id = 0;
        Event::ProposalRemove { proposer_id, proposal_id }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"proposal_remove","data":[{"proposer_id":"alice","proposal_id":0}]}"#
        );
    }

    #[test]
    fn event_removed_proposal_assets() {
        let receiver_id = &alice();
        let token_id = &token_id();
        let amount = &U128(100);
        let success = true;
        Event::RemovedProposalAssets { receiver_id, token_id, amount, success }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"removed_proposal_assets","data":[{"receiver_id":"alice","token_id":"ref","amount":"100","success":true}]}"#
        );
    }

    #[test]
    fn event_action_proposal() {
        let voter_id = &alice();
        let proposal_id = 0;
        let action = &format!("{:?}", Action::VoteApprove);
        Event::ActionProposal { voter_id, proposal_id, action }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"action_proposal","data":[{"voter_id":"alice","proposal_id":0,"action":"VoteApprove"}]}"#
        );
    }

    #[test]
    fn event_action_cancel() {
        let voter_id = &alice();
        let proposal_id = 0;
        let action = &format!("{:?}", Action::VoteApprove);
        Event::ActionCancel { voter_id, proposal_id, action }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"action_cancel","data":[{"voter_id":"alice","proposal_id":0,"action":"VoteApprove"}]}"#
        );
    }

    #[test]
    fn event_lpt_withdraw() {
        let caller_id = &alice();
        let withdraw_amount = &U128(100);
        let success = true;
        Event::LptWithdraw { caller_id, withdraw_amount, success }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"lpt_withdraw","data":[{"caller_id":"alice","withdraw_amount":"100","success":true}]}"#
        );
    }

    #[test]
    fn event_lpt_withdraw_lostfound() {
        let receiver_id = &alice();
        let withdraw_amount = &U128(100);
        let success = true;
        Event::LptWithdrawLostfound { receiver_id, withdraw_amount, success }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"lpt_withdraw_lostfound","data":[{"receiver_id":"alice","withdraw_amount":"100","success":true}]}"#
        );
    }

    #[test]
    fn event_reward_withdraw() {
        let caller_id = &alice();
        let token_id = &token_id();
        let withdraw_amount = &U128(100);
        let success = true;
        Event::RewardWithdraw { caller_id, token_id, withdraw_amount, success }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"reward_withdraw","data":[{"caller_id":"alice","token_id":"ref","withdraw_amount":"100","success":true}]}"#
        );
    }

    #[test]
    fn event_reward_deposit() {
        let caller_id = &alice();
        let proposal_id = 0;
        let token_id = &token_id();
        let deposit_amount = &U128(100);
        let total_amount = &U128(1000);
        let start_at = 1000000;
        Event::RewardDeposit { caller_id, proposal_id, token_id, deposit_amount, total_amount, start_at }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"reward_deposit","data":[{"caller_id":"alice","proposal_id":0,"token_id":"ref","deposit_amount":"100","total_amount":"1000","start_at":1000000}]}"#
        );
    }

    #[test]
    fn event_lpt_deposit() {
        let caller_id = &alice();
        let deposit_amount = &U128(100);
        let increased_ve_lpt = &U128(200);
        let duration = 1000000;
        Event::LptLock { caller_id, deposit_amount, increased_ve_lpt, duration }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"lpt_lock","data":[{"caller_id":"alice","deposit_amount":"100","increased_ve_lpt":"200","duration":1000000}]}"#
        );
    }

    #[test]
    fn event_lpt_append() {
        let caller_id = &alice();
        let deposit_amount = &U128(100);
        let increased_ve_lpt = &U128(200);
        let duration = 1000000;
        Event::LptAppend { caller_id, deposit_amount, increased_ve_lpt, duration }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"ref-ve","version":"1.0.0","event":"lpt_append","data":[{"caller_id":"alice","deposit_amount":"100","increased_ve_lpt":"200","duration":1000000}]}"#
        );
    }
}