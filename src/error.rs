use cosmwasm_std::{StdError, Addr};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Cannot decrement count. Already at zero.")]
    CannotDecrementCount,

    #[error("Address '{0}' not an admin.")]
    NotAnAdmin(Addr),
}
