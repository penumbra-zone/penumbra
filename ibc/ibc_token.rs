use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_crypto::asset;
use sha2::{Digest, Sha256};

/// IBC token respresents a token that was created through IBC.
pub struct IBCToken {
    transfer_path: String,
    base_denom: asset::Denom,
}

/// <https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md>
fn derive_ibc_denom(transfer_path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(transfer_path.as_bytes());

    let denom_bytes = hasher.finalize();
    let denom_hex = String::from_utf8(hex::encode_upper(denom_bytes).into()).unwrap();

    format!("ibc/{}", denom_hex)
}

impl IBCToken {
    pub fn new(channel_id: &ChannelId, port_id: &PortId, denom: &str) -> Self {
        let transfer_path = format!("{}/{}/{}", port_id, channel_id, denom);

        let base_denom = asset::REGISTRY
            .parse_denom(&derive_ibc_denom(transfer_path.as_str()))
            .expect("IBC denom is invalid");

        IBCToken {
            transfer_path,
            base_denom,
        }
    }

    /// Get the base denomination for this IBC token.
    pub fn denom(&self) -> asset::Denom {
        self.base_denom.clone()
    }

    /// Get the default display denomination for this IBC token.
    pub fn default_unit(&self) -> asset::Unit {
        self.base_denom.default_unit()
    }

    /// get the asset ID for this IBC token.
    pub fn id(&self) -> asset::Id {
        self.base_denom.id()
    }

    /// get the IBC transfer path of the IBC token.
    ///
    /// this takes the format of `port_id/channel_id/denom`.
    pub fn transfer_path(&self) -> String {
        self.transfer_path.clone()
    }
}
