//! Protobuf definitions for Penumbra.
//!
//! This crate only contains the `.proto` files and the Rust types generated
//! from them.  These types only handle parsing the wire format; validation
//! should be performed by converting them into an appropriate domain type, as
//! in the following diagram:
//!
//! ```ascii
//! ┌───────┐          ┌──────────────┐               ┌──────────────┐
//! │encoded│ protobuf │penumbra_sdk_proto│ TryFrom/Into  │ domain types │
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
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use prost::{Message, Name};

/// Helper methods used for shaping the JSON (and other Serde) formats derived from the protos.
pub mod serializers;

#[cfg(feature = "box-grpc")]
pub mod box_grpc_svc;

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
            pub mod v1 {
                include!("gen/penumbra.core.app.v1.rs");
                include!("gen/penumbra.core.app.v1.serde.rs");
            }
        }

        pub mod asset {
            pub mod v1 {
                include!("gen/penumbra.core.asset.v1.rs");
                include!("gen/penumbra.core.asset.v1.serde.rs");
            }
        }

        pub mod txhash {
            pub mod v1 {
                include!("gen/penumbra.core.txhash.v1.rs");
                include!("gen/penumbra.core.txhash.v1.serde.rs");
            }
        }

        /// Components of the Penumbra application.
        pub mod component {
            pub mod auction {
                pub mod v1 {
                    include!("gen/penumbra.core.component.auction.v1.rs");
                    include!("gen/penumbra.core.component.auction.v1.serde.rs");
                }
            }
            pub mod compact_block {
                pub mod v1 {
                    include!("gen/penumbra.core.component.compact_block.v1.rs");
                    include!("gen/penumbra.core.component.compact_block.v1.serde.rs");
                }
            }

            pub mod community_pool {
                pub mod v1 {
                    include!("gen/penumbra.core.component.community_pool.v1.rs");
                    include!("gen/penumbra.core.component.community_pool.v1.serde.rs");
                }
            }

            pub mod dex {
                pub mod v1 {
                    include!("gen/penumbra.core.component.dex.v1.rs");
                    include!("gen/penumbra.core.component.dex.v1.serde.rs");
                }
            }

            pub mod distributions {
                pub mod v1 {
                    include!("gen/penumbra.core.component.distributions.v1.rs");
                    include!("gen/penumbra.core.component.distributions.v1.serde.rs");
                }
            }

            pub mod fee {
                pub mod v1 {
                    include!("gen/penumbra.core.component.fee.v1.rs");
                    include!("gen/penumbra.core.component.fee.v1.serde.rs");
                }
            }

            pub mod funding {
                pub mod v1 {
                    include!("gen/penumbra.core.component.funding.v1.rs");
                    include!("gen/penumbra.core.component.funding.v1.serde.rs");
                }
            }

            pub mod governance {
                pub mod v1 {
                    include!("gen/penumbra.core.component.governance.v1.rs");
                    include!("gen/penumbra.core.component.governance.v1.serde.rs");
                }
            }

            pub mod ibc {
                pub mod v1 {
                    include!("gen/penumbra.core.component.ibc.v1.rs");
                    include!("gen/penumbra.core.component.ibc.v1.serde.rs");
                }
            }

            pub mod sct {
                pub mod v1 {
                    include!("gen/penumbra.core.component.sct.v1.rs");
                    include!("gen/penumbra.core.component.sct.v1.serde.rs");
                }
            }

            pub mod shielded_pool {
                pub mod v1 {
                    include!("gen/penumbra.core.component.shielded_pool.v1.rs");
                    include!("gen/penumbra.core.component.shielded_pool.v1.serde.rs");
                }
            }

            pub mod stake {
                pub mod v1 {
                    include!("gen/penumbra.core.component.stake.v1.rs");
                    include!("gen/penumbra.core.component.stake.v1.serde.rs");
                }
            }
        }

        pub mod keys {
            pub mod v1 {
                include!("gen/penumbra.core.keys.v1.rs");
                include!("gen/penumbra.core.keys.v1.serde.rs");
            }
        }

        pub mod num {
            pub mod v1 {
                include!("gen/penumbra.core.num.v1.rs");
                include!("gen/penumbra.core.num.v1.serde.rs");
            }
        }

        /// Transaction structures.
        pub mod transaction {
            pub mod v1 {
                include!("gen/penumbra.core.transaction.v1.rs");
                include!("gen/penumbra.core.transaction.v1.serde.rs");
            }
        }
    }

    /// Cryptography primitives used by Penumbra.
    pub mod crypto {
        pub mod decaf377_fmd {
            pub mod v1 {
                include!("gen/penumbra.crypto.decaf377_fmd.v1.rs");
                include!("gen/penumbra.crypto.decaf377_fmd.v1.serde.rs");
            }
        }

        pub mod decaf377_frost {
            pub mod v1 {
                include!("gen/penumbra.crypto.decaf377_frost.v1.rs");
                include!("gen/penumbra.crypto.decaf377_frost.v1.serde.rs");
            }
        }

        pub mod decaf377_rdsa {
            pub mod v1 {
                include!("gen/penumbra.crypto.decaf377_rdsa.v1.rs");
                include!("gen/penumbra.crypto.decaf377_rdsa.v1.serde.rs");
            }
        }

        pub mod tct {
            pub mod v1 {
                include!("gen/penumbra.crypto.tct.v1.rs");
                include!("gen/penumbra.crypto.tct.v1.serde.rs");
            }
        }
    }

    /// Custody protocol structures.
    pub mod custody {
        pub mod threshold {
            pub mod v1 {
                include!("gen/penumbra.custody.threshold.v1.rs");
                include!("gen/penumbra.custody.threshold.v1.serde.rs");
            }
        }

        pub mod v1 {
            include!("gen/penumbra.custody.v1.rs");
            include!("gen/penumbra.custody.v1.serde.rs");
        }
    }

    pub mod util {
        pub mod tendermint_proxy {
            pub mod v1 {
                include!("gen/penumbra.util.tendermint_proxy.v1.rs");
                include!("gen/penumbra.util.tendermint_proxy.v1.serde.rs");
            }
        }
    }

    pub mod tools {
        pub mod summoning {
            pub mod v1 {
                include!("gen/penumbra.tools.summoning.v1.rs");
                include!("gen/penumbra.tools.summoning.v1.serde.rs");
            }
        }
    }

    /// View protocol structures.
    pub mod view {
        pub mod v1 {
            include!("gen/penumbra.view.v1.rs");
            include!("gen/penumbra.view.v1.serde.rs");
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

    pub mod abci {
        include!("gen/tendermint.abci.rs");
    }
}

pub mod noble {
    pub mod forwarding {
        pub mod v1 {
            include!("gen/noble.forwarding.v1.rs");
        }
    }
}

pub mod cosmos {
    pub mod base {
        pub mod v1beta1 {
            include!("gen/cosmos.base.v1beta1.rs");
        }

        pub mod query {
            pub mod v1beta1 {
                include!("gen/cosmos.base.query.v1beta1.rs");
            }
        }

        pub mod abci {
            pub mod v1beta1 {
                include!("gen/cosmos.base.abci.v1beta1.rs");
            }
        }
    }

    pub mod auth {
        pub mod v1beta1 {
            include!("gen/cosmos.auth.v1beta1.rs");
        }
    }

    pub mod bank {
        pub mod v1beta1 {
            include!("gen/cosmos.bank.v1beta1.rs");
        }
    }

    pub mod tx {
        pub mod v1beta1 {
            include!("gen/cosmos.tx.v1beta1.rs");
        }

        pub mod config {
            pub mod v1 {
                include!("gen/cosmos.tx.config.v1.rs");
            }
        }

        pub mod signing {
            pub mod v1beta1 {
                include!("gen/cosmos.tx.signing.v1beta1.rs");
            }
        }
    }

    pub mod crypto {
        pub mod multisig {
            pub mod v1beta1 {
                include!("gen/cosmos.crypto.multisig.v1beta1.rs");
            }
        }
    }
}

#[cfg(feature = "rpc")]
// https://github.com/penumbra-zone/penumbra/issues/3038#issuecomment-1722534133
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("gen/proto_descriptor.bin.no_lfs");
