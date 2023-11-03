use cosmwasm_std::{Addr, Response, StdResult, Storage, ensure};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::CounterContract;
use crate::error::ContractError;
use crate::responses::AdminsResponse;
use crate::whitelist::Whitelist;

#[contract(module=crate::contract)]
#[messages(crate::whitelist as Whitelist)]
impl Whitelist for CounterContract<'_> {
    type Error = ContractError;

    #[msg(exec)]
    fn add_admin(&self, ctx: ExecCtx, admin: String) -> Result<Response, Self::Error> {
        let deps = ctx.deps;
        let storage: &mut dyn Storage = deps.storage;
        let admin = deps.api.addr_validate(&admin)?;

        ensure!(
            self.is_owner(storage, &ctx.info.sender),
            ContractError::NotTheOwner(ctx.info.sender)
        );

        self.admins
            .update(storage, |mut admins| -> StdResult<_> {
                admins.push(admin);
                admins.sort();
                admins.dedup();

                Ok(admins)
            })?;
        Ok(Response::default())
    }

    #[msg(exec)]
    fn remove_admin(&self, ctx: ExecCtx, admin: String) -> Result<Response, Self::Error> {
        let deps = ctx.deps;
        let storage: &mut dyn Storage = deps.storage;
        let admin = deps.api.addr_validate(&admin)?;

        ensure!(
            self.is_owner(storage, &ctx.info.sender),
            ContractError::NotTheOwner(ctx.info.sender)
        );

        self.admins
            .update(deps.storage, |admins| -> StdResult<_> {
                let admins = admins
                    .into_iter()
                    .filter(|iter_admin| -> bool { admin != iter_admin })
                    .collect();

                Ok(admins)
            })?;
        Ok(Response::default())
    }

    #[msg(query)]
    fn admins(&self, ctx: QueryCtx) -> Result<AdminsResponse, Self::Error> {
        let deps = ctx.deps;
        let admins: Vec<Addr> = self.admins.load(deps.storage)?;
        Ok(AdminsResponse { admins })
    }

    fn is_owner(&self, storage: &mut dyn Storage, address: &Addr) -> bool {
        // basically fail if unable to load state... be on the safe side
        let owner: Addr = self.owner.load(storage).unwrap_or(Addr::unchecked("error"));
        owner == address
    }
}
