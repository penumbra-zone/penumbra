use penumbra_crypto::transaction::Fee;
use penumbra_crypto::Nullifier;
use penumbra_crypto::{proofs::transparent::OutputProof, NotePayload};
use penumbra_proto::{dex as pb, Protobuf};
use penumbra_tct as tct;

#[derive(Debug, Clone)]
pub struct SwapClaim {
    proof: OutputProof,
    nullifier: Nullifier,
    fee: Fee,
    output_1: NotePayload,
    output_2: NotePayload,
    anchor: tct::Root,
    price_1: u64,
    price_2: u64,
}

impl Protobuf<pb::SwapClaim> for SwapClaim {}

impl From<SwapClaim> for pb::SwapClaim {
    fn from(sc: SwapClaim) -> Self {
        pb::SwapClaim {
            zkproof: sc.proof.into(),
            nullifier: Some(sc.nullifier.into()),
            fee: Some(sc.fee.into()),
            output_1: Some(sc.output_1.into()),
            output_2: Some(sc.output_2.into()),
            anchor: Some(sc.anchor.into()),
            price_1: sc.price_1,
            price_2: sc.price_2,
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
            output_1: sc
                .output_1
                .ok_or_else(|| anyhow::anyhow!("missing output_1"))?
                .try_into()?,
            output_2: sc
                .output_2
                .ok_or_else(|| anyhow::anyhow!("missing output_2"))?
                .try_into()?,
            anchor: sc
                .anchor
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            price_1: sc.price_1,
            price_2: sc.price_2,
        })
    }
}
