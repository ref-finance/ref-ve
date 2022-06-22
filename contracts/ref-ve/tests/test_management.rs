mod setup;

use crate::setup::*;

#[test]
fn test_modify_config(){
    let e = init_env();
    let users = Users::init(&e);

    let config = e.get_config();
    assert_eq!(config.min_proposal_start_vote_offset_sec, DEFAULT_MIN_PROPOSAL_START_VOTE_OFFSET_SEC);
    assert_eq!(config.min_locking_duration_sec, DEFAULT_MIN_LOCKING_DURATION_SEC);
    assert_eq!(config.max_locking_duration_sec, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_eq!(config.max_locking_multiplier, DEFAULT_MAX_LOCKING_REWARD_RATIO);

    e.extend_operators(&e.owner, vec![&users.alice], 1).assert_success();

    e.modify_min_start_vote_offset_sec(&users.alice, 500).assert_success();
    assert_eq!(e.get_config().min_proposal_start_vote_offset_sec, 500);

    assert_err!(e.modify_locking_policy(&users.alice, 500, 1000, 3000), E301_INVALID_RATIO);

    e.modify_locking_policy(&users.alice, 500, 1000, 30000).assert_success();
    assert_eq!(e.get_config().min_locking_duration_sec, 500);
    assert_eq!(e.get_config().max_locking_duration_sec, 1000);
    assert_eq!(e.get_config().max_locking_multiplier, 30000);
}

#[test]
fn test_return_lpt_lostfound(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("100"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);

    e.mft_unregister(&lpt_id(), &users.alice);
    e.withdraw_lpt(&users.alice, None).assert_success();
    assert_eq!(to_yocto("100"), e.get_metadata().lostfound.0);
    
    e.mft_storage_deposit(&lpt_id(), &users.alice);

    // error scene 
    // 1 : E002_NOT_ALLOWED
    assert_err!(e.return_lpt_lostfound(&users.alice, &users.alice, to_yocto("101")), E002_NOT_ALLOWED);

    // 2 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.return_lpt_lostfound(&e.owner, &users.alice, to_yocto("101")), E101_INSUFFICIENT_BALANCE);

    // success
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), 0);
    e.return_lpt_lostfound(&e.owner, &users.alice, to_yocto("100")).assert_success();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("100"));
    assert_eq!(0, e.get_metadata().lostfound.0);

}

#[test]
fn test_return_removed_proposal_assets(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.extend_whitelisted_incentive_tokens(&e.owner, vec![tokens.nref.account_id(), tokens.wnear.account_id()]).assert_success();
    assert_eq!(e.get_metadata().whitelisted_incentive_tokens.len(), 2);
    e.remove_whitelisted_incentive_tokens(&e.owner, vec![tokens.wnear.account_id()]).assert_success();
    assert_eq!(e.get_metadata().whitelisted_incentive_tokens.len(), 1);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.alice.account_id(), users.dude.account_id()]).assert_success();

    e.storage_deposit(&users.dude, &users.dude, to_yocto("1"));
    e.create_proposal(&users.alice, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string(), "usn.near|nusdt.near&3020".to_string()], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), 1000, 0).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, to_yocto("1")).assert_success();
    
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("2000"));
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 1).assert_success();
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1, 0).assert_success();

    assert_eq!(e.remove_proposal(&users.alice, 0).unwrap_json::<bool>(), true);
    assert_eq!(to_yocto("200"), e.list_removed_proposal_assets().get(&tokens.nref.account_id()).unwrap().0);
    assert_eq!(e.remove_proposal(&users.alice, 1).unwrap_json::<bool>(), true);
    assert_eq!(to_yocto("300"), e.list_removed_proposal_assets().get(&tokens.nref.account_id()).unwrap().0);


    // error scene 
    // 1 : E002_NOT_ALLOWED
    assert_err!(e.return_removed_proposal_assets(&e.near, &users.alice, &tokens.nref, to_yocto("200")), E002_NOT_ALLOWED);

    // 2 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.return_removed_proposal_assets(&e.owner, &users.alice, &tokens.nref, to_yocto("500")), E101_INSUFFICIENT_BALANCE);

    //success
    e.return_removed_proposal_assets(&e.owner, &users.alice, &tokens.nref, to_yocto("200")).assert_success();
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), to_yocto("1900"));
    assert_eq!(to_yocto("100"), e.list_removed_proposal_assets().get(&tokens.nref.account_id()).unwrap().0);

    e.ft_storage_unregister(&tokens.nref, &users.alice);

    e.return_removed_proposal_assets(&e.owner, &users.alice, &tokens.nref, to_yocto("100")).assert_success();
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), 0);
    assert_eq!(to_yocto("100"), e.list_removed_proposal_assets().get(&tokens.nref.account_id()).unwrap().0);

    e.ft_storage_deposit(&users.alice, &tokens.nref);

    e.return_removed_proposal_assets(&e.owner, &users.alice, &tokens.nref, to_yocto("100")).assert_success();
    assert_eq!(0, e.list_removed_proposal_assets().get(&tokens.nref.account_id()).unwrap().0);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), to_yocto("100"));
}
