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

pub mod sighash {
    include!(concat!(env!("OUT_DIR"), "/penumbra.sighash.rs"));

    use super::transaction::action::Action as TxAction;
    use super::transaction::Spend;
    use sig_hash_action::Action as SHAction;

    impl From<super::transaction::Action> for SigHashAction {
        fn from(action: super::transaction::Action) -> Self {
            let action = match action.action {
                // Pass through outputs
                Some(TxAction::Output(o)) => Some(SHAction::Output(o)),
                Some(TxAction::Spend(Spend { body: None, .. })) => None,
                // Collapse spends to spend bodies
                Some(TxAction::Spend(Spend {
                    body: Some(spend_body),
                    ..
                })) => Some(SHAction::Spend(spend_body)),
                None => None,
            };
            Self { action }
        }
    }

    impl From<super::transaction::TransactionBody> for SigHashTransaction {
        fn from(body: super::transaction::TransactionBody) -> Self {
            Self {
                actions: body.actions.into_iter().map(Into::into).collect(),
                anchor: body.anchor.to_vec(),
                expiry_height: body.expiry_height,
                chain_id: body.chain_id,
                fee: body.fee,
            }
        }
    }
}

/// Transparent proofs.
///
/// Note that these are protos for the "MVP" transparent version of Penumbra,
/// i.e. not for production use and intentionally not private.
pub mod transparent_proofs {
    include!(concat!(env!("OUT_DIR"), "/penumbra.transparent_proofs.rs"));
}

/// Light wallet protocol structures.
pub mod light_wallet {
    tonic::include_proto!("penumbra.light_wallet");
}

/// Thin wallet protocol structures.
pub mod thin_wallet {
    tonic::include_proto!("penumbra.thin_wallet");
}

mod protobuf;
pub use prost::Message;
pub use protobuf::Protobuf;
