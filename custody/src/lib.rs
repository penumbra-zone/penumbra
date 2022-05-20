//! Implementations of custody services responsible for signing transactions.
//!
//! Currently, this just has a stub software implementation that signs any
//! transaction it sees, but in the future this interface could allow
//! programmable policy (inspecting transaction plans), custom custody flows
//! (HSMs, hardware wallets with humans-in-the-loop, threshold signer clusters,
//! offline threshold signing, ...).

mod request;
mod soft_hsm;

pub use request::AuthorizeRequest;
pub use soft_hsm::SoftHSM;

/// Re-exports of protobuf messages.
pub mod proto {
    pub use penumbra_proto::{custody::AuthorizeRequest, transaction::AuthorizationData};
}
pub use penumbra_proto::custody::{
    custody_protocol_client::CustodyProtocolClient,
    custody_protocol_server::{CustodyProtocol, CustodyProtocolServer},
};
