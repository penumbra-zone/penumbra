use penumbra_crypto::{proofs::transparent::OutputProof, MockFlowCiphertext};
use penumbra_proto::{dex as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Swap", into = "pb::Swap")]
pub struct Swap {
    // A proof that this is a valid state change.
    proof: OutputProof,
    // The encrypted amount of asset 1 to be swapped.
    enc_amount_1: MockFlowCiphertext,
    // The encrypted amount of asset 2 to be swapped.
    enc_amount_2: MockFlowCiphertext,
}

impl Protobuf<pb::Swap> for Swap {}

impl From<Swap> for pb::Swap {
    fn from(s: Swap) -> Self {
        pb::Swap {
            zkproof: s.proof.into(),
            enc_amount_1: Some(s.enc_amount_1.into()),
            enc_amount_2: Some(s.enc_amount_2.into()),
        }
    }
}

impl TryFrom<pb::Swap> for Swap {
    type Error = anyhow::Error;
    fn try_from(s: pb::Swap) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: s.zkproof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Swap proof malformed"))?,
            enc_amount_1: s
                .enc_amount_1
                .ok_or_else(|| anyhow::anyhow!("missing enc_amount_1"))?
                .try_into()?,
            enc_amount_2: s
                .enc_amount_2
                .ok_or_else(|| anyhow::anyhow!("missing enc_amount_2"))?
                .try_into()?,
        })
    }
}
