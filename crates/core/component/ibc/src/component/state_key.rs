use ibc_types::{core::channel::ChannelId, core::client::ClientId, core::client::Height};

use penumbra_asset::asset;

use std::string::String;

pub fn ibc_params() -> &'static str {
    "ibc/params"
}

// these are internal helpers that are used by penumbra-ibc, but not part of the IBC spec (that is,
// counterparties don't expect to verify proofs about them)
pub fn client_processed_heights(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/processedHeights/{height}")
}
pub fn client_processed_times(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/processedTimes/{height}")
}
pub fn counter() -> &'static str {
    "ibc/connection_counter"
}
pub fn ics20_value_balance(channel_id: &ChannelId, asset_id: &asset::Id) -> String {
    format!("ibc/ics20-value-balance/{channel_id}/{asset_id}")
}
