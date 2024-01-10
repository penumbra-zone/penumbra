use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_asset::balance;
use penumbra_proto::{core::component::shielded_pool::v1alpha1 as pb, DomainType};
use penumbra_sct::Nullifier;
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::SpendProof;

#[derive(Clone, Debug)]
pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: SpendProof,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SpendBody", into = "pb::SpendBody")]
pub struct Body {
    pub balance_commitment: balance::Commitment,
    pub nullifier: Nullifier,
    pub rk: VerificationKey<SpendAuth>,
}

impl EffectingData for Body {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl EffectingData for Spend {
    fn effect_hash(&self) -> EffectHash {
        // The effecting data is in the body of the spend, so we can
        // just use hash the proto-encoding of the body.
        self.body.effect_hash()
    }
}

impl DomainType for Spend {
    type Proto = pb::Spend;
}

impl From<Spend> for pb::Spend {
    fn from(msg: Spend) -> Self {
        pb::Spend {
            body: Some(msg.body.into()),
            auth_sig: Some(msg.auth_sig.into()),
            proof: Some(msg.proof.into()),
        }
    }
}

impl TryFrom<pb::Spend> for Spend {
    type Error = Error;

    fn try_from(proto: pb::Spend) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or_else(|| anyhow::anyhow!("missing spend body"))?
            .try_into()
            .context("malformed spend body")?;
        let auth_sig = proto
            .auth_sig
            .ok_or_else(|| anyhow::anyhow!("missing auth sig"))?
            .try_into()
            .context("malformed auth sig")?;
        let proof = proto
            .proof
            .ok_or_else(|| anyhow::anyhow!("missing proof"))?
            .try_into()
            .context("malformed spend proof")?;

        Ok(Spend {
            body,
            auth_sig,
            proof,
        })
    }
}

impl DomainType for Body {
    type Proto = pb::SpendBody;
}

impl From<Body> for pb::SpendBody {
    fn from(msg: Body) -> Self {
        pb::SpendBody {
            balance_commitment: Some(msg.balance_commitment.into()),
            nullifier: Some(msg.nullifier.into()),
            rk: Some(msg.rk.into()),
        }
    }
}

impl TryFrom<pb::SpendBody> for Body {
    type Error = Error;

    fn try_from(proto: pb::SpendBody) -> anyhow::Result<Self, Self::Error> {
        let balance_commitment: balance::Commitment = proto
            .balance_commitment
            .ok_or_else(|| anyhow::anyhow!("missing value commitment"))?
            .try_into()
            .context("malformed balance commitment")?;

        let nullifier = proto
            .nullifier
            .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
            .try_into()
            .context("malformed nullifier")?;

        let rk = proto
            .rk
            .ok_or_else(|| anyhow::anyhow!("missing rk"))?
            .try_into()
            .context("malformed rk")?;

        Ok(Body {
            balance_commitment,
            nullifier,
            rk,
        })
    }
}
