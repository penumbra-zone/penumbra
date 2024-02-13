//! `penumbra-mock-consensus` is a library for testing consensus-driven applications.
//
//  see penumbra-zone/penumbra#3588.

mod block;
pub mod builder;

// TODO(kate): this is a temporary allowance while we set the test node up.
#[allow(dead_code)]
pub struct TestNode<C> {
    consensus: C,
    last_app_hash: Vec<u8>,
}

impl<C> TestNode<C> {
    /// Returns the last app_hash value, as a slice of bytes.
    pub fn last_app_hash(&self) -> &[u8] {
        &self.last_app_hash
    }

    /// Returns the last app_hash value, as a hexadecimal string.
    pub fn last_app_hash_hex(&self) -> String {
        // Use upper-case hexadecimal integers, include leading zeroes.
        // - https://doc.rust-lang.org/std/fmt/#formatting-traits
        format!("{:02X?}", self.last_app_hash)
    }
}
