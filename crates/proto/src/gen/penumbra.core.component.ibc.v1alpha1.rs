#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IbcRelay {
    #[prost(message, optional, tag = "1")]
    pub raw_action: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for IbcRelay {
    const NAME: &'static str = "IbcRelay";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
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
    /// the return address
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
    /// the recipient address on the destination chain
    #[prost(string, tag = "4")]
    pub receiver: ::prost::alloc::string::String,
}
impl ::prost::Name for FungibleTokenPacketData {
    const NAME: &'static str = "FungibleTokenPacketData";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
/// A Penumbra transaction action requesting an ICS20 transfer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ics20Withdrawal {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    #[prost(message, optional, tag = "2")]
    pub denom: ::core::option::Option<super::super::super::asset::v1alpha1::Denom>,
    /// the address on the destination chain to send the transfer to
    #[prost(string, tag = "3")]
    pub destination_chain_address: ::prost::alloc::string::String,
    /// a "sender" penumbra address to use to return funds from this withdrawal.
    /// this should be an ephemeral address
    #[prost(message, optional, tag = "4")]
    pub return_address: ::core::option::Option<
        super::super::super::keys::v1alpha1::Address,
    >,
    /// The height on the counterparty chain at which this transfer expires, and
    /// funds are sent back to the return address.
    #[prost(message, optional, tag = "5")]
    pub timeout_height: ::core::option::Option<
        ::ibc_proto::ibc::core::client::v1::Height,
    >,
    /// the timestamp at which this transfer expires.
    #[prost(uint64, tag = "6")]
    pub timeout_time: u64,
    /// the source channel used for the withdrawal
    #[prost(string, tag = "7")]
    pub source_channel: ::prost::alloc::string::String,
}
impl ::prost::Name for Ics20Withdrawal {
    const NAME: &'static str = "Ics20Withdrawal";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
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
impl ::prost::Name for ClientData {
    const NAME: &'static str = "ClientData";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientCounter {
    #[prost(uint64, tag = "1")]
    pub counter: u64,
}
impl ::prost::Name for ClientCounter {
    const NAME: &'static str = "ClientCounter";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifiedHeights {
    #[prost(message, repeated, tag = "1")]
    pub heights: ::prost::alloc::vec::Vec<::ibc_proto::ibc::core::client::v1::Height>,
}
impl ::prost::Name for VerifiedHeights {
    const NAME: &'static str = "VerifiedHeights";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionCounter {
    #[prost(uint64, tag = "1")]
    pub counter: u64,
}
impl ::prost::Name for ConnectionCounter {
    const NAME: &'static str = "ConnectionCounter";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientConnections {
    #[prost(string, repeated, tag = "1")]
    pub connections: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ClientConnections {
    const NAME: &'static str = "ClientConnections";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
/// IBC configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IbcParameters {
    /// Whether IBC (forming connections, processing IBC packets) is enabled.
    #[prost(bool, tag = "1")]
    pub ibc_enabled: bool,
    /// Whether inbound ICS-20 transfers are enabled
    #[prost(bool, tag = "2")]
    pub inbound_ics20_transfers_enabled: bool,
    /// Whether outbound ICS-20 transfers are enabled
    #[prost(bool, tag = "3")]
    pub outbound_ics20_transfers_enabled: bool,
}
impl ::prost::Name for IbcParameters {
    const NAME: &'static str = "IbcParameters";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
/// IBC genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// IBC parameters.
    #[prost(message, optional, tag = "1")]
    pub ibc_params: ::core::option::Option<IbcParameters>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.ibc.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.ibc.v1alpha1.{}", Self::NAME)
    }
}
