use cosmwasm_std::{Addr, to_binary, to_json_binary};
use sylvia::multitest::App;

use crate::{contract::sv::multitest_utils::CodeId, error::ContractError, ibc_msg::IbcExecuteMsg, whitelist_impl::sv::test_utils::Whitelist};

#[test]
fn instantiate() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner";

    let contract = code_id.instantiate(42, vec![]).call(owner).unwrap();

    let count = contract.count().unwrap().count;
    let owner_addr = contract.owner().unwrap().owner;
    assert_eq!(count, 42);
    assert_eq!(owner_addr, Addr::unchecked(owner));

    contract.increment_count().call(owner).unwrap();

    let count = contract.count().unwrap().count;
    assert_eq!(count, 43);
}

#[test]
fn decrement_below_zero() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner";

    let contract = code_id.instantiate(1, vec![]).call(owner).unwrap();

    let count = contract.count().unwrap().count;
    assert_eq!(count, 1);

    contract.decrement_count().call(owner).unwrap();

    let count = contract.count().unwrap().count;
    assert_eq!(count, 0);

    let err = contract.decrement_count().call(owner).unwrap_err();
    assert_eq!(err, ContractError::CannotDecrementCount);
}

#[test]
fn reset_counter() {
    let app: App<cw_multi_test::App> = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner";
    let admin = "admin";
    let random_user = "random";

    let contract = code_id
        .instantiate(1, vec![Addr::unchecked(admin)])
        .call(owner)
        .unwrap();
    
    let count = contract.count().unwrap().count;
    assert_eq!(count, 1);

    contract.reset_counter().call(admin).unwrap();

    let count = contract.count().unwrap().count;
    assert_eq!(count, 0);

    let err = contract.reset_counter().call(random_user).unwrap_err();
    assert_eq!(err, ContractError::NotAnAdminNorOwner(Addr::unchecked(random_user)));
}

#[test]
fn manage_admins() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner";
    let admin = "admin";
    let random_user = "random";

    let contract = code_id.instantiate(1, vec![]).call(owner).unwrap();

    // Admins list is empty
    let admins = contract.whitelist_proxy().admins().unwrap().admins;
    assert!(admins.is_empty());

    // Admin can be added
    contract
        .whitelist_proxy()
        .add_admin(admin.to_owned())
        .call(owner)
        .unwrap();

    let admins = contract.whitelist_proxy().admins().unwrap().admins;
    assert_eq!(admins, &[Addr::unchecked(admin)]);

    // Admin can NOT be removed nor added by a random Joe
    let err = contract
        .whitelist_proxy()
        .remove_admin(admin.to_owned())
        .call(random_user)
        .unwrap_err();
    assert_eq!(err, ContractError::NotTheOwner(Addr::unchecked(random_user)));

    // Admin can be removed
    contract
        .whitelist_proxy()
        .remove_admin(admin.to_owned())
        .call(owner)
        .unwrap();

    let admins = contract.whitelist_proxy().admins().unwrap().admins;
    assert!(admins.is_empty());
}

#[test]
fn ibc() {
    assert_eq!(IbcExecuteMsg::IncrementCount {}.to_string(), "increment_ibc_count");
    assert_eq!(IbcExecuteMsg::DecrementCount {}.to_string(), "decrement_ibc_count");
    assert_eq!(to_json_binary(&IbcExecuteMsg::IncrementCount {}), to_binary(&IbcExecuteMsg::IncrementCount {}));
    assert_eq!(to_json_binary(&IbcExecuteMsg::DecrementCount {}), to_binary(&IbcExecuteMsg::DecrementCount {}));
}