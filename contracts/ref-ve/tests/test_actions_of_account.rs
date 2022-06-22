mod setup;
use crate::setup::*;
use std::collections::HashMap;

#[test]
fn test_withdraw_lpt() {
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.withdraw_lpt(&users.charlie, None), E100_ACC_NOT_REGISTERED);
    
    // 2 : E305_STILL_IN_LOCK
    assert_err!(e.withdraw_lpt(&users.alice, None), E305_STILL_IN_LOCK);

    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);

    // 3 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.withdraw_lpt(&users.alice, Some(0)), E101_INSUFFICIENT_BALANCE);

    // 4 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.withdraw_lpt(&users.alice, Some(to_yocto("101"))), E101_INSUFFICIENT_BALANCE);

    // 5 : The account doesn't have enough balance
    e.storage_deposit(&users.alice, &users.dude, to_yocto("0.00125")).assert_success();
    e.transfer(&users.alice, &users.dude, to_ve_token("1"));
    assert_err!(e.withdraw_lpt(&users.alice, Some(to_yocto("100"))), "The account doesn't have enough balance");
    assert_eq!(e.balance_of(&users.alice), to_ve_token("199"));
    assert_eq!(e.balance_of(&users.dude), to_ve_token("1"));

    // success alice all
    e.transfer(&users.dude, &users.alice, to_ve_token("1"));
    let mut before = e.get_metadata();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("100"));
    e.withdraw_lpt(&users.alice, None).assert_success();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("200"));
    before.cur_total_ve_lpt = to_ve_token("200").into();
    before.cur_lock_lpt = to_yocto("100").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // success bob half
    let mut before = e.get_metadata();
    e.withdraw_lpt(&users.bob, Some(to_yocto("50"))).assert_success();
    before.cur_total_ve_lpt = to_ve_token("100").into();
    before.cur_lock_lpt = to_yocto("50").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // success bob all
    let mut before = e.get_metadata();
    e.withdraw_lpt(&users.bob, Some(to_yocto("50"))).assert_success();
    before.cur_total_ve_lpt = to_ve_token("0").into();
    before.cur_lock_lpt = to_yocto("0").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // after vote withdraw
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.extend_whitelisted_accounts(&e.owner, vec![users.alice.account_id()]).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, to_yocto("1")).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VoteApprove, None).assert_success();
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VoteApprove, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.alice));
    e.withdraw_lpt(&users.alice, Some(to_yocto("50"))).assert_success();
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VoteApprove, amount: to_ve_token("100")
    })]), e.get_vote_detail(&users.alice));
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("100"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(to_ve_token("100"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
    e.withdraw_lpt(&users.alice, None).assert_success();
    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(0, e.get_proposal(0).unwrap().ve_amount_at_last_action);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::new(), e.get_vote_detail_history(&users.alice));
}