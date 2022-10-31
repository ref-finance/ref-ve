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

    // 3 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.lock_lpt(&users.alice, 999999, DEFAULT_MAX_LOCKING_DURATION_SEC), E101_INSUFFICIENT_BALANCE);

    // 4 : E503_FIRST_LOCK_TOO_FEW
    assert_err!(e.lock_lpt(&users.alice, to_yocto("0.001"), DEFAULT_MAX_LOCKING_DURATION_SEC), E503_FIRST_LOCK_TOO_FEW);

    // success
    let mut before = e.get_metadata();
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    before.account_count = 1.into();
    before.cur_total_ve_lpt = to_ve_token("200").into();
    before.cur_lock_lpt = to_yocto("100").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // lock again
    let mut before = e.get_metadata();
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    before.cur_total_ve_lpt = to_ve_token("400").into();
    before.cur_lock_lpt = to_yocto("200").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));

    // append
    e.near.borrow_runtime_mut().cur_block.block_timestamp += 111;
    let mut before = e.get_metadata();
    assert_eq!("10", e.append_lpt(&users.alice, to_yocto("100") + 10, 0).unwrap_json_value());
    before.cur_total_ve_lpt = to_ve_token("600").into();
    before.cur_lock_lpt = to_yocto("300").into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));
    assert_eq!(to_yocto("100"), e.mft_balance_of(&users.alice, &lpt_id()));

    e.near.borrow_runtime_mut().cur_block.block_timestamp += DEFAULT_MAX_LOCKING_DURATION_SEC as u64 * 10u64.pow(9);
    assert_err!(e.lock_lpt(&users.alice, to_yocto("1"), DEFAULT_MIN_LOCKING_DURATION_SEC), E308_UNECONOMIC_LOCK);
}

#[test]
fn test_deposit_reward() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.extend_whitelisted_incentive_tokens(&e.owner, vec![tokens.nref.account_id(), tokens.wnear.account_id()]).assert_success();

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.extend_whitelisted_accounts(&e.owner, vec![users.dude.account_id()]).assert_success();
    e.storage_deposit(&users.dude, &users.dude, to_yocto("1"));

    e.create_proposal(&users.dude, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string()], total_reward: 20000 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();//, Some((tokens.nref.account_id(), IncentiveType::Evenly))
    e.create_proposal(&users.dude, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.dude, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 2, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("2000"));
    e.ft_mint(&tokens.wnear, &users.alice, to_yocto("2000"));
    e.ft_mint(&tokens.nusdc, &users.alice, to_yocto("2000"));

    // 1 : E207_INVALID_INCENTIVE_KEY
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 5), E207_INVALID_INCENTIVE_KEY);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1, 1), E207_INVALID_INCENTIVE_KEY);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 2, 1), E207_INVALID_INCENTIVE_KEY);

    // success 
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 1).assert_success();
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1, 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 2, 0).assert_success();
    assert_eq!(e.get_proposal(2).unwrap().incentive.get(&0).unwrap().incentive_amounts[0], to_yocto("100"));

    // 2 : E203_INVALID_INCENTIVE_TOKEN
    assert_err!(e.deposit_reward(&tokens.nusdc, &users.alice, to_yocto("100"), 0, 0), E203_INVALID_INCENTIVE_TOKEN);
    assert_err!(e.deposit_reward(&tokens.nusdc, &users.alice, to_yocto("100"), 1, 0), E203_INVALID_INCENTIVE_TOKEN);
    assert_err!(e.deposit_reward(&tokens.nusdc, &users.alice, to_yocto("100"), 2, 0), E203_INVALID_INCENTIVE_TOKEN);

    // 3 : E406_EXPIRED_PROPOSAL
    e.skip_time(DEFAULT_MIN_VOTING_DURATION_SEC);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 0), E406_EXPIRED_PROPOSAL);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1, 0), E406_EXPIRED_PROPOSAL);
    assert_err!(e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 2, 0), E406_EXPIRED_PROPOSAL);
}