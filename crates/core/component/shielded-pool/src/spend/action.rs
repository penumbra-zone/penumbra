use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_sdk_asset::balance;
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::SpendProof;
use crate::{backref::ENCRYPTED_BACKREF_LEN, EncryptedBackref};

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
    pub encrypted_backref: EncryptedBackref,
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
            encrypted_backref: msg.encrypted_backref.into(),
        }
    }
}

impl TryFrom<pb::SpendBody> for Body {
    type Error = Error;

    fn try_from(proto: pb::SpendBody) -> anyhow::Result<Self, Self::Error> {
        let balance_commitment: balance::Commitment = proto
            .balance_commitment
            .ok_or_else(|| anyhow::anyhow!("missing balance commitment"))?
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

        // `EncryptedBackref` must have 0 or `ENCRYPTED_BACKREF_LEN` bytes.
        let encrypted_backref: EncryptedBackref;
        if proto.encrypted_backref.len() == ENCRYPTED_BACKREF_LEN {
            let bytes: [u8; ENCRYPTED_BACKREF_LEN] = proto
                .encrypted_backref
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid encrypted backref"))?;
            encrypted_backref = EncryptedBackref::try_from(bytes)
                .map_err(|_| anyhow::anyhow!("invalid encrypted backref"))?;
        } else if proto.encrypted_backref.len() == 0 {
            encrypted_backref = EncryptedBackref::dummy();
        } else {
            return Err(anyhow::anyhow!("invalid encrypted backref length"));
        }

        Ok(Body {
            balance_commitment,
            nullifier,
            rk,
            encrypted_backref,
        })
    }
}
