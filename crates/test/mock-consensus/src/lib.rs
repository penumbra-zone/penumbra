//! `penumbra-mock-consensus` is a library for testing consensus-driven applications.
//!
//! See [`TestNode`] for more information.
//
//  see penumbra-zone/penumbra#3588.

pub mod block;
pub mod builder;
pub mod keyring;

mod abci;

/// A test node.
///
/// Construct a new test node by calling [`TestNode::builder()`]. Use [`TestNode::block()`] to
/// build a new [`Block`].
///
/// This contains a consensus service `C`, which should be a [`tower::Service`] implementor that
/// accepts [`ConsensusRequest`][0_37::abci::ConsensusRequest]s, and returns
/// [`ConsensusResponse`][0_37::abci::ConsensusResponse]s. For `tower-abci` users, this should
/// correspond with the `ConsensusService` parameter of the `Server` type.
pub struct TestNode<C> {
    consensus: C,
    last_app_hash: Vec<u8>,
    height: tendermint::block::Height,
}

impl<C> TestNode<C> {
    pub const CHAIN_ID: &'static str = "penumbra-test-chain";

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

impl<C> TestNode<C>
where
    C: tower::Service<
            tendermint::v0_37::abci::ConsensusRequest,
            Response = tendermint::v0_37::abci::ConsensusResponse,
            Error = tower::BoxError,
        > + Send
        + Clone
        + 'static,
    C::Future: Send + 'static,
    C::Error: Sized,
{
    /// Fast forwards a number of blocks.
    #[tracing::instrument(
        skip(self),
        fields(fast_forward.blocks = %blocks)
    )]
    pub async fn fast_forward(&mut self, blocks: usize) -> anyhow::Result<()> {
        use {
            tap::Tap,
            tracing::{info, trace, trace_span, Instrument},
        };

        for i in 1..=blocks {
            self.block()
                .execute()
                .tap(|_| trace!(%i, "executing empty block"))
                .instrument(trace_span!("executing empty block", %i))
                .await
                .tap(|_| trace!(%i, "finished executing empty block"))?;
        }

        Ok(()).tap(|_| info!("finished fast forward"))
    }
}
