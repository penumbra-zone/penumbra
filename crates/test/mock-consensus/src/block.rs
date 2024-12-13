//! [`Builder`] facilities for constructing [`Block`]s.
//!
//! Builders are acquired by calling [`TestNode::block()`], see [`TestNode`] for more information.

use {
    crate::TestNode,
    prost::Message,
    sha2::{Digest, Sha256},
    std::ops::Deref,
    tap::Tap,
    tendermint::{
        abci::Event,
        account,
        block::{self, header::Version, Block, Commit, Header, Round},
        evidence,
        merkle::simple_hash_from_byte_vectors,
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
        Hash, Time,
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
    /// The timestamp of the block.
    timestamp: Time,
    /// Disable producing signatures. Defaults to produce signatures.
    disable_signatures: bool,
}

// === impl TestNode ===

impl<C> TestNode<C> {
    /// Returns a new [`Builder`].
    ///
    pub fn block(&mut self) -> Builder<'_, C> {
        let ts = self.timestamp.clone();
        Builder {
            test_node: self,
            data: Default::default(),
            evidence: Default::default(),
            timestamp: ts,
            disable_signatures: false,
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

    /// Disables producing commit signatures for this block.
    pub fn without_signatures(self) -> Self {
        Self {
            disable_signatures: true,
            ..self
        }
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
    ///
    /// By default, signatures for all of the validators currently within the keyring will be
    /// included in the block. Use [`Builder::without_signatures()`] to disable producing
    /// validator signatures.
    #[instrument(level = "info", skip_all, fields(height, time))]
    pub async fn execute(self) -> Result<(EndBlockEvents, DeliverTxEvents), anyhow::Error> {
        // Calling `finish` finishes the previous block
        // and prepares the current block.
        let (test_node, block) = self.finish()?;

        let Block {
            // The header for the current block
            header,
            data,
            evidence: _,
            // Votes for the previous block
            last_commit,
            ..
        } = block.clone().tap(|block| {
            tracing::span::Span::current()
                .record("height", block.header.height.value())
                .record("time", block.header.time.unix_timestamp());
        });
        let last_commit_info = Self::last_commit_info(last_commit);

        trace!("sending block");
        test_node.begin_block(header, last_commit_info).await?;
        let mut deliver_tx_responses = Vec::new();
        for tx in data {
            let tx = tx.into();
            // The caller may want to access the DeliverTx responses
            deliver_tx_responses.push(test_node.deliver_tx(tx).await?);
        }

        // The CheckTx, BeginBlock, DeliverTx, EndBlock methods include an Events field.
        // The mock consensus code only handles EndBlock and DeliverTx events.
        // Extract the events emitted during end_block.
        let events = test_node.end_block().await?.events;
        let deliver_tx_events = deliver_tx_responses
            .iter()
            .flat_map(|response| response.events.clone())
            .collect::<Vec<_>>();

        // the commit call will set test_node.last_app_hash, preparing
        // for the next block to begin execution
        let commit_response = test_node.commit().await?;

        // NOTE: after calling .commit(), the internal status of the pd node's storage is going to be updated
        // to the next block
        // therefore we need to update the height within our mock now now

        // Set the last app hash to the new block's app hash.
        test_node.last_app_hash = commit_response.data.to_vec();
        trace!(
            last_app_hash = ?hex::encode(commit_response.data.to_vec()),
            "test node has committed block, setting last_app_hash"
        );

        // If an `on_block` callback was set, call it now.
        test_node.on_block.as_mut().map(move |f| f(block));

        Ok((EndBlockEvents(events), DeliverTxEvents(deliver_tx_events)))
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
            timestamp,
            disable_signatures,
        } = self;

        // Call the timestamp callback to increment the node's current timestamp.
        test_node.timestamp = (test_node.ts_callback)(test_node.timestamp.clone());

        // The first (non-genesis) block has height 1.
        let height = {
            let height = test_node.height.increment();
            test_node.height = height;
            tracing::Span::current().record("height", height.value());
            height
        };

        // Pull the current last_commit out of the node, since it will
        // be discarded after we build the block.
        let last_commit = test_node.last_commit.clone();

        // Set the validator set based on the current configuration.
        let pk = test_node
            .keyring
            .iter()
            .next()
            .expect("validator key in keyring")
            .0;
        let proposer_address = account::Id::new(
            <Sha256 as sha2::Digest>::digest(pk).as_slice()[0..20]
                .try_into()
                .expect(""),
        );

        let validators_hash = test_node.last_validator_set_hash.clone().unwrap();

        // The data hash is the sha256 hash of all the transactions
        // I think as long as we are consistent here it's fine.
        let data_hash = sha2::Sha256::digest(&data.concat()).to_vec();
        let consensus_hash = test_node.consensus_params_hash.clone().try_into().unwrap();
        let header = Header {
            // Protocol version. Block version 11 matches cometbft when tests were written.
            version: Version { block: 11, app: 0 },
            chain_id: tendermint::chain::Id::try_from(test_node.chain_id.clone())?,
            // Height is the height for this header.
            height,
            time: timestamp,
            // MerkleRoot of the lastCommit’s signatures. The signatures represent the validators that committed to the last block. The first block has an empty slices of bytes for the hash.
            last_commit_hash: last_commit
                .as_ref()
                .map(|c| c.hash().unwrap())
                .unwrap_or(Some(
                    // empty hash value
                    hex::decode(
                        "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                    )?
                    .try_into()?,
                )),
            last_block_id: test_node.last_commit.as_ref().map(|c| c.block_id.clone()),
            // MerkleRoot of the hash of transactions. Note: The transactions are hashed before being included in the merkle tree, the leaves of the Merkle tree are the hashes, not the transactions themselves.
            data_hash: Some(tendermint::Hash::Sha256(data_hash.try_into().unwrap())),
            // force the header to have the hash of the validator set to pass
            // the validation
            // MerkleRoot of the current validator set
            validators_hash: validators_hash.into(),
            // MerkleRoot of the next validator set
            next_validators_hash: validators_hash.into(),
            // Hash of the protobuf encoded consensus parameters.
            consensus_hash,
            // Arbitrary byte array returned by the application after executing and committing the previous block.
            app_hash: tendermint::AppHash::try_from(test_node.last_app_hash().to_vec())?,
            // TODO: we should probably have a way to set this
            // root hash of a Merkle tree built from DeliverTxResponse responses(Log,Info, Codespace and Events fields are ignored).The first block has block.Header.ResultsHash == MerkleRoot(nil), i.e. the hash of an empty input, for RFC-6962 conformance.
            // the go version will shasum empty bytes and produce "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
            last_results_hash: Some(
                hex::decode("E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855")?
                    .try_into()?,
            ),
            // MerkleRoot of the evidence of Byzantine behavior included in this block.
            evidence_hash: Some(
                hex::decode("E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855")?
                    .try_into()?,
            ),
            // Address of the original proposer of the block. Validator must be in the current validatorSet.
            proposer_address,
        };
        tracing::trace!(?header, "built block header");

        // The next block will use the signatures of this block's header.
        let signatures: Vec<block::CommitSig> = if !disable_signatures {
            test_node.generate_signatures(&header).collect()
        } else {
            vec![]
        };

        tracing::trace!(
            height=?height.value(),
            app_hash=?hex::encode(header.app_hash.clone()),
            block_id=?hex::encode(header.hash()),
            last_commit_height=?last_commit.as_ref().map(|c| c.height.value()),
            "made block"
        );
        // pass the current value of last_commit with this header
        let block = Block::new(header.clone(), data, evidence, last_commit);

        // Now that the block is finalized, we can transition to the next block.
        // Generate a commit for the header we just made, that will be
        // included in the next header.
        // Update the last_commit.
        test_node.last_commit = Some(Commit {
            height: block.header.height,
            round: Round::default(),
            block_id: block::Id {
                hash: block.header.hash().into(),
                // The part_set_header seems to be used internally by cometbft
                // and the pd node doesn't care about it
                part_set_header: block::parts::Header::new(0, Hash::None)?,
            },
            // Signatures of the last block
            signatures: signatures.clone(),
        });

        Ok((test_node, block))
    }
}

// Allows hashing of commits
pub trait CommitHashingExt: Sized {
    fn hash(&self) -> anyhow::Result<Option<Hash>>;
}

impl CommitHashingExt for Commit {
    // https://github.com/tendermint/tendermint/blob/51dc810d041eaac78320adc6d53ad8b160b06601/types/block.go#L672
    fn hash(&self) -> anyhow::Result<Option<Hash>> {
        // make a vec of the precommit protobuf encodings
        // then merkle hash them
        // https://github.com/tendermint/tendermint/blob/35581cf54ec436b8c37fabb43fdaa3f48339a170/crypto/merkle/tree.go#L9
        let bs = self
            .signatures
            .iter()
            .map(|precommit| {
                tendermint_proto::types::CommitSig::from(precommit.clone()).encode_to_vec()
            })
            .collect::<Vec<_>>();

        match bs.len() {
            0 =>
            // empty hash
            {
                Ok(Some(
                    hex::decode(
                        "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                    )?
                    .try_into()?,
                ))
            }
            _ => Ok(Some(
                simple_hash_from_byte_vectors::<Sha256>(&bs)
                    .to_vec()
                    .try_into()?,
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EndBlockEvents(pub Vec<Event>);

#[derive(Debug, Clone)]
pub struct DeliverTxEvents(pub Vec<Event>);

impl Deref for DeliverTxEvents {
    type Target = Vec<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for EndBlockEvents {
    type Target = Vec<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
