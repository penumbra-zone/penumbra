//! `penumbra-mock-consensus` is a library for testing consensus-driven applications.
//
//  see penumbra-zone/penumbra#3588.

mod block;
mod builder;

pub struct TestNode<C> {
    #[allow(dead_code)]
    consensus: C,
    _last_app_hash: Vec<u8>,
}

impl<C> TestNode<C> {
    pub fn next_block() -> tendermint::Block {
        todo!();
    }
}
