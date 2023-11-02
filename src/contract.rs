use cosmwasm_std::{Addr, Response, StdResult, ensure, Storage};
use cw_storage_plus::Item;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, entry_points};

use crate::error::*;
use crate::responses::*;

pub struct CounterContract<'a> {
    pub(crate) count: Item<'a, u32>,
    pub(crate) admins: Item<'a, Vec<Addr>>,
}

// Note that #[entry_points] is added above the #[contract].
// This is because contract removes the attributes such as #[msg(...)] on which both those macros rely.
// Remember to always place #[entry_points] first.
// To use contract as dependency, in your cargo specify:
// [dependencies]
// contract = { version = "0.1.0", features = ["library"] }
#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(ContractError)]
#[messages(crate::whitelist as Whitelist)]
impl CounterContract<'_> {
    pub const fn new() -> Self {
        Self {
            count: Item::new("count"),
            admins: Item::new("admins"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx, count: u32, mut admins: Vec<Addr>) -> StdResult<Response> {
        admins.push(ctx.info.sender);
        admins.sort();
        admins.dedup();

        self.count.save(ctx.deps.storage, &count)?;
        self.admins.save(ctx.deps.storage, &admins)?;
        Ok(Response::default())
    }

    #[msg(query)]
    pub fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
        let count = self.count.load(ctx.deps.storage)?;
        Ok(CountResponse { count })
    }

    #[msg(exec)]
    pub fn increment_count(&self, ctx: ExecCtx) -> StdResult<Response> {
        self.count
            .update(ctx.deps.storage, |count| -> StdResult<u32> {
                Ok(count + 1)
            })?;
        Ok(Response::default())
    }

    #[msg(exec)]
    pub fn decrement_count(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let count = self.count.load(ctx.deps.storage)?;
        if count == 0 {
            return Err(ContractError::CannotDecrementCount);
        }
        self.count.save(ctx.deps.storage, &(count - 1))?;
        Ok(Response::default())
    }

    #[msg(exec)]
    pub fn reset_counter(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let storage: &mut dyn Storage = ctx.deps.storage;
        ensure!(self.is_admin(storage, &ctx.info.sender), ContractError::NotAnAdmin(ctx.info.sender));
        self.count.save(storage, &0)?;
        Ok(Response::default())
    }

    fn is_admin(&self, storage: &mut dyn Storage, address: &Addr) -> bool  {
        // basically fail if unable to load state... be on the safe side
        let admins: Vec<Addr> = self.admins.load(storage).unwrap_or(vec![]);
        match admins.binary_search(address) { Ok(_) => true, _ => false }
    }
}
