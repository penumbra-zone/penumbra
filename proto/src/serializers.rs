//! Serializers for adjusting the Serde implementations derived from the Rust
//! proto types.
//!
//! This approach is inspired by the tendermint-rs implementation, and some of
//! the serializers are adapted from that code.

pub mod hexstr;
pub mod hexstr_bytes;

pub mod base64str;
pub mod base64str_bytes;

pub mod bech32str;

pub mod prost_any;
pub mod vote;
