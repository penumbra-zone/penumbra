use {
    super::Builder,
    crate::TestNode,
    sha2::{Digest, Sha256},
    tendermint::{
        abci::types::{BlockSignatureInfo, CommitInfo, VoteInfo},
        account,
        block::{BlockIdFlag, Commit, CommitSig, Round},
        vote::Power,
    },
};

/// Helper functions for generating [commit signatures].
mod sign {
    use tendermint::{account::Id, block::CommitSig, time::Time};

    /// Returns a [commit signature] saying this validator voted for the block.
    ///
    /// [commit signature]: CommitSig
    pub(super) fn commit(
        validator_address: Id,
        validator_key: &ed25519_consensus::SigningKey,
        canonical: &tendermint::vote::CanonicalVote,
    ) -> CommitSig {
        // Create a vote to be signed
        // https://github.com/informalsystems/tendermint-rs/blob/14fd628e82ae51b9f15c135a6db8870219fe3c33/testgen/src/commit.rs#L214
        // https://github.com/informalsystems/tendermint-rs/blob/14fd628e82ae51b9f15c135a6db8870219fe3c33/testgen/src/commit.rs#L104

        use tendermint_proto::v0_37::types::CanonicalVote as RawCanonicalVote;
        let sign_bytes =
            tendermint_proto::Protobuf::<RawCanonicalVote>::encode_length_delimited_vec(
                canonical.clone(),
            );

        let signature: tendermint::Signature = validator_key
            .sign(sign_bytes.as_slice())
            .try_into()
            .unwrap();

        // encode to stable-json deterministic JSON wire encoding,
        // https://github.com/informalsystems/tendermint-rs/blob/14fd628e82ae51b9f15c135a6db8870219fe3c33/testgen/src/helpers.rs#L43C1-L44C1

        CommitSig::BlockIdFlagCommit {
            validator_address,
            timestamp: canonical.timestamp.expect("timestamp should be present"),
            signature: Some(signature.into()),
        }
    }

    /// Returns a [commit signature] saying this validator voted nil.
    ///
    /// [commit signature]: CommitSig
    #[allow(dead_code)]
    pub(super) fn nil(validator_address: Id, timestamp: Time) -> CommitSig {
        CommitSig::BlockIdFlagNil {
            validator_address,
            timestamp,
            signature: None,
        }
    }
}

// === impl TestNode ===

impl<C> TestNode<C> {
    // TODO(kate): other interfaces may be helpful to add in the future, and these may eventually
    // warrant being made `pub`. we defer doing so for now, only defining what is needed to provide
    // commit signatures from all of the validators.

    /// Returns an [`Iterator`] of signatures for validators in the keyring.
    /// Signatures sign the given block header.
    pub(super) fn generate_signatures(
        &self,
        header: &tendermint::block::Header,
    ) -> impl Iterator<Item = CommitSig> + '_ {
        let block_id = tendermint::block::Id {
            hash: header.hash(),
            part_set_header: tendermint::block::parts::Header::new(0, tendermint::Hash::None)
                .unwrap(),
        };
        let canonical = tendermint::vote::CanonicalVote {
            // The mock consensus engine ONLY has precommit votes right now
            vote_type: tendermint::vote::Type::Precommit,
            height: tendermint::block::Height::from(header.height),
            // round is always 0
            round: 0u8.into(),
            block_id: Some(block_id),
            // Block header time is used throughout
            timestamp: Some(header.time.clone()),
            // timestamp: Some(last_commit_info.timestamp),
            chain_id: self.chain_id.clone(),
        };
        tracing::trace!(vote=?canonical,"canonical vote constructed");

        return self
            .keyring
            .iter()
            .map(|(vk, sk)| {
                (
                    <Sha256 as Digest>::digest(vk).as_slice()[0..20]
                        .try_into()
                        .expect(""),
                    sk,
                )
            })
            .map(move |(id, sk)| self::sign::commit(account::Id::new(id), sk, &canonical));
    }
}

// === impl Builder ===

impl<'e, C: 'e> Builder<'e, C> {
    /// Returns [`CommitInfo`] given a block's [`Commit`].
    pub(super) fn last_commit_info(last_commit: Option<Commit>) -> CommitInfo {
        let Some(Commit {
            round, signatures, ..
        }) = last_commit
        else {
            // If there is no commit information about the last block, return an empty object.
            return CommitInfo {
                round: Round::default(),
                votes: Vec::default(),
            };
        };

        CommitInfo {
            round,
            votes: signatures.into_iter().filter_map(Self::vote).collect(),
        }
    }

    /// Returns a [`VoteInfo`] for this [`CommitSig`].
    ///
    /// If no validator voted, returns [`None`].
    fn vote(commit_sig: CommitSig) -> Option<VoteInfo> {
        use tendermint::abci::types::Validator;

        // TODO(kate): upstream this into the `tendermint` library.
        let sig_info = BlockSignatureInfo::Flag(match commit_sig {
            CommitSig::BlockIdFlagAbsent => BlockIdFlag::Absent,
            CommitSig::BlockIdFlagCommit { .. } => BlockIdFlag::Commit,
            CommitSig::BlockIdFlagNil { .. } => BlockIdFlag::Nil,
        });

        let address: [u8; 20] = commit_sig
            .validator_address()?
            // TODO(kate): upstream an accessor to retrieve this as the [u8; 20] that it is.
            .as_bytes()
            .try_into()
            .expect("validator address should be 20 bytes");
        let power = Power::from(1_u8); // TODO(kate): for now, hard-code voting power to 1.
        let validator = Validator { address, power };

        Some(VoteInfo {
            validator,
            sig_info,
        })
    }
}
