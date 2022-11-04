#[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Message)]
pub struct IbcAction {
    #[prost(
        oneof = "ibc_action::Action",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17"
    )]
    pub action: ::core::option::Option<ibc_action::Action>,
}
/// Nested message and enum types in `IBCAction`.
pub mod ibc_action {
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag = "1")]
        ConnectionOpenInit(::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit),
        #[prost(message, tag = "2")]
        ConnectionOpenTry(::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry),
        #[prost(message, tag = "3")]
        ConnectionOpenAck(::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck),
        #[prost(message, tag = "4")]
        ConnectionOpenConfirm(::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm),
        #[prost(message, tag = "5")]
        ChannelOpenInit(::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit),
        #[prost(message, tag = "6")]
        ChannelOpenTry(::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry),
        #[prost(message, tag = "7")]
        ChannelOpenAck(::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck),
        #[prost(message, tag = "8")]
        ChannelOpenConfirm(::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm),
        #[prost(message, tag = "9")]
        ChannelCloseInit(::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit),
        #[prost(message, tag = "10")]
        ChannelCloseConfirm(::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm),
        #[prost(message, tag = "11")]
        RecvPacket(::ibc_proto::ibc::core::channel::v1::MsgRecvPacket),
        #[prost(message, tag = "12")]
        Timeout(::ibc_proto::ibc::core::channel::v1::MsgTimeout),
        #[prost(message, tag = "13")]
        Acknowledgement(::ibc_proto::ibc::core::channel::v1::MsgAcknowledgement),
        #[prost(message, tag = "14")]
        CreateClient(::ibc_proto::ibc::core::client::v1::MsgCreateClient),
        #[prost(message, tag = "15")]
        UpdateClient(::ibc_proto::ibc::core::client::v1::MsgUpdateClient),
        #[prost(message, tag = "16")]
        UpgradeClient(::ibc_proto::ibc::core::client::v1::MsgUpgradeClient),
        #[prost(message, tag = "17")]
        SubmitMisbehaviour(::ibc_proto::ibc::core::client::v1::MsgSubmitMisbehaviour),
    }
}
/// FungibleTokenPacketData defines a struct for the packet payload
/// See FungibleTokenPacketData spec:
/// <https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures>
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
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Message)]
pub struct Ics20Withdrawal {
    /// the chain ID of the destination chain for this Ics20 transfer
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientData {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// NOTE: left as Any to allow us to add more client types later
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<::prost_types::Any>,
    #[prost(string, tag = "3")]
    pub processed_time: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub processed_height: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientCounter {
    #[prost(uint64, tag = "1")]
    pub counter: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<::prost_types::Any>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifiedHeights {
    #[prost(message, repeated, tag = "1")]
    pub heights: ::prost::alloc::vec::Vec<::ibc_proto::ibc::core::client::v1::Height>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionCounter {
    #[prost(uint64, tag = "1")]
    pub counter: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientConnections {
    #[prost(string, repeated, tag = "1")]
    pub connections: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
