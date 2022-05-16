mod setup;
use crate::setup::*;

#[test]
fn test_storage_impl() {
    let e = init_env();
    let users = Users::init(&e);

    assert_err!(e.storage_deposit(&users.alice, &users.alice, 0), E700_NOT_NEED_STORAGE);
    assert_err!(e.storage_withdraw(&users.alice, 0), E700_NOT_NEED_STORAGE);
    assert_err!(e.storage_unregister(&users.alice, 0), E700_NOT_NEED_STORAGE);
}