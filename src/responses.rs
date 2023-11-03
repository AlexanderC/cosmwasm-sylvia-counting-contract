use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct IbcCountResponse {
    pub count: u32,
    pub ibc_channel_counts: u32,
    pub ibc_channel_timeouts: u32,
}

#[cw_serde]
pub struct CountResponse {
    pub count: u32,
}

#[cw_serde]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[cw_serde]
pub struct AdminsResponse {
    pub admins: Vec<Addr>,
}