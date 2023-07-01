use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use bytes::Bytes;
use penumbra_asset::balance;
use penumbra_crypto::{
    proofs::groth16::SpendProof,
    rdsa::{Signature, SpendAuth, VerificationKey},
    EffectHash, EffectingData, FieldExt, Nullifier,
};
use penumbra_proto::{core::transaction::v1alpha1 as transaction, DomainType, TypeUrl};

#[derive(Clone, Debug)]
pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: SpendProof,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub balance_commitment: balance::Commitment,
    pub nullifier: Nullifier,
    pub rk: VerificationKey<SpendAuth>,
}

impl EffectingData for Body {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:spend_body")
            .to_state();

        // All of these fields are fixed-length, so we can just throw them
        // in the hash one after the other.
        state.update(&self.balance_commitment.to_bytes());
        state.update(&self.nullifier.0.to_bytes());
        state.update(&self.rk.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl TypeUrl for Spend {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.Spend";
}

impl DomainType for Spend {
    type Proto = transaction::Spend;
}

impl From<Spend> for transaction::Spend {
    fn from(msg: Spend) -> Self {
        transaction::Spend {
            body: Some(msg.body.into()),
            auth_sig: Some(msg.auth_sig.into()),
            proof: Some(msg.proof.into()),
        }
    }
}

impl TryFrom<transaction::Spend> for Spend {
    type Error = Error;

    fn try_from(proto: transaction::Spend) -> anyhow::Result<Self, Self::Error> {
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

impl TypeUrl for Body {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.SpendBody";
}

impl DomainType for Body {
    type Proto = transaction::SpendBody;
}

impl From<Body> for transaction::SpendBody {
    fn from(msg: Body) -> Self {
        let nullifier_bytes: [u8; 32] = msg.nullifier.into();
        let rk_bytes: [u8; 32] = msg.rk.into();
        transaction::SpendBody {
            balance_commitment: Some(msg.balance_commitment.into()),
            nullifier: Bytes::copy_from_slice(&nullifier_bytes),
            rk: Bytes::copy_from_slice(&rk_bytes),
        }
    }
}

impl TryFrom<transaction::SpendBody> for Body {
    type Error = Error;

    fn try_from(proto: transaction::SpendBody) -> anyhow::Result<Self, Self::Error> {
        let balance_commitment: balance::Commitment = proto
            .balance_commitment
            .ok_or_else(|| anyhow::anyhow!("missing value commitment"))?
            .try_into()
            .context("malformed balance commitment")?;

        let nullifier = (proto.nullifier[..])
            .try_into()
            .context("malformed nullifier")?;

        let rk_bytes: [u8; 32] = (proto.rk[..])
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected 32-byte rk"))?;
        let rk = rk_bytes.try_into().context("malformed rk")?;

        Ok(Body {
            balance_commitment,
            nullifier,
            rk,
        })
    }
}
