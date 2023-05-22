use ibc_types::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_crypto::asset;

/// IBC token respresents a token that was created through IBC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IbcToken {
    channel_id: ChannelId,
    port_id: PortId,
    original_denom: String,

    base_denom: asset::DenomMetadata,
}

impl IbcToken {
    pub fn new(channel_id: &ChannelId, port_id: &PortId, denom: &str) -> Self {
        let transfer_path = format!("{port_id}/{channel_id}/{denom}");

        let base_denom = asset::REGISTRY
            .parse_denom(&transfer_path)
            .expect("IBC denom is invalid");

        IbcToken {
            channel_id: channel_id.clone(),
            port_id: port_id.clone(),
            original_denom: denom.to_string(),

            base_denom,
        }
    }

    /// Get the base denomination for this IBC token.
    pub fn denom(&self) -> asset::DenomMetadata {
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
        format!(
            "{}/{}/{}",
            self.port_id, self.channel_id, self.original_denom
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_ibc_denom() {
        let expected_transfer_path = "transfer/channel-31/uatom";
        let ibctoken = IbcToken::new(&ChannelId::new(31), &PortId::transfer(), "uatom");
        println!("denom: {}, id: {}", ibctoken.denom(), ibctoken.id());
        assert_eq!(expected_transfer_path, ibctoken.transfer_path());
    }
}
