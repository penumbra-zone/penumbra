use ibc_proto::protobuf::Protobuf;
use ibc_types2::core::{
    ics02_client::msgs::{
        create_client::{MsgCreateClient, TYPE_URL as CREATE_CLIENT},
        update_client::{MsgUpdateClient, TYPE_URL as UPDATE_CLIENT},
    },
    ics03_connection::msgs::{
        conn_open_ack::{MsgConnectionOpenAck, TYPE_URL as CONNECTION_OPEN_ACK},
        conn_open_confirm::{MsgConnectionOpenConfirm, TYPE_URL as CONNECTION_OPEN_CONFIRM},
        conn_open_init::{MsgConnectionOpenInit, TYPE_URL as CONNECTION_OPEN_INIT},
        conn_open_try::{MsgConnectionOpenTry, TYPE_URL as CONNECTION_OPEN_TRY},
    },
    ics04_channel::msgs::{
        acknowledgement::{MsgAcknowledgement, TYPE_URL as ACKNOWLEDGEMENT},
        chan_close_confirm::{MsgChannelCloseConfirm, TYPE_URL as CHANNEL_CLOSE_CONFIRM},
        chan_close_init::{MsgChannelCloseInit, TYPE_URL as CHANNEL_CLOSE_INIT},
        chan_open_ack::{MsgChannelOpenAck, TYPE_URL as CHANNEL_OPEN_ACK},
        chan_open_confirm::{MsgChannelOpenConfirm, TYPE_URL as CHANNEL_OPEN_CONFIRM},
        chan_open_init::{MsgChannelOpenInit, TYPE_URL as CHANNEL_OPEN_INIT},
        chan_open_try::{MsgChannelOpenTry, TYPE_URL as CHANNEL_OPEN_TRY},
        recv_packet::{MsgRecvPacket, TYPE_URL as RECV_PACKET},
        timeout::{MsgTimeout, TYPE_URL as TIMEOUT},
    },
};
use penumbra_proto::{
    core::ibc::v1alpha1::{self as pb},
    DomainType, TypeUrl,
};
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
                match ibc_types::clients::ics07_tendermint::client_state::ClientState::try_from(
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
            CREATE_CLIENT => {
                let msg = MsgCreateClient::decode(raw_action_bytes)?;
                IbcAction::CreateClient(msg)
            }
            UPDATE_CLIENT => {
                let msg = MsgUpdateClient::decode(raw_action_bytes)?;
                IbcAction::UpdateClient(msg)
            }
            CONNECTION_OPEN_INIT => {
                let msg = MsgConnectionOpenInit::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenInit(msg)
            }
            CONNECTION_OPEN_TRY => {
                let msg = MsgConnectionOpenTry::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenTry(msg)
            }
            CONNECTION_OPEN_ACK => {
                let msg = MsgConnectionOpenAck::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenAck(msg)
            }
            CONNECTION_OPEN_CONFIRM => {
                let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes)?;
                IbcAction::ConnectionOpenConfirm(msg)
            }
            ACKNOWLEDGEMENT => {
                let msg = MsgAcknowledgement::decode(raw_action_bytes)?;
                IbcAction::Acknowledgement(msg)
            }
            CHANNEL_OPEN_INIT => {
                let msg = MsgChannelOpenInit::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenInit(msg)
            }
            CHANNEL_OPEN_TRY => {
                let msg = MsgChannelOpenTry::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenTry(msg)
            }
            CHANNEL_OPEN_ACK => {
                let msg = MsgChannelOpenAck::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenAck(msg)
            }
            CHANNEL_OPEN_CONFIRM => {
                let msg = MsgChannelOpenConfirm::decode(raw_action_bytes)?;
                IbcAction::ChannelOpenConfirm(msg)
            }
            CHANNEL_CLOSE_INIT => {
                let msg = MsgChannelCloseInit::decode(raw_action_bytes)?;
                IbcAction::ChannelCloseInit(msg)
            }
            CHANNEL_CLOSE_CONFIRM => {
                let msg = MsgChannelCloseConfirm::decode(raw_action_bytes)?;
                IbcAction::ChannelCloseConfirm(msg)
            }
            RECV_PACKET => {
                let msg = MsgRecvPacket::decode(raw_action_bytes)?;
                IbcAction::RecvPacket(msg)
            }
            TIMEOUT => {
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
                type_url: CREATE_CLIENT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::UpdateClient(msg) => pbjson_types::Any {
                type_url: UPDATE_CLIENT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ConnectionOpenInit(msg) => pbjson_types::Any {
                type_url: CONNECTION_OPEN_INIT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ConnectionOpenTry(msg) => pbjson_types::Any {
                type_url: CONNECTION_OPEN_TRY.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ConnectionOpenAck(msg) => pbjson_types::Any {
                type_url: CONNECTION_OPEN_ACK.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ConnectionOpenConfirm(msg) => pbjson_types::Any {
                type_url: CONNECTION_OPEN_CONFIRM.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::Acknowledgement(msg) => pbjson_types::Any {
                type_url: ACKNOWLEDGEMENT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ChannelOpenInit(msg) => pbjson_types::Any {
                type_url: CHANNEL_OPEN_INIT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ChannelOpenTry(msg) => pbjson_types::Any {
                type_url: CHANNEL_OPEN_TRY.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ChannelOpenAck(msg) => pbjson_types::Any {
                type_url: CHANNEL_OPEN_ACK.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ChannelOpenConfirm(msg) => pbjson_types::Any {
                type_url: CHANNEL_OPEN_CONFIRM.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ChannelCloseInit(msg) => pbjson_types::Any {
                type_url: CHANNEL_CLOSE_INIT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::ChannelCloseConfirm(msg) => pbjson_types::Any {
                type_url: CHANNEL_CLOSE_CONFIRM.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::RecvPacket(msg) => pbjson_types::Any {
                type_url: RECV_PACKET.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::Timeout(msg) => pbjson_types::Any {
                type_url: TIMEOUT.to_string(),
                value: msg.encode_vec().into(),
            },
            IbcAction::Unknown(raw_action) => raw_action,
        };
        pb::IbcAction {
            raw_action: Some(raw_action),
        }
    }
}
