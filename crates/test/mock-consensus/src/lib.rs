//! `penumbra-mock-consensus` is a library for testing consensus-driven ABCI applications.
//!
//! # Overview
//!
//! This library provides facilities that can act as a stand-in for consensus engines like
//! [CometBFT][cometbft] or [Tendermint][tendermint] in integration tests.
//!
//! Testing applications using a mock consensus engine has many benefits. For example, this allows
//! integration test cases to run as fast as possible, without needing wait real wall-clock time
//! for blocks to be generated, or for integration test cases to exercise slashing logic related to
//! byzantine misbehavior (_e.g., double-signing_).
//!
//! This library is agnostic with respect to the replicable state transition machine that it
//! is used to test. This means that, while it may be used to write integration tests for the
//! [Penumbra][penumbra] network, it can also be used to test other decentralized applications.
//!
//! See [`TestNode`] for more information about using `penumbra-mock-consensus`.
//!
//! # Alternatives
//!
//! Projects implemented in Go may wish to consider using [CometMock][cometmock].
//! `penumbra-mock-consensus` is primarily oriented towards projects implemented in Rust that wish
//! to use [`cargo test`][cargo-test] or [`cargo test`][cargo-nextest] as a test-runner.
//!
//! [cargo-nextest]: https://nexte.st/
//! [cargo-test]: https://doc.rust-lang.org/cargo/commands/cargo-test.html
//! [cometbft]: https://github.com/cometbft/cometbft
//! [cometmock]: https://github.com/informalsystems/CometMock
//! [penumbra]: https://github.com/penumbra-zone/penumbra
//! [tendermint]: https://github.com/tendermint/tendermint

use {
    ed25519_consensus::{SigningKey, VerificationKey},
    std::collections::BTreeMap,
};

pub mod block;
pub mod builder;

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
    keyring: Keyring,
}

/// An ordered map of consensus keys.
///
/// Entries in this keyring consist of a [`VerificationKey`] and a [`SigningKey`].
type Keyring = BTreeMap<VerificationKey, SigningKey>;

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

    /// Returns a reference to the test node's set of consensus keys.
    pub fn keyring(&self) -> &Keyring {
        &self.keyring
    }

    /// Returns a mutable reference to the test node's set of consensus keys.
    pub fn keyring_mut(&mut self) -> &mut Keyring {
        &mut self.keyring
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
    pub async fn fast_forward(&mut self, blocks: u64) -> anyhow::Result<()> {
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
