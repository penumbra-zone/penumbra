use ibc_types::core::{
    channel::msgs::{
        MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
        MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeout,
    },
    client::msgs::{MsgCreateClient, MsgUpdateClient},
    connection::msgs::{
        MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry,
    },
};

use ibc_types::DomainType as IbcTypesDomainType;
use ibc_types::TypeUrl as IbcTypesTypeUrl;

use penumbra_proto::core::ibc::v1alpha1::{self as pb};
use penumbra_proto::{DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::IbcAction", into = "pb::IbcAction")]
pub enum IbcAction {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
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

impl IbcAction {
    /// Create a tracing span to track execution related to this action.
    ///
    /// The parent span is provided explicitly, so that this span can be constructed
    /// nested under an `Action` span.
    pub fn create_span(&self, parent: &tracing::Span) -> tracing::Span {
        match self {
            IbcAction::CreateClient(msg) => {
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
            IbcAction::UpdateClient(msg) => {
                tracing::info_span!(parent: parent, "UpdateClient", client_id = %msg.client_id)
            }
            IbcAction::ConnectionOpenInit(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenInit", client_id = %msg.client_id_on_a)
            }
            IbcAction::ConnectionOpenTry(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenTry", client_id = %msg.client_id_on_b)
            }
            IbcAction::ConnectionOpenAck(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenAck", connection_id = %msg.conn_id_on_a)
            }
            IbcAction::ConnectionOpenConfirm(msg) => {
                tracing::info_span!(parent: parent, "ConnectionOpenConfirm", connection_id = %msg.conn_id_on_b)
            }
            IbcAction::ChannelOpenInit(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenInit", port_id = %msg.port_id_on_a)
            }
            IbcAction::ChannelOpenTry(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenTry", port_id = %msg.port_id_on_b)
            }
            IbcAction::ChannelOpenAck(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenAck", chan_id = %msg.chan_id_on_a)
            }
            IbcAction::ChannelOpenConfirm(msg) => {
                tracing::info_span!(parent: parent, "ChannelOpenConfirm", chan_id = %msg.chan_id_on_b)
            }
            IbcAction::ChannelCloseInit(msg) => {
                tracing::info_span!(parent: parent, "ChannelCloseInit", chan_id = %msg.chan_id_on_a)
            }
            IbcAction::ChannelCloseConfirm(msg) => {
                tracing::info_span!(parent: parent, "ChannelCloseConfirm", chan_id = %msg.chan_id_on_b)
            }
            IbcAction::RecvPacket(msg) => {
                tracing::info_span!(parent: parent, "RecvPacket", chan_id = %msg.packet.chan_on_b, seq = %msg.packet.sequence)
            }
            IbcAction::Acknowledgement(msg) => {
                tracing::info_span!(parent: parent, "Acknowledgement", chan_id = %msg.packet.chan_on_a, seq = %msg.packet.sequence)
            }
            IbcAction::Timeout(msg) => {
                tracing::info_span!(parent: parent, "Timeout", chan_id = %msg.packet.chan_on_a, seq = %msg.packet.sequence)
            }
            IbcAction::Unknown(_) => {
                tracing::info_span!(parent: parent, "Unknown")
            }
        }
    }
}

impl TypeUrl for IbcAction {
    const TYPE_URL: &'static str = "/penumbra.core.ibc.v1alpha1.IbcAction";
}

impl DomainType for IbcAction {
    type Proto = pb::IbcAction;
}

impl TryFrom<pb::IbcAction> for IbcAction {
    type Error = anyhow::Error;
    fn try_from(value: pb::IbcAction) -> Result<Self, Self::Error> {
        let raw_action = value
            .raw_action
            .ok_or_else(|| anyhow::anyhow!("empty IBC transaction is not allowed"))?;

        let action_type = raw_action.type_url.as_str();
        let raw_action_bytes = raw_action.value.clone();

        Ok(match action_type {
            MsgCreateClient::TYPE_URL => {
                let msg = MsgCreateClient::decode(raw_action_bytes)?;
                IbcAction::CreateClient(msg)
            }
            MsgUpdateClient::TYPE_URL => {
                let msg = MsgUpdateClient::decode(raw_action_bytes)?;
                IbcAction::UpdateClient(msg)
            }
            MsgConnectionOpenInit::TYPE_URL => {
                let msg = MsgConnectionOpenInit::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenInit(msg)
            }
            MsgConnectionOpenTry::TYPE_URL => {
                let msg = MsgConnectionOpenTry::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenTry(msg)
            }
            MsgConnectionOpenAck::TYPE_URL => {
                let msg = MsgConnectionOpenAck::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenAck(msg)
            }
            MsgConnectionOpenConfirm::TYPE_URL => {
                let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenConfirm(msg)
            }
            MsgAcknowledgement::TYPE_URL => {
                let msg = MsgAcknowledgement::decode(raw_action_bytes)?;
                IbcAction::Acknowledgement(msg)
            }
            MsgChannelOpenInit::TYPE_URL => {
                let msg = MsgChannelOpenInit::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenInit(msg)
            }
            MsgChannelOpenTry::TYPE_URL => {
                let msg = MsgChannelOpenTry::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenTry(msg)
            }
            MsgChannelOpenAck::TYPE_URL => {
                let msg = MsgChannelOpenAck::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenAck(msg)
            }
            MsgChannelOpenConfirm::TYPE_URL => {
                let msg = MsgChannelOpenConfirm::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenConfirm(msg)
            }
            MsgChannelCloseInit::TYPE_URL => {
                let msg = MsgChannelCloseInit::decode(raw_action_bytes)?;
                IbcAction::ChannelCloseInit(msg)
            }
            MsgChannelCloseConfirm::TYPE_URL => {
                let msg = MsgChannelCloseConfirm::decode(raw_action_bytes)?;
                IbcAction::ChannelCloseConfirm(msg)
            }
            MsgRecvPacket::TYPE_URL => {
                let msg = MsgRecvPacket::decode(raw_action_bytes)?;
                IbcAction::RecvPacket(msg)
            }
            MsgTimeout::TYPE_URL => {
                let msg = MsgTimeout::decode(raw_action_bytes)?;
                IbcAction::Timeout(msg)
            }
            _ => IbcAction::Unknown(raw_action),
        })
    }
}

impl From<IbcAction> for pb::IbcAction {
    fn from(value: IbcAction) -> Self {
        let raw_action = match value {
            IbcAction::CreateClient(msg) => pbjson_types::Any {
                type_url: MsgCreateClient::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::UpdateClient(msg) => pbjson_types::Any {
                type_url: MsgUpdateClient::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ConnectionOpenInit(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenInit::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ConnectionOpenTry(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenTry::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ConnectionOpenAck(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenAck::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ConnectionOpenConfirm(msg) => pbjson_types::Any {
                type_url: MsgConnectionOpenConfirm::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::Acknowledgement(msg) => pbjson_types::Any {
                type_url: MsgAcknowledgement::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ChannelOpenInit(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenInit::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ChannelOpenTry(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenTry::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ChannelOpenAck(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenAck::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ChannelOpenConfirm(msg) => pbjson_types::Any {
                type_url: MsgChannelOpenConfirm::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ChannelCloseInit(msg) => pbjson_types::Any {
                type_url: MsgChannelCloseInit::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::ChannelCloseConfirm(msg) => pbjson_types::Any {
                type_url: MsgChannelCloseConfirm::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::RecvPacket(msg) => pbjson_types::Any {
                type_url: MsgRecvPacket::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::Timeout(msg) => pbjson_types::Any {
                type_url: MsgTimeout::TYPE_URL.to_string(),
                value: msg.encode_to_vec().into(),
            },
            IbcAction::Unknown(raw_action) => raw_action,
        };
        pb::IbcAction {
            raw_action: Some(raw_action),
        }
    }
}
