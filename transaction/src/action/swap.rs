use bytes::Bytes;

use penumbra_crypto::rdsa::{Signature, SpendAuth};
use penumbra_crypto::value;
use penumbra_crypto::NotePayload;
use penumbra_crypto::{proofs::transparent::SpendProof, MockFlowCiphertext};
use penumbra_proto::dex::TradingPair;
use penumbra_proto::{dex as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct Swap {
    // A proof that this is a valid state change.
    pub proof: SpendProof,
    // The encrypted amount of asset 1 to be swapped.
    pub enc_amount_1: MockFlowCiphertext,
    // The encrypted amount of asset 2 to be swapped.
    pub enc_amount_2: MockFlowCiphertext,
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
}

impl Protobuf<pb::Swap> for Swap {}

impl From<Swap> for pb::Swap {
    fn from(s: Swap) -> Self {
        pb::Swap {
            zkproof: s.proof.into(),
            enc_amount_1: Some(s.enc_amount_1.into()),
            enc_amount_2: Some(s.enc_amount_2.into()),
            body: Some(s.body.into()),
            auth_sig: Some(s.auth_sig.into()),
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
            body: s
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body"))?
                .try_into()?,
            auth_sig: s
                .auth_sig
                .ok_or_else(|| anyhow::anyhow!("missing auth_sig"))?
                .try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
// TODO: unsure yet what size needs to be here
pub struct SwapCiphertext([u8; 128]);

#[derive(Debug, Clone)]
pub struct Body {
    pub trading_pair: TradingPair,
    pub ca1: value::Commitment,
    pub ca2: value::Commitment,
    pub cf: value::Commitment,
    pub swap_nft: NotePayload,
    pub swap_ciphertext: SwapCiphertext,
}

impl Protobuf<pb::SwapBody> for Body {}

impl From<Body> for pb::SwapBody {
    fn from(s: Body) -> Self {
        pb::SwapBody {
            trading_pair: s.trading_pair.into(),
            ca1: Bytes::copy_from_slice(&s.ca1),
            ca2: Bytes::copy_from_slice(&s.ca2),
            cf: Bytes::copy_from_slice(&s.cf),
            swap_nft: s.swap_nft.into(),
            swap_ciphertext: s.swap_ciphertext.into(),
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
            ca1: s
                .ca1
                .ok_or_else(|| anyhow::anyhow!("missing ca1"))?
                .try_into()?,
            ca2: s
                .ca2
                .ok_or_else(|| anyhow::anyhow!("missing ca2"))?
                .try_into()?,
            cv: s
                .cf
                .ok_or_else(|| anyhow::anyhow!("missing cf"))?
                .try_into()?,
            swap_nft: s
                .swap_nft
                .ok_or_else(|| anyhow::anyhow!("missing swap_nft"))?
                .try_into()?,
            swap_ciphertext: s
                .swap_ciphertext
                .ok_or_else(|| anyhow::anyhow!("missing swap_ciphertext"))?
                .try_into()?,
        })
    }
}
