use ark_ff::Zero;
use decaf377::Fr;
use penumbra_crypto::dex::BatchSwapOutputData;
use penumbra_crypto::dex::TradingPair;
use penumbra_crypto::transaction::Fee;
use penumbra_crypto::value;
use penumbra_crypto::Nullifier;
use penumbra_crypto::{proofs::transparent::SwapClaimProof, NotePayload};
use penumbra_proto::{dex as pb, Protobuf};

#[derive(Debug, Clone)]
pub struct SwapClaim {
    pub zkproof: SwapClaimProof,
    pub body: Body,
}

impl SwapClaim {
    /// Compute a commitment to the value contributed to a transaction by this swap claim.
    /// Will add (f,fee_token)
    pub fn value_commitment(&self) -> value::Commitment {
        self.body.fee.commit(Fr::zero())
    }
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
            output_1: sc
                .output_1
                .ok_or_else(|| anyhow::anyhow!("missing output_1"))?
                .try_into()?,
            output_2: sc
                .output_2
                .ok_or_else(|| anyhow::anyhow!("missing output_2"))?
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
