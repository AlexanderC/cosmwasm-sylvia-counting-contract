use cosmwasm_std::Addr;
use sylvia::multitest::App;

use crate::{contract::multitest_utils::CodeId, error::ContractError, whitelist_impl::test_utils::Whitelist};

#[test]
fn instantiate() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner";

    let contract = code_id.instantiate(42, vec![]).call(owner).unwrap();

    let count = contract.count().unwrap().count;
    assert_eq!(count, 42);

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
fn manage_admins() {
  let app = App::default();
  let code_id = CodeId::store_code(&app);

  let owner = "owner";
  let admin = "admin";

  let contract = code_id.instantiate(1, vec![]).call(owner).unwrap();

  // Admins list is empty
  let admins = contract.whitelist_proxy().admins().unwrap().admins;
  assert_eq!(admins, &[Addr::unchecked(owner)]);

  // Admin can be added
  contract
      .whitelist_proxy()
      .add_admin(admin.to_owned())
      .call(owner)
      .unwrap();

  let admins = contract.whitelist_proxy().admins().unwrap().admins;
  assert_eq!(admins, &[Addr::unchecked(admin), Addr::unchecked(owner)]);

  // Admin can be removed
  contract
      .whitelist_proxy()
      .remove_admin(owner.to_owned())
      .call(owner)
      .unwrap();

  let admins = contract.whitelist_proxy().admins().unwrap().admins;
  assert_eq!(admins, &[Addr::unchecked(admin)]);
}