//! Implementations of custody services responsible for signing transactions.
//!
//! This crate currently focuses on the [`soft_kms`] implementation, a basic
//! software key management system that can perform basic policy-based
//! authorization or blind signing.

mod client;
mod pre_auth;
mod request;

pub mod policy;
pub mod soft_kms;

pub use client::CustodyClient;
pub use pre_auth::PreAuthorization;
pub use request::AuthorizeRequest;
