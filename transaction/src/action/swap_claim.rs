use penumbra_crypto::proofs::transparent::OutputProof;
use penumbra_crypto::Nullifier;
use penumbra_proto::{dex as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::Fee;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapClaim", into = "pb::SwapClaim")]
pub struct SwapClaim {
    proof: OutputProof,
    nullifier: Nullifier,
    fee: Fee,
}

impl Protobuf<pb::SwapClaim> for SwapClaim {}

impl From<SwapClaim> for pb::SwapClaim {
    fn from(sc: SwapClaim) -> Self {
        pb::SwapClaim {
            zkproof: sc.proof.into(),
            nullifier: Some(sc.nullifier.into()),
            fee: Some(sc.fee.into()),
        }
    }
}

impl TryFrom<pb::SwapClaim> for SwapClaim {
    type Error = anyhow::Error;
    fn try_from(sc: pb::SwapClaim) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: sc.zkproof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("SwapClaim proof malformed"))?,
            nullifier: sc
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            fee: sc.fee.ok_or_else(|| anyhow::anyhow!("missing fee"))?.into(),
        })
    }
}
