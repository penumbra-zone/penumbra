//! [`Builder`] facilities for constructing [`Block`]s.
//!
/// Builders are acquired by calling [`TestNode::block()`].
use {
    crate::TestNode,
    anyhow::bail,
    tendermint::{
        account,
        block::{header::Version, Block, Commit, Header, Height},
        chain, evidence, AppHash, Hash,
    },
};

/// A builder, used to prepare and instantiate a new [`Block`].
///
/// These are acquired by calling [`TestNode::block()`].
pub struct Builder<'e, C> {
    /// A unique reference to the test node.
    //
    //  NB: this is currently unused, but will eventually be used to fill in header fields, etc.
    #[allow(dead_code)]
    test_node: &'e mut TestNode<C>,

    /// Transaction data.
    data: Option<Vec<Vec<u8>>>,

    /// Evidence of malfeasance.
    evidence: Option<evidence::List>,

    /// Last commit.
    last_commit: Option<Commit>,
}

impl<C> TestNode<C> {
    /// Returns a new [`Builder`].
    pub fn block<'e>(&'e mut self) -> Builder<'e, C> {
        Builder {
            test_node: self,
            data: Default::default(),
            evidence: Default::default(),
            last_commit: Default::default(),
        }
    }
}

impl<'e, C> Builder<'e, C> {
    /// Sets the data for this block.
    pub fn with_data(self, data: Vec<Vec<u8>>) -> Self {
        Self {
            data: Some(data),
            ..self
        }
    }

    /// Sets the evidence [`List`][evidence::List] for this block.
    pub fn with_evidence(self, evidence: evidence::List) -> Self {
        Self {
            evidence: Some(evidence),
            ..self
        }
    }

    /// Sets the last [`Commit`] for this block.
    pub fn with_last_commit(self, last_commit: Commit) -> Self {
        Self {
            last_commit: Some(last_commit),
            ..self
        }
    }

    // TODO(kate): add more `with_` setters for fields in the header.
    // TODO(kate): set some fields using state in the test node.

    /// Consumes this builder, returning a [`Block`].
    pub fn finish(self) -> Result<Block, anyhow::Error> {
        let Self {
            data: Some(data),
            evidence: Some(evidence),
            last_commit,
            test_node: _,
        } = self
        else {
            bail!("builder was not fully initialized")
        };

        let header = Header {
            version: Version { block: 1, app: 1 },
            chain_id: chain::Id::try_from("test".to_owned())?,
            height: Height::try_from(1_u8)?,
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

        Block::new(header, data, evidence, last_commit).map_err(Into::into)
    }
}
