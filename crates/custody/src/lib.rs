//! Implementations of custody services responsible for signing transactions.
//!
//! This crate currently focuses on the [`soft_kms`] implementation, a basic
//! software key management system that can perform basic policy-based
//! authorization or blind signing.

#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[macro_use]
extern crate serde_with;

mod client;
mod pre_auth;
mod request;
mod terminal;

pub mod encrypted;
pub mod null_kms;
pub mod policy;
pub mod soft_kms;
pub mod threshold;

pub use client::CustodyClient;
pub use pre_auth::PreAuthorization;
pub use request::{
    AuthorizeRequest, AuthorizeValidatorDefinitionRequest, AuthorizeValidatorVoteRequest,
};
