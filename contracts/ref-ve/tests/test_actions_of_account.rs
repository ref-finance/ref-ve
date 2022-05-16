mod setup;
use crate::setup::*;

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
    e.transfer(&users.alice, &users.bob, to_yocto("1"));
    assert_err!(e.withdraw_lpt(&users.alice, Some(to_yocto("100"))), "The account doesn't have enough balance");
    assert_eq!(e.balance_of(&users.alice), to_yocto("199"));
    assert_eq!(e.balance_of(&users.bob), to_yocto("201"));

    // success alice all
    e.transfer(&users.bob, &users.alice, to_yocto("1"));
    let mut before = e.get_metadata();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("100"));
    e.withdraw_lpt(&users.alice, None).assert_success();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("200"));
    before.account_count = 1.into();
    before.cur_total_ve_lpt = to_yocto("200").into();
    before.cur_lock_lpt = to_yocto("100").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // success bob half
    let mut before = e.get_metadata();
    e.withdraw_lpt(&users.bob, Some(to_yocto("50"))).assert_success();
    before.account_count = 1.into();
    before.cur_total_ve_lpt = to_yocto("100").into();
    before.cur_lock_lpt = to_yocto("50").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // success bob all
    let mut before = e.get_metadata();
    e.withdraw_lpt(&users.bob, Some(to_yocto("50"))).assert_success();
    before.account_count = 0.into();
    before.cur_total_ve_lpt = to_yocto("0").into();
    before.cur_lock_lpt = to_yocto("0").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // after vote withdraw
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.create_proposal(&users.alice, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, to_yocto("1")).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VoteApprove, None).assert_success();
    assert_eq!(vec![to_yocto("200"), 0, 0, to_yocto("200")], e.get_proposal(0).unwrap().votes);
    e.withdraw_lpt(&users.alice, None).assert_success();
    assert_eq!(vec![0, 0, 0, 0], e.get_proposal(0).unwrap().votes);
}