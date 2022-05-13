//! Protobuf definitions for Penumbra.o
//!
//! This crate only contains the `.proto` files and the Rust types generated
//! from them.  These types only handle parsing the wire format; validation
//! should be performed by converting them into an appropriate domain type, as
//! in the following diagram:
//!
//! ```ascii
//! ┌───────┐          ┌──────────────┐               ┌──────────────┐
//! │encoded│ protobuf │penumbra_proto│ TryFrom/Into  │ domain types │
//! │ bytes │<──wire ─>│    types     │<─validation ─>│(other crates)│
//! └───────┘  format  └──────────────┘   boundary    └──────────────┘
//! ```
//!
//! The [`Protobuf`] marker trait can be implemented on a domain type to ensure
//! these conversions exist.

pub use prost::Message;

/// Helper methods used for shaping the JSON (and other Serde) formats derived from the protos.
pub mod serializers;

mod protobuf;
pub use protobuf::Protobuf;

/// Crypto structures.
pub mod crypto {
    include!(concat!(env!("OUT_DIR"), "/penumbra.crypto.rs"));
}

/// Staking structures.
pub mod stake {
    include!(concat!(env!("OUT_DIR"), "/penumbra.stake.rs"));
}

/// Transaction structures.
pub mod transaction {
    include!(concat!(env!("OUT_DIR"), "/penumbra.transaction.rs"));
}

/// Chain-related structures.
pub mod chain {
    tonic::include_proto!("penumbra.chain");
}

/// Client protocol structures.
pub mod client {
    pub mod oblivious {
        tonic::include_proto!("penumbra.client.oblivious");
    }
    pub mod specific {
        tonic::include_proto!("penumbra.client.specific");
    }
}

/// IBC protocol structures.
pub mod ibc {
    tonic::include_proto!("penumbra.ibc");

    extern crate ibc as ibc_rs;

    use ibc_rs::core::ics02_client::client_state::AnyClientState;

    use crate::Protobuf;
    use ibc_proto::google::protobuf::Any;
    use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
    use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
    use ibc_rs::core::ics02_client::client_consensus::AnyConsensusState;
    use ibc_rs::core::ics03_connection::connection::ConnectionEnd;
    use ibc_rs::core::ics04_channel::channel::ChannelEnd;

    impl Protobuf<RawConnectionEnd> for ConnectionEnd {}
    impl Protobuf<Any> for AnyClientState {}
    impl Protobuf<Any> for AnyConsensusState {}
    impl Protobuf<RawChannel> for ChannelEnd {}
}

/// Wallet protocol structures.
pub mod wallet {
    tonic::include_proto!("penumbra.wallet");
}

/// Transparent proofs.
///
/// Note that these are protos for the "MVP" transparent version of Penumbra,
/// i.e. not for production use and intentionally not private.
pub mod transparent_proofs {
    include!(concat!(env!("OUT_DIR"), "/penumbra.transparent_proofs.rs"));
}
