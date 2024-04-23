//! [`Builder`] facilities for constructing [`Block`]s.
//!
//! Builders are acquired by calling [`TestNode::block()`], see [`TestNode`] for more information.

use {
    crate::TestNode,
    tap::Tap,
    tendermint::{
        account,
        block::{self, header::Version, Block, Commit, Header, Round},
        chain, evidence,
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
        AppHash, Hash,
    },
    tower::{BoxError, Service},
    tracing::{instrument, trace},
};

/// Interfaces for generating commit signatures.
mod signature;

/// A block builder.
///
/// A block builder can be used to prepare and instantiate a new [`Block`]. A block builder is
/// acquired by calling [`TestNode::block()`]. This builder holds an exclusive reference to a
/// [`TestNode`], so only one block may be built at once.
///
/// This builder can be consumed, executing the block against the [`TestNode`]'s consensus service,
/// by calling [`Builder::execute()`].
pub struct Builder<'e, C> {
    /// A unique reference to the test node.
    test_node: &'e mut TestNode<C>,
    /// Transaction data.
    data: Vec<Vec<u8>>,
    /// Evidence of malfeasance.
    evidence: evidence::List,
    /// The list of signatures.
    signatures: Vec<block::CommitSig>,
}

// === impl TestNode ===

impl<C> TestNode<C> {
    /// Returns a new [`Builder`].
    ///
    /// By default, signatures for all of the validators currently within the keyring will be
    /// included in the block. Use [`Builder::with_signatures()`] to set a different set of
    /// validator signatures.
    pub fn block<'e>(&'e mut self) -> Builder<'e, C> {
        let signatures = self.generate_signatures().collect();
        Builder {
            test_node: self,
            data: Default::default(),
            evidence: Default::default(),
            signatures,
        }
    }
}

// === impl Builder ===

impl<'e, C> Builder<'e, C> {
    /// Sets the data for this block.
    pub fn with_data(self, data: Vec<Vec<u8>>) -> Self {
        let Self { data: prev, .. } = self;

        if !prev.is_empty() {
            tracing::warn!(
                count = %prev.len(),
                "block builder overwriting transaction data, this may be a bug!"
            );
        }

        Self { data, ..self }
    }

    /// Appends the given tx to this block's data.
    pub fn add_tx(mut self, tx: Vec<u8>) -> Self {
        self.data.push(tx);
        self
    }

    /// Sets the evidence [`List`][evidence::List] for this block.
    pub fn with_evidence(self, evidence: evidence::List) -> Self {
        Self { evidence, ..self }
    }

    /// Sets the [`CommitSig`][block::CommitSig] commit signatures for this block.
    pub fn with_signatures(self, signatures: Vec<block::CommitSig>) -> Self {
        Self { signatures, ..self }
    }
}

impl<'e, C> Builder<'e, C>
where
    C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError>
        + Send
        + Clone
        + 'static,
    C::Future: Send + 'static,
    C::Error: Sized,
{
    /// Consumes this builder, executing the [`Block`] using the consensus service.
    ///
    /// Use [`TestNode::block()`] to build a new block.
    #[instrument(level = "info", skip_all, fields(height, time))]
    pub async fn execute(self) -> Result<(), anyhow::Error> {
        let (test_node, block) = self.finish()?;

        let Block {
            header,
            data,
            evidence: _,
            last_commit,
            ..
        } = block.tap(|block| {
            tracing::span::Span::current()
                .record("height", block.header.height.value())
                .record("time", block.header.time.unix_timestamp());
        });
        let last_commit_info = Self::last_commit_info(last_commit);

        trace!("sending block");
        test_node.begin_block(header, last_commit_info).await?;
        for tx in data {
            let tx = tx.into();
            test_node.deliver_tx(tx).await?;
        }
        test_node.end_block().await?;
        test_node.commit().await?;
        trace!("finished sending block");

        Ok(())
    }

    /// Consumes this builder, returning its [`TestNode`] reference and a [`Block`].
    #[instrument(
        level = "info"
        skip(self),
        fields(height),
    )]
    fn finish(self) -> Result<(&'e mut TestNode<C>, Block), anyhow::Error> {
        tracing::trace!("building block");
        let Self {
            data,
            evidence,
            test_node,
            signatures,
        } = self;

        let height = {
            let height = test_node.height.increment();
            test_node.height = height.clone();
            tracing::Span::current().record("height", height.value());
            height
        };

        let last_commit = if height.value() != 1 {
            let block_id = block::Id {
                hash: Hash::None,
                part_set_header: block::parts::Header::new(0, Hash::None)?,
            };
            Some(Commit {
                height,
                round: Round::default(),
                block_id,
                signatures,
            })
        } else {
            None // The first block has no previous commit to speak of.
        };

        let header = Header {
            version: Version { block: 1, app: 1 },
            chain_id: chain::Id::try_from("test".to_owned())?,
            height,
            time: tendermint::Time::now(),
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: Hash::None,
            next_validators_hash: Hash::None,
            consensus_hash: Hash::None,
            app_hash: AppHash::try_from(Vec::default())?,
            last_results_hash: None,
            evidence_hash: None,
            proposer_address: account::Id::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]),
        };
        let block = Block::new(header, data, evidence, last_commit)?;

        Ok((test_node, block))
    }
}
