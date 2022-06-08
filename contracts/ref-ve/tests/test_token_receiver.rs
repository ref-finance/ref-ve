mod setup;
use crate::setup::*;

#[test]
fn test_lock_lpt(){
    let e = init_env();
    let users = Users::init(&e);
    
    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("400"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    // error scene 
    // 1 : E302_INVALID_DURATION
    assert_err!(e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC + 1), E302_INVALID_DURATION);

    // 2 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.lock_lpt(&users.alice, to_yocto("0"), DEFAULT_MAX_LOCKING_DURATION_SEC), E101_INSUFFICIENT_BALANCE);

    // success
    let mut before = e.get_metadata();
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    println!("{:?}",e.get_metadata());
    before.account_count = 1.into();
    before.cur_total_ve_lpt = to_ve_token("200").into();
    before.cur_lock_lpt = to_yocto("100").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));
    println!("{:?}", e.ft_balance_of(&e.ve_contract.user_account, &users.alice));

    // lock again
    let mut before = e.get_metadata();
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    println!("{:?}",e.get_metadata());
    before.cur_total_ve_lpt = to_ve_token("400").into();
    before.cur_lock_lpt = to_yocto("200").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));
    println!("{:?}", e.ft_balance_of(&e.ve_contract.user_account, &users.alice));

    // append
    let mut before = e.get_metadata();
    e.append_lpt(&users.alice, to_yocto("100"), 0).assert_success();
    println!("{:?}",e.get_metadata());
    before.cur_total_ve_lpt = to_ve_token("600").into();
    before.cur_lock_lpt = to_yocto("300").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));
    println!("{:?}", e.ft_balance_of(&e.ve_contract.user_account, &users.alice));
}

#[test]
fn test_withdraw_reward() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.extend_whitelisted_accounts(&e.owner, vec![users.dude.account_id()]).assert_success();
    e.storage_deposit(&users.dude, &users.dude, to_yocto("1"));

    e.create_proposal(&users.dude, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::Common { description: "common".to_string() }, to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, 0).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("200"));

    // 1 : E405_PROPOSAL_NOT_SUPPORT_INCENTIVE
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1), E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 2), E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);

    // success 
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0).assert_success();
    assert_eq!(e.get_proposal(0).unwrap().incentive.unwrap().incentive_amount, to_yocto("100"));

    // 2 : E406_EXPIRED_PROPOSAL
    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0), E406_EXPIRED_PROPOSAL);
}