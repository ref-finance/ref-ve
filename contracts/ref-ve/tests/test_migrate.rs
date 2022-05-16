mod setup;
use crate::setup::*;

#[test]
fn test_update(){
    let e = Env::init_with_contract(previous_ref_ve_wasm_bytes());
    let users = Users::init(&e);

    assert_err!(
        e.upgrade_contract(&users.alice, ref_ve_wasm_bytes()),
        E002_NOT_ALLOWED
    );

    e.upgrade_contract(&e.owner, ref_ve_wasm_bytes()).assert_success();
    assert_eq!(e.get_metadata().version, "0.0.1".to_string());
}