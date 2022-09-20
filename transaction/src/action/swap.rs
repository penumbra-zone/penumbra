use ark_ff::Zero;
use decaf377::{FieldExt, Fr};
use penumbra_crypto::dex::TradingPair;
use penumbra_crypto::proofs::transparent::SwapProof;
use penumbra_crypto::symmetric::OvkWrappedKey;
use penumbra_crypto::{dex::swap::SwapCiphertext, value};
use penumbra_crypto::{NotePayload, Value};
use penumbra_proto::{dex as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct Swap {
    // A proof that this is a valid state change.
    pub proof: SwapProof,
    // Amounts will be plaintext until flow encryption is available.
    // // The encrypted amount of asset 1 to be swapped.
    // pub enc_amount_1: MockFlowCiphertext,
    // // The encrypted amount of asset 2 to be swapped.
    // pub enc_amount_2: MockFlowCiphertext,
    pub body: Body,
}

impl Swap {
    /// Compute a commitment to the value contributed to a transaction by this swap.
    /// Will subtract (v1,t1), (v2,t2), and (f,fee_token)
    pub fn value_commitment(&self) -> value::Commitment {
        let input_1 = Value {
            amount: self.body.delta_1,
            asset_id: self.body.trading_pair.asset_1(),
        }
        .commit(Fr::zero());
        let input_2 = Value {
            amount: self.body.delta_2,
            asset_id: self.body.trading_pair.asset_2(),
        }
        .commit(Fr::zero());

        -(input_1 + input_2 + self.body.fee_commitment)
    }
}

impl Protobuf<pb::Swap> for Swap {}

impl From<Swap> for pb::Swap {
    fn from(s: Swap) -> Self {
        pb::Swap {
            proof: s.proof.into(),
            body: Some(s.body.into()),
        }
    }
}

impl TryFrom<pb::Swap> for Swap {
    type Error = anyhow::Error;
    fn try_from(s: pb::Swap) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: s.proof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Swap proof malformed"))?,
            body: s
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body"))?
                .try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub trading_pair: TradingPair,
    // No commitments for the values, as they're plaintext
    // until flow encryption is available
    // pub asset_1_commitment: value::Commitment,
    // pub asset_2_commitment: value::Commitment,
    pub delta_1: u64,
    pub delta_2: u64,
    pub fee_commitment: value::Commitment,
    pub fee_blinding: Fr,
    // TODO: rename to note_payload
    pub swap_nft: NotePayload,
    pub swap_ciphertext: SwapCiphertext,
    // ESK used to encrypt the swap ciphertext/swap NFT NotePayload
    // pub ovk_wrapped_key: OvkWrappedKey,
    // TODO: do we need to put the wrapped memo key in here similar to Output?
}

impl Protobuf<pb::SwapBody> for Body {}

impl From<Body> for pb::SwapBody {
    fn from(s: Body) -> Self {
        pb::SwapBody {
            trading_pair: Some(s.trading_pair.into()),
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            fee_blinding: s.fee_blinding.to_bytes().to_vec(),
            fee_commitment: s.fee_commitment.to_bytes().to_vec(),
            swap_nft: Some(s.swap_nft.into()),
            swap_ciphertext: s.swap_ciphertext.0.to_vec(),
        }
    }
}

impl TryFrom<pb::SwapBody> for Body {
    type Error = anyhow::Error;
    fn try_from(s: pb::SwapBody) -> Result<Self, Self::Error> {
        let fee_blinding_bytes: [u8; 32] = s.fee_blinding[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("proto malformed"))?;
        // let ovk_wrapped_key: OvkWrappedKey = proto.ovk_wrapped_key[..]
        //     .try_into()
        //     .map_err(|_| anyhow::anyhow!("output malformed"))?;

        Ok(Self {
            trading_pair: s
                .trading_pair
                .ok_or_else(|| anyhow::anyhow!("missing trading_pair"))?
                .try_into()?,
            delta_1: s.delta_1,
            delta_2: s.delta_2,
            fee_commitment: (&s.fee_commitment[..]).try_into()?,
            swap_nft: s
                .swap_nft
                .ok_or_else(|| anyhow::anyhow!("missing swap_nft"))?
                .try_into()?,
            swap_ciphertext: (&s.swap_ciphertext[..]).try_into()?,
            fee_blinding: Fr::from_bytes(fee_blinding_bytes)?,
            // ovk_wrapped_key,
        })
    }
}
