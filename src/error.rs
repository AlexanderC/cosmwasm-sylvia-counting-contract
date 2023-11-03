use cosmwasm_std::{StdError, Addr};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Cannot decrement count. Already at zero.")]
    CannotDecrementCount,

    #[error("Address '{0}' is not the owner.")]
    NotTheOwner(Addr),

    #[error("Address '{0}' is not an admin.")]
    NotAnAdmin(Addr),

    #[error("Address '{0}' is not an admin nor owner.")]
    NotAnAdminNorOwner(Addr),

    #[error("Only unordered IBC channels are supported")]
    OrderedIBCChannel,

    #[error("invalid IBC channel version. Got ({actual}), expected ({expected})")]
    InvalidIBCVersion { actual: String, expected: String },
}
