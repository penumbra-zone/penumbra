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
//! The [`DomainType`] marker trait can be implemented on a domain type to ensure
//! these conversions exist.

// The autogen code is not clippy-clean, so we disable some clippy warnings for this crate.
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::unwrap_used)]
#![allow(non_snake_case)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use prost::{Message, Name};

/// Helper methods used for shaping the JSON (and other Serde) formats derived from the protos.
pub mod serializers;

/// Helper trait for using Protobuf messages as ABCI events.
pub mod event;
mod protobuf;
pub use protobuf::DomainType;

#[cfg(feature = "cnidarium")]
pub mod state;
#[cfg(feature = "cnidarium")]
pub use state::StateReadProto;
#[cfg(feature = "cnidarium")]
pub use state::StateWriteProto;

pub use penumbra::*;

pub mod penumbra {
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

        pub mod txhash {
            pub mod v1alpha1 {
                include!("gen/penumbra.core.txhash.v1alpha1.rs");
                include!("gen/penumbra.core.txhash.v1alpha1.serde.rs");
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

            pub mod community_pool {
                pub mod v1alpha1 {
                    include!("gen/penumbra.core.component.community_pool.v1alpha1.rs");
                    include!("gen/penumbra.core.component.community_pool.v1alpha1.serde.rs");
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
        pub mod threshold {
            pub mod v1alpha1 {
                include!("gen/penumbra.custody.threshold.v1alpha1.rs");
                include!("gen/penumbra.custody.threshold.v1alpha1.serde.rs");
            }
        }

        pub mod v1alpha1 {
            include!("gen/penumbra.custody.v1alpha1.rs");
            include!("gen/penumbra.custody.v1alpha1.serde.rs");
        }
    }

    pub mod cnidarium {
        pub mod v1alpha1 {
            include!("gen/penumbra.cnidarium.v1alpha1.rs");
            include!("gen/penumbra.cnidarium.v1alpha1.serde.rs");
        }
    }

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
