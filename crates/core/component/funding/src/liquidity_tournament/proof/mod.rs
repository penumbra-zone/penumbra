use ark_groth16::{PreparedVerifyingKey, ProvingKey};
use base64::prelude::*;
use decaf377::{Bls12_377, Fq};
use decaf377_rdsa::{Fr, SpendAuth, VerificationKey};
use penumbra_sdk_asset::Value;
use penumbra_sdk_governance::{
    DelegatorVoteProof, DelegatorVoteProofPrivate, DelegatorVoteProofPublic,
};
use penumbra_sdk_keys::keys::NullifierKey;
use penumbra_sdk_proof_params::VerifyingKeyExt as _;
use penumbra_sdk_proto::{core::component::funding::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_shielded_pool::Note;
use penumbra_sdk_tct as tct;

#[derive(Clone, Debug)]
pub struct LiquidityTournamentVoteProofPublic {
    /// the merkle root of the state commitment tree.
    pub anchor: tct::Root,
    /// The value of the note being used to vote.
    pub value: Value,
    /// nullifier of the note to be spent.
    pub nullifier: Nullifier,
    /// the randomized verification spend key.
    pub rk: VerificationKey<SpendAuth>,
    /// the start position of the proposal being voted on.
    pub start_position: tct::Position,
}

impl LiquidityTournamentVoteProofPublic {
    fn to_delegator_vote(self) -> DelegatorVoteProofPublic {
        DelegatorVoteProofPublic {
            anchor: self.anchor,
            balance_commitment: self.value.commit(Default::default()),
            nullifier: self.nullifier,
            rk: self.rk,
            start_position: self.start_position,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LiquidityTournamentVoteProofPrivate {
    /// Inclusion proof for the note commitment.
    pub state_commitment_proof: tct::Proof,
    /// The note being spent.
    pub note: Note,
    /// The randomizer used for generating the randomized spend auth key.
    pub spend_auth_randomizer: Fr,
    /// The spend authorization key.
    pub ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    pub nk: NullifierKey,
}

impl LiquidityTournamentVoteProofPrivate {
    fn to_delegator_vote(self) -> DelegatorVoteProofPrivate {
        DelegatorVoteProofPrivate {
            state_commitment_proof: self.state_commitment_proof,
            note: self.note,
            v_blinding: Default::default(),
            spend_auth_randomizer: self.spend_auth_randomizer,
            ak: self.ak,
            nk: self.nk,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LiquidityTournamentVoteProof(DelegatorVoteProof);

impl LiquidityTournamentVoteProof {
    #![allow(clippy::too_many_arguments)]
    /// Generate a `LiquidityTournamentVoteProof` given the proving key,
    // public inputs, witness data, and the necessary randomness.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: LiquidityTournamentVoteProofPublic,
        private: LiquidityTournamentVoteProofPrivate,
    ) -> anyhow::Result<Self> {
        let proof = DelegatorVoteProof::prove(
            blinding_r,
            blinding_s,
            pk,
            public.to_delegator_vote(),
            private.to_delegator_vote(),
        )?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?BASE64_STANDARD.encode(self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: LiquidityTournamentVoteProofPublic,
    ) -> anyhow::Result<()> {
        Ok(self.0.verify(vk, public.to_delegator_vote())?)
    }
}

impl DomainType for LiquidityTournamentVoteProof {
    type Proto = pb::ZkLiquidityTournamentVoteProof;
}

impl From<LiquidityTournamentVoteProof> for pb::ZkLiquidityTournamentVoteProof {
    fn from(proof: LiquidityTournamentVoteProof) -> Self {
        pb::ZkLiquidityTournamentVoteProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkLiquidityTournamentVoteProof> for LiquidityTournamentVoteProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkLiquidityTournamentVoteProof) -> Result<Self, Self::Error> {
        Ok(LiquidityTournamentVoteProof(proto.inner[..].try_into()?))
    }
}
