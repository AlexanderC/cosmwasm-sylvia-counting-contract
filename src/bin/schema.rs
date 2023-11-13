use cosmwasm_schema::write_api;
use counting_contract::contract::sv::{InstantiateMsg, ContractExecMsg, ContractQueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}