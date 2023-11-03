use cosmwasm_std::{ensure, Addr, Response, StdResult, Storage};
// use cosmwasm_std::{
//     IbcChannelCloseMsg, IbcChannelConnectMsg,
//     IbcChannelOpenMsg, IbcPacketAckMsg,
//     IbcPacketReceiveMsg, IbcPacketTimeoutMsg
// };
use cw2::set_contract_version;
use cw_storage_plus::Item;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::contract;
#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use crate::error::*;
use crate::responses::*;

const CONTRACT_NAME: &str = "crates.io:counting-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CounterContract<'a> {
    pub(crate) count: Item<'a, u32>,
    pub(crate) owner: Item<'a, Addr>,
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
// #[sv::override_entry_point(ibc_channel_open=crate::ibc::ibc_channel_open(IbcChannelOpenMsg))]
// TODO: override IBC entry points from ibs.rs when possible... for now Sylvia does not support it
#[messages(crate::whitelist as Whitelist)]
impl CounterContract<'_> {
    pub const fn new() -> Self {
        Self {
            count: Item::new("count"),
            owner: Item::new("owner"),
            admins: Item::new("admins"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        count: u32,
        admins: Vec<Addr>,
    ) -> StdResult<Response> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        self.owner.save(ctx.deps.storage, &ctx.info.sender)?;
        self.count.save(ctx.deps.storage, &count)?;
        self.admins.save(ctx.deps.storage, &admins)?;
        Ok(Response::default())
    }

    #[msg(query)]
    pub fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
        let count = self.count.load(ctx.deps.storage)?;
        Ok(CountResponse { count })
    }

    #[msg(query)]
    fn owner(&self, ctx: QueryCtx) -> Result<OwnerResponse, ContractError> {
        let deps = ctx.deps;
        let owner: Addr = self.owner.load(deps.storage)?;
        Ok(OwnerResponse { owner })
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
        ensure!(
            self.is_admin_or_owner(storage, &ctx.info.sender),
            ContractError::NotAnAdminNorOwner(ctx.info.sender)
        );
        self.count.save(storage, &0)?;
        Ok(Response::default())
    }

    fn is_admin_or_owner(&self, storage: &mut dyn Storage, address: &Addr) -> bool {
        return self.is_owner(storage, address) || self.is_admin(storage, address);
    }

    fn is_owner(&self, storage: &mut dyn Storage, address: &Addr) -> bool {
        // basically fail if unable to load state... be on the safe side
        let owner: Addr = self.owner.load(storage).unwrap_or(Addr::unchecked("error"));
        owner == address
    }

    fn is_admin(&self, storage: &mut dyn Storage, address: &Addr) -> bool {
        // basically fail if unable to load state... be on the safe side
        let admins: Vec<Addr> = self.admins.load(storage).unwrap_or(vec![]);
        match admins.binary_search(address) {
            Ok(_) => true,
            _ => false,
        }
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn execute(
//     _deps: DepsMut,
//     env: Env,
//     _info: MessageInfo,
//     msg: ExecuteMsg,
// ) -> Result<Response, ContractError> {
//     match msg {
//         ExecuteMsg::Increment { channel } => Ok(Response::new()
//             .add_attribute("method", "execute_increment")
//             .add_attribute("channel", channel.clone())
//             // outbound IBC message, where packet is then received on other chain
//             .add_message(IbcMsg::SendPacket {
//                 channel_id: channel,
//                 data: to_binary(&IbcExecuteMsg::Increment {})?,
//                 // default timeout of two minutes.
//                 timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(120)),
//             })),
//     }
// }
