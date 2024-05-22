#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! Facilities for proxying gRPC requests to an upstream Tendermint/CometBFT RPC.
//!
//! Most importantly, this crate provides [`TendermintProxy`], which implements Penumbra's
//! [`tendermint_proxy`][proxy-proto] RPC.
//!
//! [proxy-proto]: https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.util.tendermint_proxy.v1

mod tendermint_proxy;
pub use tendermint_proxy::TendermintProxy;
