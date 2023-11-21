use ibc_types::core::{
    channel::msgs::{
        MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
        MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeout,
    },
    client::msgs::{MsgCreateClient, MsgSubmitMisbehaviour, MsgUpdateClient, MsgUpgradeClient},
    connection::msgs::{
        MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry,
    },
};

use ibc_types::DomainType as IbcTypesDomainType;
use ibc_types::TypeUrl as IbcTypesTypeUrl;

use penumbra_proto::penumbra::core::component::ibc::v1alpha1::{self as pb};
use penumbra_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::IbcRelay", into = "pb::IbcRelay")]
pub enum IbcRelay {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
    UpgradeClient(MsgUpgradeClient),
    SubmitMisbehavior(MsgSubmitMisbehaviour),
    ConnectionOpenInit(MsgConnectionOpenInit),
    ConnectionOpenTry(MsgConnectionOpenTry),
    ConnectionOpenAck(MsgConnectionOpenAck),
    ConnectionOpenConfirm(MsgConnectionOpenConfirm),
    ChannelOpenInit(MsgChannelOpenInit),
    ChannelOpenTry(MsgChannelOpenTry),
    ChannelOpenAck(MsgChannelOpenAck),
    ChannelOpenConfirm(MsgChannelOpenConfirm),
    ChannelCloseInit(MsgChannelCloseInit),
    ChannelCloseConfirm(MsgChannelCloseConfirm),
    RecvPacket(MsgRecvPacket),
    Acknowledgement(MsgAcknowledgement),
    Timeout(MsgTimeout),
    Unknown(pbjson_types::Any),
}

impl IbcRelay {
    /// Create a tracing span to track execution related to this action.
    ///
    /// The parent span is provided explicitly, so that this span can be constructed
    /// nested under an `Action` span.
    pub fn create_span(&self, parent: &tracing::Span) -> tracing::Span {
        match self {
            IbcRelay::CreateClient(msg) => {
                // HACK: not a better way to get tm light client data
                match ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
                    msg.client_state.clone(),
                ) {
                    Ok(tm_client) => {
                        tracing::info_span!(parent: parent, "CreateClient", chain_id = %tm_client.chain_id)
                    }
                    Err(_) => tracing::info_span!(parent: parent, "CreateClient"),
                }
            }
            IbcRelay::UpdateClient(msg) => {
                tracing::info_span!(parent: parent, "UpdateClient", client_id = %msg.client_id)
            }
            IbcRelay::UpgradeClient(msg) => {
                tracing::info_span!(parent: parent, "UpgradeClient", client_id = %msg.client_id)
            }
            IbcRelay::SubmitMisbehavior(msg) => {
                tracing::info_span!(parent: parent, "SubmitMisbehavior", client_id = %msg.client_id)
            }
            IbcRelay::ConnectionOpenInit(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenInit", client_id = %msg.client_id_on_a)
            }
            IbcRelay::ConnectionOpenTry(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenTry", client_id = %msg.client_id_on_b)
            }
            IbcRelay::ConnectionOpenAck(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenAck", connection_id = %msg.conn_id_on_a)
            }
            IbcRelay::ConnectionOpenConfirm(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenConfirm", connection_id = %msg.conn_id_on_b)
            }
            IbcRelay::ChannelOpenInit(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenInit", port_id = %msg.port_id_on_a)
            }
            IbcRelay::ChannelOpenTry(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenTry", port_id = %msg.port_id_on_b)
            }
            IbcRelay::ChannelOpenAck(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenAck", chan_id = %msg.chan_id_on_a)
            }
            IbcRelay::ChannelOpenConfirm(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenConfirm", chan_id = %msg.chan_id_on_b)
            }
            IbcRelay::ChannelCloseInit(msg) => {
                tracing::info_span!(parent: parent, "ChannelCloseInit", chan_id = %msg.chan_id_on_a)
            }
            IbcRelay::ChannelCloseConfirm(msg) => {
                tracing::info_span!(parent: parent, "ChannelCloseConfirm", chan_id = %msg.chan_id_on_b)
            }
            IbcRelay::RecvPacket(msg) => {
                tracing::info_span!(parent: parent, "RecvPacket", chan_id = %msg.packet.chan_on_b, seq = %msg.packet.sequence)
            }
            IbcRelay::Acknowledgement(msg) => {
                tracing::info_span!(parent: parent, "Acknowledgement", chan_id = %msg.packet.chan_on_a, seq = %msg.packet.sequence)
            }
            IbcRelay::Timeout(msg) => {
                tracing::info_span!(parent: parent, "Timeout", chan_id = %msg.packet.chan_on_a, seq = %msg.packet.sequence)
            }
            IbcRelay::Unknown(_) => {
                tracing::info_span!(parent: parent, "Unknown")
            }
        }
    }
}

impl DomainType for IbcRelay {
    type Proto = pb::IbcRelay;
}

impl TryFrom<pb::IbcRelay> for IbcRelay {
    type Error = anyhow::Error;
    fn try_from(value: pb::IbcRelay) -> Result<Self, Self::Error> {
        let raw_action = value
            .raw_action
            .ok_or_else(|| anyhow::anyhow!("empty IBC transaction is not allowed"))?;

        let action_type = raw_action.type_url.as_str();
        let raw_action_bytes = raw_action.value.clone();

        Ok(match action_type {
            MsgCreateClient::TYPE_URL => {
                let msg = MsgCreateClient::decode(raw_action_bytes)?;
                IbcRelay::CreateClient(msg)
            }
            MsgUpdateClient::TYPE_URL => {
                let msg = MsgUpdateClient::decode(raw_action_bytes)?;
                IbcRelay::UpdateClient(msg)
            }
            MsgUpgradeClient::TYPE_URL => {
                let msg = MsgUpgradeClient::decode(raw_action_bytes)?;
                IbcRelay::UpgradeClient(msg)
            }
            MsgConnectionOpenInit::TYPE_URL => {
                let msg = MsgConnectionOpenInit::decode(raw_action_bytes)?;
                IbcRelay::ConnectionOpenInit(msg)
            }
            MsgConnectionOpenTry::TYPE_URL => {
                let msg = MsgConnectionOpenTry::decode(raw_action_bytes)?;
                IbcRelay::ConnectionOpenTry(msg)
            }
            MsgConnectionOpenAck::TYPE_URL => {
                let msg = MsgConnectionOpenAck::decode(raw_action_bytes)?;
                IbcRelay::ConnectionOpenAck(msg)
            }
            MsgConnectionOpenConfirm::TYPE_URL => {
                let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes)?;
                IbcRelay::ConnectionOpenConfirm(msg)
            }
            MsgAcknowledgement::TYPE_URL => {
                let msg = MsgAcknowledgement::decode(raw_action_bytes)?;
                IbcRelay::Acknowledgement(msg)
            }
            MsgChannelOpenInit::TYPE_URL => {
                let msg = MsgChannelOpenInit::decode(raw_action_bytes)?;
                IbcRelay::ChannelOpenInit(msg)
            }
            MsgChannelOpenTry::TYPE_URL => {
                let msg = MsgChannelOpenTry::decode(raw_action_bytes)?;
                IbcRelay::ChannelOpenTry(msg)
            }
            MsgChannelOpenAck::TYPE_URL => {
                let msg = MsgChannelOpenAck::decode(raw_action_bytes)?;
                IbcRelay::ChannelOpenAck(msg)
            }
            MsgChannelOpenConfirm::TYPE_URL => {
                let msg = MsgChannelOpenConfirm::decode(raw_action_bytes)?;
                IbcRelay::ChannelOpenConfirm(msg)
            }
            MsgChannelCloseInit::TYPE_URL => {
                let msg = MsgChannelCloseInit::decode(raw_action_bytes)?;
                IbcRelay::ChannelCloseInit(msg)
            }
            MsgChannelCloseConfirm::TYPE_URL => {
                let msg = MsgChannelCloseConfirm::decode(raw_action_bytes)?;
                IbcRelay::ChannelCloseConfirm(msg)
            }
            MsgRecvPacket::TYPE_URL => {
                let msg = MsgRecvPacket::decode(raw_action_bytes)?;
                IbcRelay::RecvPacket(msg)
            }
            MsgTimeout::TYPE_URL => {
                let msg = MsgTimeout::decode(raw_action_bytes)?;
                IbcRelay::Timeout(msg)
            }
            _ => IbcRelay::Unknown(raw_action),
        })
    }
}

impl From<IbcRelay> for pb::IbcRelay {
    fn from(value: IbcRelay) -> Self {
        let raw_action = match value {
            IbcRelay::CreateClient(msg) => pbjson_types::Any {
                type_url: MsgCreateClient::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::UpdateClient(msg) => pbjson_types::Any {
                type_url: MsgUpdateClient::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::UpgradeClient(msg) => pbjson_types::Any {
                type_url: MsgUpgradeClient::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::SubmitMisbehavior(msg) => pbjson_types::Any {
                type_url: MsgSubmitMisbehaviour::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenInit(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenInit::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenTry(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenTry::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenAck(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenAck::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenConfirm(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenConfirm::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::Acknowledgement(msg) => pbjson_types::Any {
                type_url: MsgAcknowledgement::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenInit(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenInit::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenTry(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenTry::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenAck(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenAck::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenConfirm(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenConfirm::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelCloseInit(msg) => pbjson_types::Any {
                type_url: MsgChannelCloseInit::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelCloseConfirm(msg) => pbjson_types::Any {
                type_url: MsgChannelCloseConfirm::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::RecvPacket(msg) => pbjson_types::Any {
                type_url: MsgRecvPacket::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::Timeout(msg) => pbjson_types::Any {
                type_url: MsgTimeout::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::Unknown(raw_action) => raw_action,
        };
        pb::IbcRelay {
            raw_action: Some(raw_action),
        }
    }
}
