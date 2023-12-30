use anyhow::Context;
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_asset::{balance, Balance, Value};
use penumbra_num::Amount;
use penumbra_proto::{
    core::component::dex::v1alpha1 as pbc, penumbra::core::component::dex::v1alpha1 as pb,
    DomainType,
};
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::TradingPair;

use super::{proof::SwapProof, SwapPayload};

#[derive(Clone, Debug)]
pub struct Swap {
    pub proof: SwapProof,
    pub body: Body,
}

impl Swap {
    /// Temporary method until we resolve where `IsAction::balance_commitment` should live.
    pub fn balance_commitment_inner(&self) -> balance::Commitment {
        let input_1 = Value {
            amount: self.body.delta_1_i,
            asset_id: self.body.trading_pair.asset_1(),
        };
        let input_1 = -Balance::from(input_1);
        let commitment_input_1 = input_1.commit(Fr::zero());
        let input_2 = Value {
            amount: self.body.delta_2_i,
            asset_id: self.body.trading_pair.asset_2(),
        };
        let input_2 = -Balance::from(input_2);
        let commitment_input_2 = input_2.commit(Fr::zero());

        commitment_input_1 + commitment_input_2 + self.body.fee_commitment
    }
}

impl EffectingData for Swap {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the swap, so we can
        // just use hash the proto-encoding of the body.
        self.body.effect_hash()
    }
}

impl DomainType for Swap {
    type Proto = pb::Swap;
}

impl From<Swap> for pb::Swap {
    fn from(s: Swap) -> Self {
        let proof: pbc::ZkSwapProof = s.proof.into();
        pb::Swap {
            proof: Some(proof),
            body: Some(s.body.into()),
        }
    }
}

impl TryFrom<pb::Swap> for Swap {
    type Error = anyhow::Error;
    fn try_from(s: pb::Swap) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: s
                .proof
                .ok_or_else(|| anyhow::anyhow!("missing swap proof"))?
                .try_into()
                .context("swap proof malformed")?,
            body: s
                .body
                .ok_or_else(|| anyhow::anyhow!("missing swap body"))?
                .try_into()
                .context("swap body malformed")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapBody", into = "pb::SwapBody")]
pub struct Body {
    pub trading_pair: TradingPair,
    pub delta_1_i: Amount,
    pub delta_2_i: Amount,
    pub fee_commitment: balance::Commitment,
    pub payload: SwapPayload,
}

impl EffectingData for Body {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for Body {
    type Proto = pb::SwapBody;
}

impl From<Body> for pb::SwapBody {
    fn from(s: Body) -> Self {
        pb::SwapBody {
            trading_pair: Some(s.trading_pair.into()),
            delta_1_i: Some(s.delta_1_i.into()),
            delta_2_i: Some(s.delta_2_i.into()),
            fee_commitment: Some(s.fee_commitment.into()),
            payload: Some(s.payload.into()),
        }
    }
}

impl TryFrom<pb::SwapBody> for Body {
    type Error = anyhow::Error;
    fn try_from(s: pb::SwapBody) -> Result<Self, Self::Error> {
        Ok(Self {
            trading_pair: s
                .trading_pair
                .ok_or_else(|| anyhow::anyhow!("missing trading_pair"))?
                .try_into()?,

            delta_1_i: s
                .delta_1_i
                .ok_or_else(|| anyhow::anyhow!("missing delta_1"))?
                .try_into()?,
            delta_2_i: s
                .delta_2_i
                .ok_or_else(|| anyhow::anyhow!("missing delta_2"))?
                .try_into()?,

            fee_commitment: s
                .fee_commitment
                .ok_or_else(|| anyhow::anyhow!("missing fee_commitment"))?
                .try_into()?,
            payload: s
                .payload
                .ok_or_else(|| anyhow::anyhow!("missing payload"))?
                .try_into()?,
        })
    }
}
