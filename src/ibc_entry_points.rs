use std::sync::Mutex;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, DepsMut, Env, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdResult,
};

use crate::{
    ack::{make_ack_fail, make_ack_success},
    contract::{IBC_VERSION, CounterContract},
    error::ContractError,
    ibc_msg::{IbcExecuteMsg, Never},
};

// ---------- THE STATE IS CLONED BECAUSE SYLVIA DOES NOT SUPPORT IBC NATIVELY!!! ---------- //
// ---------- See: https://github.com/CosmWasm/sylvia/issues/19#issuecomment-1792586062 ---------- //

// Access state by simply creating the contract (use Mutex based Singleton pattern)
// Alternatively state can be accessed directly by using cw_storage_plus
static CONTRACT: Mutex<CounterContract> = Mutex::new(CounterContract::new());

/// Work with Syliva counter state directly, avoiding sylvia entry points
pub fn try_update_state(
    deps: DepsMut,
    action: IbcExecuteMsg,
    channel: String,
) -> Result<u32, ContractError> {
    CONTRACT.lock().unwrap().ibc_counts.update(deps.storage, channel, |count| -> StdResult<u32> {
        Ok(count.unwrap_or_default() + 1)
    })?;
    CONTRACT.lock().unwrap().count.update(deps.storage, |count| -> Result<u32, ContractError> {
        match action {
            IbcExecuteMsg::IncrementCount {} => Ok(count + 1),
            IbcExecuteMsg::DecrementCount {} => {
                if count == 0 {
                    return Err(ContractError::CannotDecrementCount);
                }

                Ok(count - 1)
            }
        }
    })
}

// ----------------------------- IBC RELATED FUNCTIONALITY ------------------------------ //

/// Handles the `OpenInit` and `OpenTry` parts of the IBC handshake.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_ibc_channel(msg.channel(), msg.counterparty_version())?;
    Ok(IbcChannelOpenResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_ibc_channel(msg.channel(), msg.counterparty_version())?;

    // Initialize the count for this channel to zero.
    let channel = msg.channel().endpoint.channel_id.clone();
    CONTRACT.lock().unwrap().ibc_counts.save(deps.storage, channel.clone(), &0)?;

    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel_id", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel().endpoint.channel_id.clone();
    // Reset the state for the channel.
    CONTRACT.lock().unwrap().ibc_counts.remove(deps.storage, channel.clone());
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_close")
        .add_attribute("channel", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    // Regardless of if our processing of this packet works we need to
    // commit an ACK to the chain. As such, we wrap all handling logic
    // in a seprate function and on error write out an error ack.
    match do_ibc_packet_receive(deps, env, msg) {
        Ok(response) => Ok(response),
        Err(error) => Ok(IbcReceiveResponse::new()
            .add_attribute("method", "ibc_packet_receive")
            .add_attribute("error", error.to_string())
            .set_ack(make_ack_fail(error.to_string()))),
    }
}

pub fn do_ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    // The channel this packet is being relayed along on this chain.
    let channel = msg.packet.dest.channel_id;
    let msg: IbcExecuteMsg = from_json(&msg.packet.data)?;

    let count = try_update_state(deps, msg.clone(), channel)?;
    Ok(IbcReceiveResponse::new()
        .add_attribute("method", msg.to_string())
        .add_attribute("count", count.to_string())
        .set_ack(make_ack_success()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // Nothing to do here. We don't keep any state about the other
    // chain, just deliver messages so nothing to update.
    //
    // If we did care about how the other chain received our message
    // we could deserialize the data field into an `Ack` and inspect
    // it.
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_ack"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    CONTRACT.lock().unwrap().ibc_timeouts.update(
        deps.storage,
        // timed out packets are sent by us, so lookup based on packet
        // source, not destination.
        msg.packet.src.channel_id,
        |count| -> StdResult<_> { Ok(count.unwrap_or_default() + 1) },
    )?;
    // As with ack above, nothing to do here. If we cared about
    // keeping track of state between the two chains then we'd want to
    // respond to this likely as it means that the packet in question
    // isn't going anywhere.
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
}

pub fn validate_ibc_channel(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    // !!! IMPORTANT | ACHTUNG !!!
    // You might want to validate counterparty first, to be sure you are
    //      communicating with certain contract on certain chain. For that purpose use smth like:
    //          channel.endpoint.channel_id = "x" && channel.counterparty_endpoint.port_id = "y"
    // >>> Each channel only connects to a specific chain. 
    //      So if you know channel-4 on your chain is for osmosis, then you can trust channel-4.
    // NEVER use port_id standalone as it can be compromised by forking a chain...

    // We expect an unordered channel here. Ordered channels have the
    // property that if a message is lost the entire channel will stop
    // working until you start it again.
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OrderedIBCChannel);
    }

    if channel.version != IBC_VERSION {
        return Err(ContractError::InvalidIBCVersion {
            actual: channel.version.to_string(),
            expected: IBC_VERSION.to_string(),
        });
    }

    // Make sure that we're talking with a counterparty who speaks the
    // same "protocol" as us.
    //
    // For a connection between chain A and chain B being established
    // by chain A, chain B knows counterparty information during
    // `OpenTry` and chain A knows counterparty information during
    // `OpenAck`. We verify it when we have it but when we don't it's
    // alright.
    if let Some(counterparty_version) = counterparty_version {
        if counterparty_version != IBC_VERSION {
            return Err(ContractError::InvalidIBCVersion {
                actual: counterparty_version.to_string(),
                expected: IBC_VERSION.to_string(),
            });
        }
    }

    Ok(())
}
