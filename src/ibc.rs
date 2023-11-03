use cosmwasm_std::{Response, StdError};
use sylvia::interface;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::responses::*;

#[interface]
pub trait Ibc {
    type Error: From<StdError>;

    #[msg(query)]
    fn ibc_count(&self, ctx: QueryCtx, channel: String) -> Result<IbcCountResponse, Self::Error>;

    #[msg(exec)]
    fn increment_ibc_count(&self, ctx: ExecCtx, channel: String) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn decrement_ibc_count(&self, ctx: ExecCtx, channel: String) -> Result<Response, Self::Error>;
}
