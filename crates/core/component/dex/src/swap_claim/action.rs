use anyhow::Context;
use penumbra_asset::Balance;
use penumbra_fee::Fee;
use penumbra_proof_params::GROTH16_PROOF_LENGTH_BYTES;
use penumbra_proto::{penumbra::core::component::dex::v1alpha1 as pb, DomainType};
use penumbra_sct::Nullifier;
use penumbra_tct as tct;
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::BatchSwapOutputData;

use super::proof::SwapClaimProof;

#[derive(Debug, Clone)]
pub struct SwapClaim {
    pub proof: SwapClaimProof,
    pub body: Body,
    pub epoch_duration: u64,
}

impl SwapClaim {
    /// Compute a commitment to the value contributed to a transaction by this swap claim.
    /// Will add (f,fee_token) representing the pre-paid fee
    pub fn balance(&self) -> Balance {
        self.body.fee.value().into()
    }
}

impl EffectingData for SwapClaim {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the swap claim, so we can
        // just use hash the proto-encoding of the body.
        self.body.effect_hash()
    }
}

impl DomainType for SwapClaim {
    type Proto = pb::SwapClaim;
}

impl From<SwapClaim> for pb::SwapClaim {
    fn from(sc: SwapClaim) -> Self {
        pb::SwapClaim {
            proof: Some(sc.proof.into()),
            body: Some(sc.body.into()),
            epoch_duration: sc.epoch_duration,
        }
    }
}

impl TryFrom<pb::SwapClaim> for SwapClaim {
    type Error = anyhow::Error;
    fn try_from(sc: pb::SwapClaim) -> Result<Self, Self::Error> {
        let proof_bytes: [u8; GROTH16_PROOF_LENGTH_BYTES] = sc
            .proof
            .ok_or_else(|| anyhow::anyhow!("missing swap claim proof"))?
            .inner
            .as_slice()
            .try_into()
            .context("swap claim proof malformed")?;
        Ok(Self {
            body: sc
                .body
                .ok_or_else(|| anyhow::anyhow!("missing swap claim body"))?
                .try_into()
                .context("swap claim body malformed")?,
            epoch_duration: sc.epoch_duration,
            proof: SwapClaimProof(proof_bytes),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapClaimBody", into = "pb::SwapClaimBody")]
pub struct Body {
    pub nullifier: Nullifier,
    pub fee: Fee,
    pub output_1_commitment: tct::StateCommitment,
    pub output_2_commitment: tct::StateCommitment,
    pub output_data: BatchSwapOutputData,
}

impl EffectingData for Body {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for Body {
    type Proto = pb::SwapClaimBody;
}

impl From<Body> for pb::SwapClaimBody {
    fn from(s: Body) -> Self {
        pb::SwapClaimBody {
            nullifier: Some(s.nullifier.into()),
            fee: Some(s.fee.into()),
            output_1_commitment: Some(s.output_1_commitment.into()),
            output_2_commitment: Some(s.output_2_commitment.into()),
            output_data: Some(s.output_data.into()),
        }
    }
}

impl TryFrom<pb::SwapClaimBody> for Body {
    type Error = anyhow::Error;
    fn try_from(sc: pb::SwapClaimBody) -> Result<Self, Self::Error> {
        Ok(Self {
            nullifier: sc
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            fee: sc
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing fee"))?
                .try_into()?,
            output_1_commitment: sc
                .output_1_commitment
                .ok_or_else(|| anyhow::anyhow!("missing output_1"))?
                .try_into()?,
            output_2_commitment: sc
                .output_2_commitment
                .ok_or_else(|| anyhow::anyhow!("missing output_2"))?
                .try_into()?,
            output_data: sc
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
        })
    }
}
