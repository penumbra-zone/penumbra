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

// The autogen code is not clippy-clean, so we disable some clippy warnings for this crate.
#![allow(clippy::derive_partial_eq_without_eq)]

pub use prost::Message;

/// Helper methods used for shaping the JSON (and other Serde) formats derived from the protos.
pub mod serializers;

mod protobuf;
pub use protobuf::Protobuf;

#[cfg(feature = "penumbra-storage")]
mod state;
#[cfg(feature = "penumbra-storage")]
pub use state::StateReadProto;
#[cfg(feature = "penumbra-storage")]
pub use state::StateWriteProto;

/// Core protocol structures.
pub mod core {
    /// Crypto structures.
    pub mod crypto {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.crypto.v1alpha1.rs");
        }
    }

    /// Staking structures.
    pub mod stake {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.stake.v1alpha1.rs");
        }
    }

    /// Decentralized exchange structures.
    pub mod dex {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.dex.v1alpha1.rs");
        }
    }

    /// Governance structures.
    pub mod governance {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.governance.v1alpha1.rs");
        }
    }

    /// Transaction structures.
    pub mod transaction {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.transaction.v1alpha1.rs");
        }
    }

    /// Chain-related structures.
    pub mod chain {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.chain.v1alpha1.rs");
        }
    }

    /// IBC protocol structures.
    pub mod ibc {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.ibc.v1alpha1.rs");
        }
    }

    /// Transparent proofs.
    ///
    /// Note that these are protos for the "MVP" transparent version of Penumbra,
    /// i.e. not for production use and intentionally not private.
    pub mod transparent_proofs {
        pub mod v1alpha1 {
            include!("gen/penumbra.core.transparent_proofs.v1alpha1.rs");
        }
    }
}

/// Client protocol structures.
pub mod client {
    pub mod v1alpha1 {
        include!("gen/penumbra.client.v1alpha1.rs");

        use async_stream::try_stream;
        use futures::Stream;
        use futures::StreamExt;
        use specific_query_service_client::SpecificQueryServiceClient;
        use std::pin::Pin;
        use tonic::{
            body::BoxBody,
            codegen::{Body, StdError},
        };

        // Convenience methods for fetching data...

        impl<C> SpecificQueryServiceClient<C> {
            /// Get the Rust protobuf type corresponding to a state key.
            ///
            /// Prefer `key_domain` when applicable, because this gets the validated domain type,
            /// rather than just the raw translation of the protobuf.
            pub async fn key_proto<P>(&mut self, key: impl AsRef<str>) -> anyhow::Result<P>
            where
                P: prost::Message + Default + From<P>,
                C: tonic::client::GrpcService<BoxBody> + 'static,
                C::ResponseBody: Send,
                <C as tonic::client::GrpcService<BoxBody>>::ResponseBody:
                    tonic::codegen::Body<Data = bytes::Bytes>,
                <C::ResponseBody as Body>::Error: Into<StdError> + Send,
            {
                let request = KeyValueRequest {
                    key: key.as_ref().to_string(),
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
                <C as tonic::client::GrpcService<BoxBody>>::ResponseBody:
                    tonic::codegen::Body<Data = bytes::Bytes>,
                <C::ResponseBody as Body>::Error: Into<StdError> + Send,
            {
                let request = KeyValueRequest {
                    key: key.as_ref().to_string(),
                    ..Default::default()
                };

                let t = T::decode(self.key_value(request).await?.into_inner().value.as_slice())?;

                Ok(t)
            }

            /// Get the typed domain value corresponding to prefixes of a state key.
            pub async fn prefix_domain<T, P>(
                &mut self,
                prefix: impl AsRef<str>,
            ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<T>> + Send + 'static>>>
            where
                T: crate::Protobuf<P> + TryFrom<P> + Send + Sync + 'static + Unpin,
                T::Error: Into<anyhow::Error> + Send + Sync + 'static,
                P: prost::Message + Default + From<T>,
                C: tonic::client::GrpcService<BoxBody> + 'static,
                C::ResponseBody: Send,
                <C as tonic::client::GrpcService<BoxBody>>::ResponseBody:
                    tonic::codegen::Body<Data = bytes::Bytes>,
                <C::ResponseBody as Body>::Error: Into<StdError> + Send,
            {
                let request = PrefixValueRequest {
                    prefix: prefix.as_ref().to_string(),
                    ..Default::default()
                };

                let mut stream = self.prefix_value(request).await?.into_inner();
                let out_stream = try_stream! {
                    while let Some(pv_rsp) = stream.message().await? {
                        let t = T::decode(pv_rsp.value.as_slice())?;
                        yield t;
                    }
                };

                Ok(out_stream.boxed())
            }
        }
    }
}

/// View protocol structures.
pub mod view {
    pub mod v1alpha1 {
        include!("gen/penumbra.view.v1alpha1.rs");
    }
}

/// Custody protocol structures.
pub mod custody {
    pub mod v1alpha1 {
        include!("gen/penumbra.custody.v1alpha1.rs");
    }
}
