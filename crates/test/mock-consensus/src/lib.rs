//! `penumbra-mock-consensus` is a library for testing consensus-driven applications.
//
//  see penumbra-zone/penumbra#3588.

mod block;
mod builder;

// TODO(kate): this is a temporary allowance while we set the test node up.
#[allow(dead_code)]
pub struct TestNode<C> {
    consensus: C,
    last_app_hash: Vec<u8>,
}

impl<C> TestNode<C> {
    pub fn next_block() -> tendermint::Block {
        todo!();
    }
}
