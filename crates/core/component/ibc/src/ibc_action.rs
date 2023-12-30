use ibc_proto::ibc::core::{
    channel::v1::{
        MsgAcknowledgement as RawMsgAcknowledgement,
        MsgChannelCloseConfirm as RawMsgChannelCloseConfirm,
        MsgChannelCloseInit as RawMsgChannelCloseInit, MsgChannelOpenAck as RawMsgChannelOpenAck,
        MsgChannelOpenConfirm as RawMsgChannelOpenConfirm,
        MsgChannelOpenInit as RawMsgChannelOpenInit, MsgChannelOpenTry as RawMsgChannelOpenTry,
        MsgRecvPacket as RawMsgRecvPacket, MsgTimeout as RawMsgTimeout,
    },
    client::v1::{
        MsgCreateClient as RawMsgCreateClient, MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour,
        MsgUpdateClient as RawMsgUpdateClient, MsgUpgradeClient as RawMsgUpgradeClient,
    },
    connection::v1::{
        MsgConnectionOpenAck as RawMsgConnectionOpenAck,
        MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm,
        MsgConnectionOpenInit as RawMsgConnectionOpenInit,
        MsgConnectionOpenTry as RawMsgConnectionOpenTry,
    },
};
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

use penumbra_proto::penumbra::core::component::ibc::v1alpha1::{self as pb};
use penumbra_proto::{DomainType, Name};
use penumbra_txhash::{EffectHash, EffectingData};
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

impl EffectingData for IbcRelay {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
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

        // fn calls not allowed in match patterns, so we have a huge if else block
        let outer_msg = if action_type == RawMsgCreateClient::type_url() {
            let msg = MsgCreateClient::decode(raw_action_bytes)?;
            IbcRelay::CreateClient(msg)
        } else if action_type == RawMsgUpdateClient::type_url() {
            let msg = MsgUpdateClient::decode(raw_action_bytes)?;
            IbcRelay::UpdateClient(msg)
        } else if action_type == RawMsgUpgradeClient::type_url() {
            let msg = MsgUpgradeClient::decode(raw_action_bytes)?;
            IbcRelay::UpgradeClient(msg)
        } else if action_type == RawMsgSubmitMisbehaviour::type_url() {
            let msg = MsgSubmitMisbehaviour::decode(raw_action_bytes)?;
            IbcRelay::SubmitMisbehavior(msg)
        } else if action_type == RawMsgConnectionOpenInit::type_url() {
            let msg = MsgConnectionOpenInit::decode(raw_action_bytes)?;
            IbcRelay::ConnectionOpenInit(msg)
        } else if action_type == RawMsgConnectionOpenTry::type_url() {
            let msg = MsgConnectionOpenTry::decode(raw_action_bytes)?;
            IbcRelay::ConnectionOpenTry(msg)
        } else if action_type == RawMsgConnectionOpenAck::type_url() {
            let msg = MsgConnectionOpenAck::decode(raw_action_bytes)?;
            IbcRelay::ConnectionOpenAck(msg)
        } else if action_type == RawMsgConnectionOpenConfirm::type_url() {
            let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes)?;
            IbcRelay::ConnectionOpenConfirm(msg)
        } else if action_type == RawMsgAcknowledgement::type_url() {
            let msg = MsgAcknowledgement::decode(raw_action_bytes)?;
            IbcRelay::Acknowledgement(msg)
        } else if action_type == RawMsgChannelOpenInit::type_url() {
            let msg = MsgChannelOpenInit::decode(raw_action_bytes)?;
            IbcRelay::ChannelOpenInit(msg)
        } else if action_type == RawMsgChannelOpenTry::type_url() {
            let msg = MsgChannelOpenTry::decode(raw_action_bytes)?;
            IbcRelay::ChannelOpenTry(msg)
        } else if action_type == RawMsgChannelOpenAck::type_url() {
            let msg = MsgChannelOpenAck::decode(raw_action_bytes)?;
            IbcRelay::ChannelOpenAck(msg)
        } else if action_type == RawMsgChannelOpenConfirm::type_url() {
            let msg = MsgChannelOpenConfirm::decode(raw_action_bytes)?;
            IbcRelay::ChannelOpenConfirm(msg)
        } else if action_type == RawMsgChannelCloseInit::type_url() {
            let msg = MsgChannelCloseInit::decode(raw_action_bytes)?;
            IbcRelay::ChannelCloseInit(msg)
        } else if action_type == RawMsgChannelCloseConfirm::type_url() {
            let msg = MsgChannelCloseConfirm::decode(raw_action_bytes)?;
            IbcRelay::ChannelCloseConfirm(msg)
        } else if action_type == RawMsgRecvPacket::type_url() {
            let msg = MsgRecvPacket::decode(raw_action_bytes)?;
            IbcRelay::RecvPacket(msg)
        } else if action_type == RawMsgTimeout::type_url() {
            let msg = MsgTimeout::decode(raw_action_bytes)?;
            IbcRelay::Timeout(msg)
        } else {
            IbcRelay::Unknown(raw_action)
        };

        Ok(outer_msg)
    }
}

impl From<IbcRelay> for pb::IbcRelay {
    fn from(value: IbcRelay) -> Self {
        let raw_action = match value {
            IbcRelay::CreateClient(msg) => pbjson_types::Any {
                type_url: RawMsgCreateClient::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::UpdateClient(msg) => pbjson_types::Any {
                type_url: RawMsgUpdateClient::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::UpgradeClient(msg) => pbjson_types::Any {
                type_url: RawMsgUpgradeClient::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::SubmitMisbehavior(msg) => pbjson_types::Any {
                type_url: RawMsgSubmitMisbehaviour::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenInit(msg) => pbjson_types::Any {
                type_url: RawMsgConnectionOpenInit::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenTry(msg) => pbjson_types::Any {
                type_url: RawMsgConnectionOpenTry::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenAck(msg) => pbjson_types::Any {
                type_url: RawMsgConnectionOpenAck::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ConnectionOpenConfirm(msg) => pbjson_types::Any {
                type_url: RawMsgConnectionOpenConfirm::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::Acknowledgement(msg) => pbjson_types::Any {
                type_url: RawMsgAcknowledgement::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenInit(msg) => pbjson_types::Any {
                type_url: RawMsgChannelOpenInit::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenTry(msg) => pbjson_types::Any {
                type_url: RawMsgChannelOpenTry::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenAck(msg) => pbjson_types::Any {
                type_url: RawMsgChannelOpenAck::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelOpenConfirm(msg) => pbjson_types::Any {
                type_url: RawMsgChannelOpenConfirm::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelCloseInit(msg) => pbjson_types::Any {
                type_url: RawMsgChannelCloseInit::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::ChannelCloseConfirm(msg) => pbjson_types::Any {
                type_url: RawMsgChannelCloseConfirm::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::RecvPacket(msg) => pbjson_types::Any {
                type_url: RawMsgRecvPacket::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::Timeout(msg) => pbjson_types::Any {
                type_url: RawMsgTimeout::type_url(),
                value: msg.encode_to_vec().into(),
            },
            IbcRelay::Unknown(raw_action) => raw_action,
        };
        pb::IbcRelay {
            raw_action: Some(raw_action),
        }
    }
}
