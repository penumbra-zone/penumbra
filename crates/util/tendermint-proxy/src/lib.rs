#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod tendermint_proxy;
pub use tendermint_proxy::TendermintProxy;
