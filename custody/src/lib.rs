//! Implementations of custody services responsible for signing transactions.
//!
//! Currently, this just has a stub software implementation that signs any
//! transaction it sees, but in the future this interface could allow
//! programmable policy (inspecting transaction plans), custom custody flows
//! (HSMs, hardware wallets with humans-in-the-loop, threshold signer clusters,
//! offline threshold signing, ...).

mod client;
mod request;
mod soft_kms;

pub use client::CustodyClient;
pub use request::AuthorizeRequest;
pub use soft_kms::SoftKms;
