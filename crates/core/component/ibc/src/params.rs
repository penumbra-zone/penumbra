use penumbra_proto::core::component::ibc::v1alpha1 as pb;
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::IbcParameters", into = "pb::IbcParameters")]
pub struct IBCParameters {
    /// Whether IBC (forming connections, processing IBC packets) is enabled.
    pub ibc_enabled: bool,
    /// Whether inbound ICS-20 transfers are enabled
    pub inbound_ics20_transfers_enabled: bool,
    /// Whether outbound ICS-20 transfers are enabled
    pub outbound_ics20_transfers_enabled: bool,
}

impl DomainType for IBCParameters {
    type Proto = pb::IbcParameters;
}

impl TryFrom<pb::IbcParameters> for IBCParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::IbcParameters) -> anyhow::Result<Self> {
        Ok(IBCParameters {
            ibc_enabled: msg.ibc_enabled,
            inbound_ics20_transfers_enabled: msg.inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled: msg.outbound_ics20_transfers_enabled,
        })
    }
}

impl From<IBCParameters> for pb::IbcParameters {
    fn from(params: IBCParameters) -> Self {
        pb::IbcParameters {
            ibc_enabled: params.ibc_enabled,
            inbound_ics20_transfers_enabled: params.inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled: params.outbound_ics20_transfers_enabled,
        }
    }
}

impl Default for IBCParameters {
    fn default() -> Self {
        Self {
            ibc_enabled: true,
            inbound_ics20_transfers_enabled: true,
            outbound_ics20_transfers_enabled: true,
        }
    }
}
