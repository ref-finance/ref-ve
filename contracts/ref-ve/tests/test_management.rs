mod setup;

use crate::setup::*;

#[test]
fn test_modify_config(){
    let e = init_env();
    let users = Users::init(&e);

    let config = e.get_config();
    assert_eq!(config.min_proposal_start_vote_offset, DEFAULT_MIN_PROPOSAL_START_VOTE_OFFSET);
    assert_eq!(config.lock_near_per_proposal, DEFAULT_LOCK_NEAR_AMOUNT_FOR_PROPOSAL);
    assert_eq!(config.max_locking_duration_sec, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_eq!(config.max_locking_multiplier, DEFAULT_MAX_LOCKING_REWARD_RATIO);

    e.extend_operators(&e.owner, vec![&users.alice], 1).assert_success();

    e.modify_min_start_vote_offset(&users.alice, 500).assert_success();
    assert_eq!(e.get_config().min_proposal_start_vote_offset, 500);

    e.modify_lock_near_per_proposal(&users.alice, 1111).assert_success();
    assert_eq!(e.get_config().lock_near_per_proposal, 1111);

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
    assert_err!(e.return_lpt_lostfound(&users.alice, &users.alice, to_yocto("101")), E002_NOT_ALLOWED);

    // 2 : E101_INSUFFICIENT_BALANCE
    assert_err!(e.return_lpt_lostfound(&e.owner, &users.alice, to_yocto("101")), E101_INSUFFICIENT_BALANCE);

    // success
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), 0);
    e.return_lpt_lostfound(&e.owner, &users.alice, to_yocto("100")).assert_success();
    assert_eq!(e.mft_balance_of(&users.alice, &lpt_id()), to_yocto("100"));
    assert_eq!(0, e.get_metadata().lostfound.0);

}
