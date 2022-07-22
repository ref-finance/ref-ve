mod setup;
use crate::setup::*;

#[test]
fn test_storage_impl() {
    let e = init_env();
    let users = Users::init(&e);

    assert_err!(e.storage_deposit(&users.alice, &users.alice, to_yocto("0.000125")), E102_INSUFFICIENT_STORAGE);

    assert_err!(e.storage_withdraw(&users.alice, 1), E101_INSUFFICIENT_BALANCE);

    // alice register
    assert_eq!(e.get_metadata().account_count.0, 0);
    e.storage_deposit(&users.alice, &users.alice, to_yocto("0.00125")).assert_success();
    assert_eq!(e.get_metadata().account_count.0, 1);
    // alice register again
    e.storage_deposit(&users.alice, &users.alice, to_yocto("0.00125")).assert_success();
    assert_eq!(e.get_metadata().account_count.0, 1);
    assert_eq!(e.get_account_info(&users.alice).unwrap().sponsor_id, users.alice.account_id());

    // alice help bob register
    e.storage_deposit(&users.alice, &users.bob, to_yocto("0.00125")).assert_success();
    assert_eq!(e.get_metadata().account_count.0, 2);
    assert_eq!(e.get_account_info(&users.bob).unwrap().sponsor_id, users.alice.account_id());

    assert_eq!(e.storage_unregister(&users.bob, 1).unwrap_json::<bool>(), true);
    e.storage_deposit(&users.alice, &users.bob, to_yocto("0.00125")).assert_success();
    
    // alice unregister
    let user_balance_before = users.alice.account().unwrap().amount;
    assert_eq!(e.storage_unregister(&users.alice, 1).unwrap_json::<bool>(), true);
    let user_balance_after = users.alice.account().unwrap().amount;
    assert!(user_balance_after > user_balance_before);
    assert!(user_balance_after - user_balance_before < to_yocto("0.00125"));
    assert_eq!(e.get_metadata().account_count.0, 1);

    // bob unregister
    let user_balance_before = users.alice.account().unwrap().amount;
    let user_balance_before_bob = users.bob.account().unwrap().amount;
    assert_eq!(e.storage_unregister(&users.bob, 1).unwrap_json::<bool>(), true);
    let user_balance_after = users.alice.account().unwrap().amount;
    let user_balance_after_bob = users.bob.account().unwrap().amount;
    assert_eq!(user_balance_after - user_balance_before, to_yocto("0.00125"));
    assert!(user_balance_after_bob < user_balance_before_bob);
    assert_eq!(e.get_metadata().account_count.0, 0);

    // dude regisger by lock_lpt
    e.mft_mint(&lpt_inner_id(), &users.dude, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.dude, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_eq!(e.get_metadata().account_count.0, 1);

    assert_eq!(e.storage_unregister(&users.bob, 1).unwrap_json::<bool>(), false);
    assert_eq!(e.get_metadata().account_count.0, 1);

    assert_err!(e.storage_unregister(&users.dude, 1), E104_STILL_HAS_LPT);
    assert_eq!(e.get_metadata().account_count.0, 1);

    // dude unregister
    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.withdraw_lpt(&users.dude, None).assert_success();
    let user_balance_before = users.dude.account().unwrap().amount;
    assert_eq!(e.storage_unregister(&users.dude, 1).unwrap_json::<bool>(), true);
    let user_balance_after = users.dude.account().unwrap().amount;
    assert!(user_balance_after < user_balance_before);
    assert_eq!(e.get_metadata().account_count.0, 0);
}