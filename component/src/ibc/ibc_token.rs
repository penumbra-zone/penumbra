use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_crypto::asset;

/// IBC token respresents a token that was created through IBC.
pub struct IBCToken {
    transfer_path: String,
    base_denom: asset::Denom,
}

impl IBCToken {
    pub fn new(channel_id: &ChannelId, port_id: &PortId, denom: &str) -> Self {
        let transfer_path = format!("{}/{}/{}", port_id, channel_id, denom);

        let base_denom = asset::REGISTRY
            .parse_denom(&transfer_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_ibc_denom() {
        let expected_transfer_path = "transfer/channel-31/uatom";
        let ibctoken = IBCToken::new(&ChannelId::new(31), &PortId::transfer(), "uatom");
        println!("denom: {}, id: {}", ibctoken.denom(), ibctoken.id());
        assert_eq!(expected_transfer_path, ibctoken.transfer_path());
    }
}
