use ibc_types::{core::client::ClientId, core::client::Height};

use std::string::String;

pub fn ibc_params() -> &'static str {
    "ibc/params"
}

// these are internal helpers that are used by penumbra-ibc, but not part of the IBC spec (that is,
// counterparties don't expect to verify proofs about them)
pub fn client_processed_heights(client_id: &ClientId, height: &Height) -> String {
    format!("ibc/clients/{client_id}/processedHeights/{height}")
}
pub fn client_processed_times(client_id: &ClientId, height: &Height) -> String {
    format!("ibc/clients/{client_id}/processedTimes/{height}")
}
pub fn counter() -> &'static str {
    "ibc/connection_counter"
}

pub mod ics20_value_balance {
    use ibc_types::core::channel::ChannelId;
    use penumbra_sdk_asset::asset;

    pub fn prefix() -> &'static str {
        "ibc/ics20-value-balance/"
    }

    pub fn by_asset_id(channel_id: &ChannelId, asset_id: &asset::Id) -> String {
        format!("ibc/ics20-value-balance/{channel_id}/{asset_id}")
    }
}
