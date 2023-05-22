//! TODO: Extract the Bech32 prefixes into a module at the root of the crate,
//! and then remove this module entirely.  We don't want to be custom-shaping
//! serde formats, we want to have those match protobuf JSON exactly.

pub mod bech32str;
