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

    println!("{:?}", e.owner.view_method_call(
        e.ve_contract.contract.get_config()
    ).unwrap_json_value());
    e.upgrade_contract(&e.owner, ref_ve_wasm_bytes()).assert_success();
    assert_eq!(e.get_metadata().version, "0.2.0".to_string());
    println!("{:?}", e.get_config());
}