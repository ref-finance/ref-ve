mod setup;
use ref_ve::ProposalStatus;

use crate::setup::*;

#[test]
fn test_modify_config(){
    let e = init_env();
    let users = Users::init(&e);

    let config = e.get_config();
    assert_eq!(config.min_proposal_voting_period, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_eq!(config.max_proposal_voting_period, DEFAULT_MAX_PROPOSAL_VOTING_PERIOD_SEC);
    assert_eq!(config.min_proposal_start_vote_offset, DEFAULT_MIN_PROPOSAL_START_VOTE_OFFSET);
    assert_eq!(config.lock_near_per_proposal, DEFAULT_LOCK_NEAR_AMOUNT_FOR_PROPOSAL);
    assert_eq!(config.min_per_lock_lpt_amount, DEFAULT_MIN_PER_LOCK_LPT_AMOUNT);
    assert_eq!(config.max_locking_duration_sec, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_eq!(config.max_locking_multiplier, DEFAULT_MAX_LOCKING_REWARD_RATIO);

    e.extend_operators(&e.owner, vec![&users.alice], 1).assert_success();

    e.modify_voting_period_range(&users.alice, 100, 200).assert_success();
    assert_eq!(e.get_config().min_proposal_voting_period, 100);
    assert_eq!(e.get_config().max_proposal_voting_period, 200);

    e.modify_min_start_vote_offset(&users.alice, 500).assert_success();
    assert_eq!(e.get_config().min_proposal_start_vote_offset, 500);

    e.modify_lock_near_per_proposal(&users.alice, 1111).assert_success();
    assert_eq!(e.get_config().lock_near_per_proposal, 1111);

    e.modify_min_per_lock_lpt_amount(&users.alice, 2222).assert_success();
    assert_eq!(e.get_config().min_per_lock_lpt_amount, 2222);

    assert_err!(e.modify_locking_policy(&users.alice, 1000, 3000), E301_INVALID_RATIO);

    e.modify_locking_policy(&users.alice, 1000, 30000).assert_success();
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
    assert_err!(e.return_lpt_lostfound(&e.dao, &users.alice, to_yocto("101")), E002_NOT_ALLOWED);

    // 2 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.return_lpt_lostfound(&e.owner, &users.alice, to_yocto("101")), E101_INSUFFICIENT_BALANCE);

    // success
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), 0);
    e.return_lpt_lostfound(&e.owner, &users.alice, to_yocto("100")).assert_success();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("100"));
    assert_eq!(0, e.get_metadata().lostfound.0);

}

#[test]
fn test_withdraw_lpt_slashed(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("100"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, to_yocto("1")).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VoteNonsense, None).assert_success();
    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);

    assert_eq!(e.get_proposal(0).unwrap().status.unwrap(), ProposalStatus::Expired);
    assert_eq!(e.get_proposal(0).unwrap().is_nonsense.unwrap(), true);

    // error scene 
    // 1 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.withdraw_lpt_slashed(&e.owner), E101_INSUFFICIENT_BALANCE);

    e.redeem_near_in_expired_proposal(&users.alice, 0);

    // 2 : E002_NOT_ALLOWED
    assert_err!(e.withdraw_lpt_slashed(&e.dao), E002_NOT_ALLOWED);

    assert_eq!(to_yocto("1"), e.get_metadata().slashed.0);
    assert_eq!(e.mft_balance_of(&e.owner, &lpt_id()), 0);

    // 3 : ERR_RECEIVER_NOT_REGISTERED
    assert_err!(e.withdraw_lpt_slashed(&e.owner), "ERR_RECEIVER_NOT_REGISTERED");
    assert_eq!(to_yocto("1"), e.get_metadata().slashed.0);

    // success
    e.mft_storage_deposit(&lpt_id(), &e.owner);
    e.withdraw_lpt_slashed(&e.owner).assert_success();
    assert_eq!(e.mft_balance_of(&e.owner, &lpt_id()), to_yocto("1"));
    assert_eq!(0, e.get_metadata().slashed.0);
}