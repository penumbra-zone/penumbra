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

/// Genesis-related structures.
pub mod genesis {
    tonic::include_proto!("penumbra.genesis");
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
}

pub mod sighash {
    include!(concat!(env!("OUT_DIR"), "/penumbra.sighash.rs"));

    use sig_hash_action::Action as SHAction;

    use super::transaction::{action::Action as TxAction, Spend};

    impl From<super::transaction::Action> for SigHashAction {
        fn from(action: super::transaction::Action) -> Self {
            let action = match action.action {
                // Pass through other actions
                Some(TxAction::Output(o)) => Some(SHAction::Output(o)),
                Some(TxAction::Delegate(d)) => Some(SHAction::Delegate(d)),
                Some(TxAction::Undelegate(d)) => Some(SHAction::Undelegate(d)),
                // The `ValidatorDefinition` contains sig bytes, but they're across the validator itself,
                // not the transaction, therefore it's fine to include them in the sighash.
                Some(TxAction::ValidatorDefinition(vd)) => Some(SHAction::ValidatorDefinition(vd)),
                // Collapse spends to spend bodies
                Some(TxAction::Spend(Spend { body: None, .. })) => None,
                Some(TxAction::Spend(Spend {
                    body: Some(spend_body),
                    ..
                })) => Some(SHAction::Spend(spend_body)),
                Some(TxAction::IbcAction(i)) => Some(SHAction::IbcAction(i)),
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
