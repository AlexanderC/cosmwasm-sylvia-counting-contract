use cosmwasm_std::{IbcMsg, IbcTimeout, Response, to_json_binary};
use sylvia::contract;
use sylvia::types::{ExecCtx, QueryCtx};

use crate::contract::CounterContract;
use crate::error::ContractError;
use crate::ibc::Ibc;
use crate::ibc_msg::IbcExecuteMsg;
use crate::responses::*;

#[contract(module=crate::contract)]
#[messages(crate::ibc as Ibc)]
impl Ibc for CounterContract<'_> {
    type Error = ContractError;

    #[msg(query)]
    fn ibc_count(&self, ctx: QueryCtx, channel: String) -> Result<IbcCountResponse, Self::Error> {
        let count = self.count.load(ctx.deps.storage)?;
        let ibc_channel_counts = self
            .ibc_counts
            .may_load(ctx.deps.storage, channel.clone())?
            .unwrap_or_default();
        let ibc_channel_timeouts = self
            .ibc_timeouts
            .may_load(ctx.deps.storage, channel.clone())?
            .unwrap_or_default();
        Ok(IbcCountResponse {
            count,
            ibc_channel_counts,
            ibc_channel_timeouts,
        })
    }

    #[msg(exec)]
    fn increment_ibc_count(&self, ctx: ExecCtx, channel: String) -> Result<Response, Self::Error> {
        Ok(Response::new()
            .add_attribute("method", IbcExecuteMsg::IncrementCount {}.to_string())
            .add_attribute("channel", channel.clone())
            // outbound IBC message, where packet is then received on other chain
            .add_message(IbcMsg::SendPacket {
                channel_id: channel,
                data: to_json_binary(&IbcExecuteMsg::IncrementCount {})?,
                // default timeout of two minutes.
                timeout: IbcTimeout::with_timestamp(ctx.env.block.time.plus_seconds(120)),
            }))
    }

    #[msg(exec)]
    fn decrement_ibc_count(&self, ctx: ExecCtx, channel: String) -> Result<Response, Self::Error> {
        Ok(Response::new()
            .add_attribute("method", IbcExecuteMsg::DecrementCount {}.to_string())
            .add_attribute("channel", channel.clone())
            // outbound IBC message, where packet is then received on other chain
            .add_message(IbcMsg::SendPacket {
                channel_id: channel,
                data: to_json_binary(&IbcExecuteMsg::DecrementCount {})?,
                // default timeout of two minutes.
                timeout: IbcTimeout::with_timestamp(ctx.env.block.time.plus_seconds(120)),
            }))
    }
}
