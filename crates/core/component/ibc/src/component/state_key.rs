use ibc_types::{core::channel::ChannelId, core::client::ClientId, core::client::Height};

use penumbra_asset::asset;

use std::string::String;

pub fn ibc_params() -> &'static str {
    "ibc_params"
}

// TODO (ava): move these to ibc-types eventually
pub fn client_processed_heights(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/processedHeights/{height}")
}
pub fn client_processed_times(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/processedTimes/{height}")
}

pub mod connections {
    use ibc_types::core::client::ClientId;
    use ibc_types::core::connection::ConnectionId;

    use std::string::String;

    // This is part of the ICS-3 spec but not exposed yet:
    // https://github.com/cosmos/ibc/tree/main/spec/core/ics-003-connection-semantics
    #[allow(dead_code)]
    pub fn by_client_id_list(client_id: &ClientId) -> String {
        format!("clients/{client_id}/connections/")
    }

    pub fn by_client_id(client_id: &ClientId, connection_id: &ConnectionId) -> String {
        format!(
            "clients/{}/connections/{}",
            client_id,
            connection_id.as_str()
        )
    }

    pub fn by_connection_id(connection_id: &ConnectionId) -> String {
        format!("connections/{}", connection_id.as_str())
    }

    pub fn counter() -> &'static str {
        "ibc/ics03-connection/connection_counter"
    }
}

pub fn ics20_value_balance(channel_id: &ChannelId, asset_id: &asset::Id) -> String {
    format!("ics20-value-balance/{channel_id}/{asset_id}")
}
