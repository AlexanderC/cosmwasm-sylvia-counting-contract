use std::fmt;
use cosmwasm_schema::cw_serde;
use thiserror::Error;

// The implementations for IBC only as Sylvia scaffolds it for other entry points... 

#[derive(Error, Debug)]
pub enum Never {}

#[cw_serde]
pub enum IbcExecuteMsg {
    IncrementCount {},
    DecrementCount {},
}

// To be able to use IbcExecuteMsg.to_string() when sending IBC msgs
impl fmt::Display for IbcExecuteMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IbcExecuteMsg::IncrementCount {} => write!(f, "increment_ibc_count"),
            IbcExecuteMsg::DecrementCount {} => write!(f, "decrement_ibc_count"),
        }
    }
}
