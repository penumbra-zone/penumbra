#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! Facilities for proxying gRPC requests to an upstream Tendermint/CometBFT RPC.
//!
//! Most importantly, this crate provides [`TendermintProxy`], which implements Penumbra's
//! [`tendermint_proxy`][proxy-proto] RPC.
//!
//! [proxy-proto]: https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.util.tendermint_proxy.v1

mod tendermint_proxy;

/// Implements service traits for Tonic gRPC services.
///
/// The fields of this struct are the configuration and data
/// necessary to the gRPC services.
#[derive(Clone, Debug)]
pub struct TendermintProxy {
    /// Address of upstream Tendermint server to proxy requests to.
    tendermint_url: url::Url,
}

impl TendermintProxy {
    /// Returns a new [`TendermintProxy`].
    pub fn new(tendermint_url: url::Url) -> Self {
        Self { tendermint_url }
    }
}
