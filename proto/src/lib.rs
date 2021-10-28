//! Protobuf definitions for Penumbra.
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

/// Transaction structures.
pub mod transaction {
    include!(concat!(env!("OUT_DIR"), "/penumbra.transaction.rs"));
}

/// Transparent proofs.
///
/// Note that these are protos for the "MVP" transparent version of Penumbra,
/// i.e. not for production use and intentionally not private.
pub mod transparent_proofs {
    include!(concat!(env!("OUT_DIR"), "/penumbra.transparent_proofs.rs"));
}

/// Wallet protocol structures.
pub mod wallet {
    tonic::include_proto!("penumbra.wallet");
}

mod protobuf;
pub use prost::Message;
pub use protobuf::Protobuf;
