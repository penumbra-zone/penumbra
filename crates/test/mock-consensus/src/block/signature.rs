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
    pub(super) fn commit(validator_address: Id) -> CommitSig {
        CommitSig::BlockIdFlagCommit {
            validator_address,
            timestamp: timestamp(),
            signature: None,
        }
    }

    /// Returns a [commit signature] saying this validator voted nil.
    ///
    /// [commit signature]: CommitSig
    #[allow(dead_code)]
    pub(super) fn nil(validator_address: Id) -> CommitSig {
        CommitSig::BlockIdFlagNil {
            validator_address,
            timestamp: timestamp(),
            signature: None,
        }
    }

    /// Generates a new timestamp, marked at the current time.
    //
    //  TODO(kate): see https://github.com/penumbra-zone/penumbra/issues/3759, re: timestamps.
    //              eventually, we will add hooks so that we can control these timestamps.
    fn timestamp() -> Time {
        Time::now()
    }
}

// === impl TestNode ===

impl<C> TestNode<C> {
    // TODO(kate): other interfaces may be helpful to add in the future, and these may eventually
    // warrant being made `pub`. we defer doing so for now, only defining what is needed to provide
    // commit signatures from all of the validators.

    /// Returns an [`Iterator`] of signatures for validators in the keyring.
    pub(super) fn generate_signatures(&self) -> impl Iterator<Item = CommitSig> + '_ {
        self.keyring
            .iter()
            // Compute the address of this validator.
            .map(|(vk, _)| -> [u8; 20] {
                <Sha256 as Digest>::digest(vk).as_slice()[0..20]
                    .try_into()
                    .expect("")
            })
            .map(account::Id::new)
            .map(self::sign::commit)
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
            votes: signatures
                .into_iter()
                .map(Self::vote)
                .filter_map(|v| v)
                .collect(),
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
