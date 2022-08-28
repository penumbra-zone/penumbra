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

/// Decentralized exchange structures.
pub mod dex {
    include!(concat!(env!("OUT_DIR"), "/penumbra.dex.rs"));
}

/// Governance structures.
pub mod governance {
    include!(concat!(env!("OUT_DIR"), "/penumbra.governance.rs"));
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

        use specific_query_client::SpecificQueryClient;
        use tonic::{
            body::BoxBody,
            codegen::{Body, StdError},
        };

        // Convenience methods for fetching data...

        impl<C> SpecificQueryClient<C> {
            /// Get the Rust protobuf type corresponding to a state key.
            ///
            /// Prefer `key_domain` when applicable, because this gets the validated domain type,
            /// rather than just the raw translation of the protobuf.
            pub async fn key_proto<P>(&mut self, key: impl AsRef<str>) -> anyhow::Result<P>
            where
                P: prost::Message + Default + From<P>,
                C: tonic::client::GrpcService<BoxBody> + 'static,
                C::ResponseBody: Send,
                <C::ResponseBody as Body>::Error: Into<StdError> + Send,
            {
                let request = KeyValueRequest {
                    key: key.as_ref().as_bytes().to_vec(),
                    ..Default::default()
                };

                let t = P::decode(self.key_value(request).await?.into_inner().value.as_slice())?;

                Ok(t)
            }

            /// Get the typed domain value corresponding to a state key.
            pub async fn key_domain<T, P>(&mut self, key: impl AsRef<str>) -> anyhow::Result<T>
            where
                T: crate::Protobuf<P> + TryFrom<P>,
                T::Error: Into<anyhow::Error> + Send + Sync + 'static,
                P: prost::Message + Default + From<T>,
                C: tonic::client::GrpcService<BoxBody> + 'static,
                C::ResponseBody: Send,
                <C::ResponseBody as Body>::Error: Into<StdError> + Send,
            {
                let request = KeyValueRequest {
                    key: key.as_ref().as_bytes().to_vec(),
                    ..Default::default()
                };

                let t = T::decode(self.key_value(request).await?.into_inner().value.as_slice())?;

                Ok(t)
            }
        }
    }
}

/// IBC protocol structures.
pub mod ibc {
    tonic::include_proto!("penumbra.ibc");
}

/// View protocol structures.
pub mod view {
    tonic::include_proto!("penumbra.view");
}

/// Custody protocol structures.
pub mod custody {
    tonic::include_proto!("penumbra.custody");
}

/// Transparent proofs.
///
/// Note that these are protos for the "MVP" transparent version of Penumbra,
/// i.e. not for production use and intentionally not private.
pub mod transparent_proofs {
    include!(concat!(env!("OUT_DIR"), "/penumbra.transparent_proofs.rs"));
}
