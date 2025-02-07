use anyhow::{anyhow, Context};
use decaf377::{Fq, Fr};
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_sdk_asset::asset::Denom;
use penumbra_sdk_keys::{Address, FullViewingKey};
use penumbra_sdk_proof_params::DELEGATOR_VOTE_PROOF_PROVING_KEY;
use penumbra_sdk_proto::{core::component::funding::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_shielded_pool::note::Note;
use penumbra_sdk_tct::{self as tct};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};

use super::{
    proof::{
        LiquidityTournamentVoteProof, LiquidityTournamentVoteProofPrivate,
        LiquidityTournamentVoteProofPublic,
    },
    ActionLiquidityTournamentVote, LiquidityTournamentVoteBody,
};

/// A plan to vote in the liquidity tournament.
///
/// This structure represents the planned vote before it is actually executed,
/// containing the necessary information and blinding factors for the voting proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionLiquidityTournamentVotePlan",
    into = "pb::ActionLiquidityTournamentVotePlan"
)]
pub struct ActionLiquidityTournamentVotePlan {
    /// The asset the user wants to vote for.
    pub incentivized: Denom,
    /// The address the user wants potential rewards to go to.
    pub rewards_recipient: Address,
    /// The note containing the staked note used for voting.
    pub staked_note: Note,
    /// The position of the staked note.
    pub staked_note_position: tct::Position,
    /// The start position of the tournament.
    pub start_position: tct::Position,
    /// Randomizer for proof of spend capability.
    pub randomizer: Fr,
    /// The first blinding factor used for generating the ZK proof.
    pub proof_blinding_r: Fq,
    /// The second blinding factor used for generating the ZK proof.
    pub proof_blinding_s: Fq,
}

impl ActionLiquidityTournamentVotePlan {
    /// Create a new [`ActionLiquidityTournamentVotePlan`] that votes using the given positioned [`Note`].
    #[allow(clippy::too_many_arguments)]
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        incentivized: Denom,
        rewards_recipient: Address,
        staked_note: Note,
        staked_note_position: tct::Position,
        start_position: tct::Position,
    ) -> ActionLiquidityTournamentVotePlan {
        ActionLiquidityTournamentVotePlan {
            incentivized,
            rewards_recipient,
            staked_note,
            staked_note_position,
            start_position,
            randomizer: Fr::rand(rng),
            proof_blinding_r: Fq::rand(rng),
            proof_blinding_s: Fq::rand(rng),
        }
    }

    pub fn to_body(&self, fvk: &FullViewingKey) -> LiquidityTournamentVoteBody {
        let commitment = self.staked_note.commit();

        let nk = fvk.nullifier_key();
        let nullifier = Nullifier::derive(nk, self.staked_note_position, &commitment);
        let ak = fvk.spend_verification_key();
        let rk = ak.randomize(&self.randomizer);
        let value = self.staked_note.value();

        LiquidityTournamentVoteBody {
            incentivized: self.incentivized.clone(),
            rewards_recipient: self.rewards_recipient.clone(),
            start_position: self.start_position,
            value,
            nullifier,
            rk,
        }
    }

    /// Convert this plan into an action.
    ///
    /// * `fvk`: [`FullViewingKey`], in order to derive keys.
    /// * `auth_sig`: [`Signature<SpendAuth>`], as the signature for the transaction.
    /// * `auth_path`: [`tct::Proof`], witnessing the inclusion of the spent note.
    pub fn to_action(
        self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
    ) -> ActionLiquidityTournamentVote {
        let commitment = self.staked_note.commit();

        let nk = fvk.nullifier_key();
        let nullifier = Nullifier::derive(nk, self.staked_note_position, &commitment);
        let ak = fvk.spend_verification_key();
        let rk = ak.randomize(&self.randomizer);
        let value = self.staked_note.value();

        let public = LiquidityTournamentVoteProofPublic {
            anchor: auth_path.root(),
            value,
            nullifier,
            rk,
            start_position: self.start_position,
        };
        let private = LiquidityTournamentVoteProofPrivate {
            state_commitment_proof: auth_path,
            note: self.staked_note.clone(),
            spend_auth_randomizer: self.randomizer,
            ak: *ak,
            nk: *nk,
        };
        let proof = LiquidityTournamentVoteProof::prove(
            self.proof_blinding_r,
            self.proof_blinding_s,
            &DELEGATOR_VOTE_PROOF_PROVING_KEY,
            public,
            private,
        )
        .expect("can generate ZK LQT voting proof");

        ActionLiquidityTournamentVote {
            body: self.to_body(fvk),
            auth_sig,
            proof,
        }
    }
}

impl DomainType for ActionLiquidityTournamentVotePlan {
    type Proto = pb::ActionLiquidityTournamentVotePlan;
}

impl TryFrom<pb::ActionLiquidityTournamentVotePlan> for ActionLiquidityTournamentVotePlan {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ActionLiquidityTournamentVotePlan) -> Result<Self, Self::Error> {
        let proof_blinding_r_bytes: [u8; 32] = proto
            .proof_blinding_r
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed r in `DelegatorVotePlan`"))?;
        let proof_blinding_s_bytes: [u8; 32] = proto
            .proof_blinding_s
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed s in `DelegatorVotePlan`"))?;
        Result::<_, Self::Error>::Ok(Self {
            incentivized: proto
                .incentivized
                .ok_or_else(|| anyhow!("missing `incentivized`"))?
                .try_into()?,
            rewards_recipient: proto
                .rewards_recipient
                .ok_or_else(|| anyhow!("missing `rewards_recipient`"))?
                .try_into()?,
            staked_note: proto
                .staked_note
                .ok_or_else(|| anyhow!("missing `staked_note`"))?
                .try_into()?,
            staked_note_position: proto.staked_note_position.into(),
            start_position: proto.start_position.into(),
            randomizer: Fr::from_bytes_checked(
                proto
                    .randomizer
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid randomizer"))?,
            )
            .map_err(|_| anyhow!("randomizer malformed"))?,
            proof_blinding_r: Fq::from_bytes_checked(&proof_blinding_r_bytes)
                .map_err(|_| anyhow!("proof_blinding_r malformed"))?,
            proof_blinding_s: Fq::from_bytes_checked(&proof_blinding_s_bytes)
                .map_err(|_| anyhow!("proof_blinding_s malformed"))?,
        })
        .with_context(|| format!("while parsing {}", std::any::type_name::<Self>()))
    }
}

impl From<ActionLiquidityTournamentVotePlan> for pb::ActionLiquidityTournamentVotePlan {
    fn from(value: ActionLiquidityTournamentVotePlan) -> Self {
        Self {
            incentivized: Some(value.incentivized.into()),
            rewards_recipient: Some(value.rewards_recipient.into()),
            staked_note: Some(value.staked_note.into()),
            staked_note_position: value.staked_note_position.into(),
            start_position: value.start_position.into(),
            randomizer: value.randomizer.to_bytes().to_vec(),
            proof_blinding_r: value.proof_blinding_r.to_bytes().to_vec(),
            proof_blinding_s: value.proof_blinding_s.to_bytes().to_vec(),
        }
    }
}
