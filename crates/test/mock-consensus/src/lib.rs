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
//! to use [`cargo test`][cargo-test] or [`cargo nextest`][cargo-nextest] as a test-runner.
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
    tendermint::{
        block::{Commit, Height},
        Time,
    },
};

pub mod block;
pub mod builder;

mod abci;

/// A test node.
///
/// A [`TestNode<C>`] represents a validator node containing an instance of the state transition
/// machine and its accompanying consensus engine.
///
/// # Initialization
///
/// Construct a new test node by calling [`TestNode::builder()`]. The [`builder::Builder`]
/// returned by that method can be used to set the initial application state, and configure
/// validators' consensus keys that should be present at genesis. Use
/// [`builder::Builder::init_chain()`] to consume the builder and initialize the application.
///
/// # Consensus Service
///
/// A test node is generic in terms of a consensus service `C`. This service should implement
/// [`tower::Service`], accepting [`ConsensusRequest`][consensus-request]s, and returning
/// [`ConsensusResponse`][consensus-response]s.
///
/// For [`tower-abci`][tower-abci] users, this should correspond with the `C` parameter of the
/// [`Server`][tower-abci-server] type.
///
/// # Blocks
///
/// Blocks can be executed by using [`TestNode::block()`]. This can be used to add transactions,
/// signatures, and evidence to a [`Block`][tendermint-rs-block], before invoking
/// [`block::Builder::execute()`] to execute the next block.
///
/// [consensus-request]: tendermint::v0_37::abci::ConsensusRequest
/// [consensus-response]: tendermint::v0_37::abci::ConsensusResponse
/// [tendermint-rs-block]: tendermint::block::Block
/// [tower-abci-server]: https://docs.rs/tower-abci/latest/tower_abci/v037/struct.Server.html#
/// [tower-abci]: https://docs.rs/tower-abci/latest/tower_abci
pub struct TestNode<C> {
    /// The inner consensus service being tested.
    consensus: C,
    /// The last `app_hash` value.
    last_app_hash: Vec<u8>,
    /// The last validator set hash value.
    last_validator_set_hash: Option<tendermint::Hash>,
    /// The last tendermint block header commit value.
    last_commit: Option<tendermint::block::Commit>,
    /// The consensus params hash.
    consensus_params_hash: Vec<u8>,
    /// The current block [`Height`][tendermint::block::Height].
    height: tendermint::block::Height,
    /// Validators' consensus keys.
    ///
    /// Entries in this keyring consist of a [`VerificationKey`] and a [`SigningKey`].
    keyring: Keyring,
    /// A callback that will be invoked when a new block is constructed.
    on_block: Option<OnBlockFn>,
    /// A callback that will be invoked when a new block is committed, to produce the next timestamp.
    ts_callback: TsCallbackFn,
    /// The current timestamp of the node.
    timestamp: Time,
    /// The chain ID.
    chain_id: tendermint::chain::Id,
}

/// A type alias for the `TestNode::on_block` callback.
pub type OnBlockFn = Box<dyn FnMut(tendermint::Block) + Send + Sync + 'static>;

/// A type alias for the `TestNode::ts_callback` callback.
pub type TsCallbackFn = Box<dyn Fn(Time) -> Time + Send + Sync + 'static>;

/// An ordered map of consensus keys.
///
/// Entries in this keyring consist of a [`VerificationKey`] and a [`SigningKey`].
type Keyring = BTreeMap<VerificationKey, SigningKey>;

/// Accessors.
impl<C> TestNode<C> {
    /// A chain ID for use in tests.
    pub const CHAIN_ID: &'static str = "penumbra-test-chain";

    /// Returns the last `app_hash` value, represented as a slice of bytes.
    pub fn last_app_hash(&self) -> &[u8] {
        &self.last_app_hash
    }

    /// Returns the last `commit` value.
    pub fn last_commit(&self) -> Option<&Commit> {
        self.last_commit.as_ref()
    }

    /// Returns the last `validator_set_hash` value.
    pub fn last_validator_set_hash(&self) -> Option<&tendermint::Hash> {
        self.last_validator_set_hash.as_ref()
    }

    /// Returns the most recent `timestamp` value.
    pub fn timestamp(&self) -> &Time {
        &self.timestamp
    }

    /// Returns the last `app_hash` value, represented as a hexadecimal string.
    pub fn last_app_hash_hex(&self) -> String {
        // Use upper-case hexadecimal integers, include leading zeroes.
        // - https://doc.rust-lang.org/std/fmt/#formatting-traits
        self.last_app_hash
            .iter()
            .map(|b| format!("{:02X}", b).to_string())
            .collect::<Vec<String>>()
            .join("")
    }

    /// Returns a reference to the test node's set of consensus keys.
    pub fn keyring(&self) -> &Keyring {
        &self.keyring
    }

    /// Returns a mutable reference to the test node's set of consensus keys.
    pub fn keyring_mut(&mut self) -> &mut Keyring {
        &mut self.keyring
    }

    pub fn height(&self) -> &Height {
        &self.height
    }
}

/// Fast forward interfaces.
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
    /// Fast forwards the given number of blocks.
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

/// Assert that a [`TestNode`] is both [`Send`] and [`Sync`].
#[allow(dead_code)]
mod assert_address_is_send_and_sync {
    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}
    fn f() {
        is_send::<super::TestNode<()>>();
        is_sync::<super::TestNode<()>>();
    }
}
