use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proof_params::DELEGATOR_VOTE_PROOF_PROVING_KEY;
use penumbra_sdk_proto::{core::component::governance::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_shielded_pool::Note;
use penumbra_sdk_tct as tct;
use rand::{CryptoRng, RngCore};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::delegator_vote::action::{DelegatorVote, DelegatorVoteBody};
use crate::delegator_vote::proof::DelegatorVoteProof;
use crate::DelegatorVoteProofPrivate;
use crate::DelegatorVoteProofPublic;
use crate::{vote::Vote, VotingReceiptToken};

/// A plan to vote as a delegator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::DelegatorVotePlan", into = "pb::DelegatorVotePlan")]
pub struct DelegatorVotePlan {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The start position of the proposal.
    pub start_position: tct::Position,
    /// The vote to cast.
    pub vote: Vote,
    /// A staked note that was spendable before the proposal started.
    pub staked_note: Note,
    /// The unbonded amount corresponding to the staked note.
    pub unbonded_amount: Amount,
    /// The position of the staked note.
    pub position: tct::Position,
    /// The randomizer to use.
    pub randomizer: Fr,
}

impl DelegatorVotePlan {
    /// Create a new [`DelegatorVotePlan`] that votes using the given positioned `note`.
    #[allow(clippy::too_many_arguments)]
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        proposal: u64,
        start_position: tct::Position,
        vote: Vote,
        staked_note: Note,
        position: tct::Position,
        unbonded_amount: Amount,
    ) -> DelegatorVotePlan {
        DelegatorVotePlan {
            proposal,
            start_position,
            vote,
            staked_note,
            unbonded_amount,
            position,
            randomizer: Fr::rand(rng),
        }
    }

    /// Convenience method to construct the [`DelegatorVote`] described by this [`DelegatorVotePlan`].
    pub fn delegator_vote(
        &self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
    ) -> DelegatorVote {
        DelegatorVote {
            body: self.delegator_vote_body(fvk),
            auth_sig,
            proof: self.delegator_vote_proof(fvk, auth_path),
        }
    }

    /// Construct the [`DelegatorVoteBody`] described by this [`DelegatorVotePlan`].
    pub fn delegator_vote_body(&self, fvk: &FullViewingKey) -> DelegatorVoteBody {
        DelegatorVoteBody {
            proposal: self.proposal,
            start_position: self.start_position,
            vote: self.vote,
            value: self.staked_note.value(),
            unbonded_amount: self.unbonded_amount,
            nullifier: Nullifier::derive(
                fvk.nullifier_key(),
                self.position,
                &self.staked_note.commit(),
            ),
            rk: fvk.spend_verification_key().randomize(&self.randomizer),
        }
    }

    /// Construct the [`DelegatorVoteProof`] required by the [`DelegatorVoteBody`] described by this [`DelegatorVotePlan`].
    pub fn delegator_vote_proof(
        &self,
        fvk: &FullViewingKey,
        state_commitment_proof: tct::Proof,
    ) -> DelegatorVoteProof {
        let public = DelegatorVoteProofPublic {
            anchor: state_commitment_proof.root(),
            balance_commitment: self.staked_note.value().commit(Fr::zero()),
            nullifier: self.nullifier(fvk),
            rk: self.rk(fvk),
            start_position: self.start_position,
        };
        let private = DelegatorVoteProofPrivate {
            state_commitment_proof,
            note: self.staked_note.clone(),
            v_blinding: Fr::from(0u64),
            spend_auth_randomizer: self.randomizer,
            ak: *fvk.spend_verification_key(),
            nk: *fvk.nullifier_key(),
        };
        DelegatorVoteProof::prove(
            &mut OsRng,
            &DELEGATOR_VOTE_PROOF_PROVING_KEY,
            public,
            private,
        )
        .expect("can generate ZK delegator vote proof")
    }

    /// Construct the randomized verification key associated with this [`DelegatorVotePlan`].
    pub fn rk(&self, fvk: &FullViewingKey) -> decaf377_rdsa::VerificationKey<SpendAuth> {
        fvk.spend_verification_key().randomize(&self.randomizer)
    }

    /// Construct the [`Nullifier`] associated with this [`DelegatorVotePlan`].
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        Nullifier::derive(
            fvk.nullifier_key(),
            self.position,
            &self.staked_note.commit(),
        )
    }

    pub fn balance(&self) -> Balance {
        Value {
            amount: self.unbonded_amount,
            asset_id: VotingReceiptToken::new(self.proposal).id(),
        }
        .into()
    }
}

#[allow(deprecated)]
impl From<DelegatorVotePlan> for pb::DelegatorVotePlan {
    fn from(inner: DelegatorVotePlan) -> Self {
        pb::DelegatorVotePlan {
            proposal: inner.proposal,
            vote: Some(inner.vote.into()),
            start_position: inner.start_position.into(),
            staked_note: Some(inner.staked_note.into()),
            unbonded_amount: Some(inner.unbonded_amount.into()),
            staked_note_position: inner.position.into(),
            randomizer: inner.randomizer.to_bytes().to_vec(),
            proof_blinding_r: vec![],
            proof_blinding_s: vec![],
        }
    }
}

impl TryFrom<pb::DelegatorVotePlan> for DelegatorVotePlan {
    type Error = anyhow::Error;

    fn try_from(value: pb::DelegatorVotePlan) -> Result<Self, Self::Error> {
        Ok(DelegatorVotePlan {
            proposal: value.proposal,
            start_position: value.start_position.into(),
            vote: value
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `DelegatorVotePlan`"))?
                .try_into()?,
            staked_note: value
                .staked_note
                .ok_or_else(|| anyhow::anyhow!("missing staked note in `DelegatorVotePlan`"))?
                .try_into()?,
            unbonded_amount: value
                .unbonded_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonded amount in `DelegatorVotePlan`"))?
                .try_into()?,
            position: value.staked_note_position.into(),
            randomizer: Fr::from_bytes_checked(
                value
                    .randomizer
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid randomizer"))?,
            )
            .expect("randomizer malformed"),
        })
    }
}

impl DomainType for DelegatorVotePlan {
    type Proto = pb::DelegatorVotePlan;
}
