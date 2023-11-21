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
//! The [`DomainType`] marker trait can be implemented on a domain type to ensure
//! these conversions exist.

// The autogen code is not clippy-clean, so we disable some clippy warnings for this crate.
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::needless_borrow)]
#![allow(non_snake_case)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use prost::{Message, Name};

/// Helper methods used for shaping the JSON (and other Serde) formats derived from the protos.
pub mod serializers;

mod protobuf;
pub use protobuf::DomainType;
pub use protobuf::TypeUrl;

pub mod type_names;

#[cfg(feature = "penumbra-storage")]
pub mod state;
#[cfg(feature = "penumbra-storage")]
pub use state::StateReadProto;
#[cfg(feature = "penumbra-storage")]
pub use state::StateWriteProto;

pub use penumbra::*;

pub mod penumbra {
    /*
    /// Client protocol structures.
    pub mod client {
        pub mod v1alpha1 {
            include!("gen/penumbra.client.v1alpha1.rs");
            impl From<Vec<u8>> for key_value_response::Value {
                fn from(v: Vec<u8>) -> Self {
                    key_value_response::Value { value: v }
                }
            }

            include!("gen/penumbra.client.v1alpha1.serde.rs");

            // TODO(hdevalence): do we want any of this code?

            use async_stream::try_stream;
            use futures::Stream;
            use futures::StreamExt;
            use std::pin::Pin;
            #[cfg(feature = "rpc")]
            use tonic::{
                body::BoxBody,
                codegen::{Body, StdError},
            };

            // Convenience methods for fetching data...
            #[cfg(feature = "rpc")]
            use specific_query_service_client::SpecificQueryServiceClient;

            #[cfg(feature = "rpc")]
            impl<C> SpecificQueryServiceClient<C> {
                /// Get the Rust protobuf type corresponding to a state key.
                ///
                /// Prefer `key_domain` when applicable, because this gets the validated domain type,
                /// rather than just the raw translation of the protobuf.
                pub async fn key_proto<P>(
                    &mut self,
                    key: impl AsRef<str>,
                ) -> anyhow::Result<Option<P>>
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

                    let response = self.key_value(request).await?.into_inner();

                    let Some(value_buffer) = response.value else {
                        return Ok(None);
                    };

                    let t = P::decode(value_buffer.value.as_slice())?;
                    Ok(Some(t))
                }

                /// Get the typed domain value corresponding to a state key.
                pub async fn key_domain<T>(
                    &mut self,
                    key: impl AsRef<str>,
                ) -> anyhow::Result<Option<T>>
                where
                    T: crate::DomainType,
                    anyhow::Error: From<<T as TryFrom<T::Proto>>::Error>,
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

                    let response = self.key_value(request).await?.into_inner();

                    let Some(value_buffer) = response.value else {
                        return Ok(None);
                    };

                    let t = T::decode(value_buffer.value.as_slice())?;
                    Ok(Some(t))
                }

                /// Get the typed domain value corresponding to prefixes of a state key.
                pub async fn prefix_domain<T>(
                    &mut self,
                    prefix: impl AsRef<str>,
                ) -> anyhow::Result<
                    Pin<Box<dyn Stream<Item = anyhow::Result<(String, T)>> + Send + 'static>>,
                >
                where
                    T: crate::DomainType + Send + 'static + Unpin,
                    anyhow::Error: From<<T as TryFrom<T::Proto>>::Error>,
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
                            yield (pv_rsp.key, t);
                        }
                    };

                    Ok(out_stream.boxed())
                }
            }
        }
    }
    */

    /// Core protocol structures.
    pub mod core {
        /// Top-level structures for the Penumbra application.
        pub mod app {
            pub mod v1alpha1 {
                include!("gen/penumbra.core.app.v1alpha1.rs");
                include!("gen/penumbra.core.app.v1alpha1.serde.rs");
            }
        }

        pub mod asset {
            pub mod v1alpha1 {
                include!("gen/penumbra.core.asset.v1alpha1.rs");
                include!("gen/penumbra.core.asset.v1alpha1.serde.rs");
            }
        }

        /// Components of the Penumbra application.
        pub mod component {

            pub mod chain {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.chain.v1alpha1.rs");
                    include!("gen/penumbra.core.component.chain.v1alpha1.serde.rs");
                }
            }

            pub mod compact_block {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.compact_block.v1alpha1.rs");
                    include!("gen/penumbra.core.component.compact_block.v1alpha1.serde.rs");
                }
            }

            pub mod dao {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.dao.v1alpha1.rs");
                    include!("gen/penumbra.core.component.dao.v1alpha1.serde.rs");
                }
            }

            pub mod dex {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.dex.v1alpha1.rs");
                    include!("gen/penumbra.core.component.dex.v1alpha1.serde.rs");
                }
            }

            pub mod distributions {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.distributions.v1alpha1.rs");
                    include!("gen/penumbra.core.component.distributions.v1alpha1.serde.rs");
                }
            }

            pub mod fee {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.fee.v1alpha1.rs");
                    include!("gen/penumbra.core.component.fee.v1alpha1.serde.rs");
                }
            }

            pub mod governance {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.governance.v1alpha1.rs");
                    include!("gen/penumbra.core.component.governance.v1alpha1.serde.rs");
                }
            }

            pub mod ibc {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.ibc.v1alpha1.rs");
                    include!("gen/penumbra.core.component.ibc.v1alpha1.serde.rs");
                }
            }

            pub mod sct {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.sct.v1alpha1.rs");
                    include!("gen/penumbra.core.component.sct.v1alpha1.serde.rs");
                }
            }

            pub mod shielded_pool {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.shielded_pool.v1alpha1.rs");
                    include!("gen/penumbra.core.component.shielded_pool.v1alpha1.serde.rs");
                }
            }

            pub mod stake {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.stake.v1alpha1.rs");
                    include!("gen/penumbra.core.component.stake.v1alpha1.serde.rs");
                }
            }
        }

        pub mod keys {
            pub mod v1alpha1 {
                include!("gen/penumbra.core.keys.v1alpha1.rs");
                include!("gen/penumbra.core.keys.v1alpha1.serde.rs");
            }
        }

        pub mod num {
            pub mod v1alpha1 {
                include!("gen/penumbra.core.num.v1alpha1.rs");
                include!("gen/penumbra.core.num.v1alpha1.serde.rs");
            }
        }

        /// Transaction structures.
        pub mod transaction {
            pub mod v1alpha1 {
                include!("gen/penumbra.core.transaction.v1alpha1.rs");
                include!("gen/penumbra.core.transaction.v1alpha1.serde.rs");
            }
        }
    }

    /// Cryptography primitives used by Penumbra.
    pub mod crypto {
        pub mod decaf377_fmd {
            pub mod v1alpha1 {
                include!("gen/penumbra.crypto.decaf377_fmd.v1alpha1.rs");
                include!("gen/penumbra.crypto.decaf377_fmd.v1alpha1.serde.rs");
            }
        }

        pub mod decaf377_frost {
            pub mod v1alpha1 {
                include!("gen/penumbra.crypto.decaf377_frost.v1alpha1.rs");
                include!("gen/penumbra.crypto.decaf377_frost.v1alpha1.serde.rs");
            }
        }

        pub mod decaf377_rdsa {
            pub mod v1alpha1 {
                include!("gen/penumbra.crypto.decaf377_rdsa.v1alpha1.rs");
                include!("gen/penumbra.crypto.decaf377_rdsa.v1alpha1.serde.rs");
            }
        }

        pub mod tct {
            pub mod v1alpha1 {
                include!("gen/penumbra.crypto.tct.v1alpha1.rs");
                include!("gen/penumbra.crypto.tct.v1alpha1.serde.rs");
            }
        }
    }

    /// Custody protocol structures.
    pub mod custody {
        pub mod v1alpha1 {
            include!("gen/penumbra.custody.v1alpha1.rs");
            include!("gen/penumbra.custody.v1alpha1.serde.rs");
        }
    }

    /// Narsil protocol structures.
    pub mod narsil {
        pub mod v1alpha1 {
            pub mod ledger {
                include!("gen/penumbra.narsil.ledger.v1alpha1.rs");
                include!("gen/penumbra.narsil.ledger.v1alpha1.serde.rs");
            }
        }
    }

    /*
    pub mod storage {
        pub mod v1alpha1 {
            include!("gen/penumbra.storage.v1alpha1.rs");
            include!("gen/penumbra.storage.v1alpha1.serde.rs");
        }
    }
    */

    pub mod util {
        pub mod tendermint_proxy {
            pub mod v1alpha1 {
                include!("gen/penumbra.util.tendermint_proxy.v1alpha1.rs");
                include!("gen/penumbra.util.tendermint_proxy.v1alpha1.serde.rs");
            }
        }
    }

    pub mod tools {
        pub mod summoning {
            pub mod v1alpha1 {
                include!("gen/penumbra.tools.summoning.v1alpha1.rs");
                include!("gen/penumbra.tools.summoning.v1alpha1.serde.rs");
            }
        }
    }

    /// View protocol structures.
    pub mod view {
        pub mod v1alpha1 {
            include!("gen/penumbra.view.v1alpha1.rs");
            include!("gen/penumbra.view.v1alpha1.serde.rs");
        }
    }
}

pub mod tendermint {
    pub mod crypto {
        include!("gen/tendermint.crypto.rs");
    }

    #[allow(clippy::large_enum_variant)]
    pub mod types {
        include!("gen/tendermint.types.rs");
    }

    pub mod version {
        include!("gen/tendermint.version.rs");
    }

    pub mod p2p {
        include!("gen/tendermint.p2p.rs");
    }
}

#[cfg(feature = "rpc")]
// https://github.com/penumbra-zone/penumbra/issues/3038#issuecomment-1722534133
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("gen/proto_descriptor.bin.no_lfs");
