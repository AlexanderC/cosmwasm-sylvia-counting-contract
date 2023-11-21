// #![allow(deprecated)]

use cosmwasm_std::{Response, StdError};
use sylvia::interface;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::responses::*;

#[interface]
pub trait Whitelist {
    type Error: From<StdError>;

    #[msg(exec)]
    fn add_admin(&self, ctx: ExecCtx, address: String) -> Result<Response, Self::Error>;

    #[msg(exec)]
    fn remove_admin(&self, ctx: ExecCtx, address: String) -> Result<Response, Self::Error>;

    #[msg(query)]
    fn admins(&self, ctx: QueryCtx) -> Result<AdminsResponse, Self::Error>;
}