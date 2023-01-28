#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IbcAction {
    ///
    /// oneof action {
    /// .ibc.core.connection.v1.MsgConnectionOpenInit connection_open_init = 1;
    /// .ibc.core.connection.v1.MsgConnectionOpenTry connection_open_try = 2;
    /// .ibc.core.connection.v1.MsgConnectionOpenAck connection_open_ack = 3;
    /// .ibc.core.connection.v1.MsgConnectionOpenConfirm connection_open_confirm = 4;
    ///
    /// .ibc.core.channel.v1.MsgChannelOpenInit channel_open_init = 5;
    /// .ibc.core.channel.v1.MsgChannelOpenTry channel_open_try = 6;
    /// .ibc.core.channel.v1.MsgChannelOpenAck channel_open_ack = 7;
    /// .ibc.core.channel.v1.MsgChannelOpenConfirm channel_open_confirm = 8;
    /// .ibc.core.channel.v1.MsgChannelCloseInit channel_close_init = 9;
    /// .ibc.core.channel.v1.MsgChannelCloseConfirm channel_close_confirm = 10;
    ///
    /// .ibc.core.channel.v1.MsgRecvPacket recv_packet = 11;
    /// .ibc.core.channel.v1.MsgTimeout timeout = 12;
    /// .ibc.core.channel.v1.MsgAcknowledgement acknowledgement = 13;
    ///
    /// .ibc.core.client.v1.MsgCreateClient create_client = 14;
    /// .ibc.core.client.v1.MsgUpdateClient update_client = 15;
    /// .ibc.core.client.v1.MsgUpgradeClient upgrade_client = 16;
    /// .ibc.core.client.v1.MsgSubmitMisbehaviour submit_misbehaviour = 17;
    /// }
    #[prost(message, optional, tag = "1")]
    pub raw_action: ::core::option::Option<::pbjson_types::Any>,
}
/// FungibleTokenPacketData defines a struct for the packet payload
/// See FungibleTokenPacketData spec:
/// <https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FungibleTokenPacketData {
    /// the token denomination to be transferred
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// the token amount to be transferred
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
    /// the sender address
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
    /// the recipient address on the destination chain
    #[prost(string, tag = "4")]
    pub receiver: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ics20Withdrawal {
    /// the chain ID of the destination chain for this ICS20 transfer
    #[prost(string, tag = "1")]
    pub destination_chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub denom: ::core::option::Option<super::super::crypto::v1alpha1::Denom>,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// the address on the destination chain to send the transfer to
    #[prost(string, tag = "4")]
    pub destination_chain_address: ::prost::alloc::string::String,
    /// a "sender" penumbra address to use to return funds from this withdrawal.
    /// this should be an ephemeral address
    #[prost(message, optional, tag = "5")]
    pub return_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// the height (on Penumbra) at which this transfer expires (and funds are sent
    /// back to the sender address?). NOTE: if funds are sent back to the sender,
    /// we MUST verify a nonexistence proof before accepting the timeout, to
    /// prevent relayer censorship attacks. The core IBC implementation does this
    /// in its handling of validation of timeouts.
    #[prost(uint64, tag = "6")]
    pub timeout_height: u64,
    /// the timestamp at which this transfer expires.
    #[prost(uint64, tag = "7")]
    pub timeout_time: u64,
    /// the source port that identifies the channel used for the withdrawal
    #[prost(string, tag = "8")]
    pub source_port: ::prost::alloc::string::String,
    /// the source channel used for the withdrawal
    #[prost(string, tag = "9")]
    pub source_channel: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientData {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// NOTE: left as Any to allow us to add more client types later
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<::pbjson_types::Any>,
    #[prost(string, tag = "3")]
    pub processed_time: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub processed_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientCounter {
    #[prost(uint64, tag = "1")]
    pub counter: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<::pbjson_types::Any>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifiedHeights {
    #[prost(message, repeated, tag = "1")]
    pub heights: ::prost::alloc::vec::Vec<::ibc_proto::ibc::core::client::v1::Height>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionCounter {
    #[prost(uint64, tag = "1")]
    pub counter: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientConnections {
    #[prost(string, repeated, tag = "1")]
    pub connections: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
