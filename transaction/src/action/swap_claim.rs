use penumbra_crypto::dex::BatchSwapOutputData;
use penumbra_crypto::dex::TradingPair;
use penumbra_crypto::transaction::Fee;
use penumbra_crypto::Nullifier;
use penumbra_crypto::{proofs::transparent::OutputProof, NotePayload};
use penumbra_proto::{dex as pb, Protobuf};
use penumbra_tct as tct;

#[derive(Debug, Clone)]
pub struct SwapClaim {
    zkproof: OutputProof,
    body: Body,
}

impl Protobuf<pb::SwapClaim> for SwapClaim {}

impl From<SwapClaim> for pb::SwapClaim {
    fn from(sc: SwapClaim) -> Self {
        pb::SwapClaim {
            zkproof: sc.zkproof.into(),
            body: Some(sc.body.into()),
        }
    }
}

impl TryFrom<pb::SwapClaim> for SwapClaim {
    type Error = anyhow::Error;
    fn try_from(sc: pb::SwapClaim) -> Result<Self, Self::Error> {
        Ok(Self {
            zkproof: sc.zkproof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("SwapClaim proof malformed"))?,
            body: sc
                .body
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub nullifier: Nullifier,
    pub fee: Fee,
    pub output_1: NotePayload,
    pub output_2: NotePayload,
    pub output_data: BatchSwapOutputData,
    pub anchor: tct::Root,
    pub trading_pair: TradingPair,
}

impl Protobuf<pb::SwapClaimBody> for Body {}

impl From<Body> for pb::SwapClaimBody {
    fn from(s: Body) -> Self {
        pb::SwapClaimBody {
            trading_pair: Some(s.trading_pair.into()),
            nullifier: Some(s.nullifier.into()),
            fee: Some(s.fee.into()),
            output_1: Some(s.output_1.into()),
            output_2: Some(s.output_2.into()),
            anchor: Some(s.anchor.into()),
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
            output_data: sc
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            trading_pair: sc
                .trading_pair
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
        })
    }
}
