use anyhow::Context;
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_crypto::asset::Amount;
use penumbra_crypto::dex::swap::SwapPayload;
use penumbra_crypto::dex::TradingPair;
use penumbra_crypto::proofs::transparent::SwapProof;
use penumbra_crypto::Value;
use penumbra_crypto::{balance, dex::swap::SwapCiphertext, Balance};
use penumbra_proto::{core::dex::v1alpha1 as pb, Protobuf};

use crate::view::action_view::SwapView;
use crate::{ActionView, IsAction, TransactionPerspective};

#[derive(Clone, Debug)]
pub struct Swap {
    pub proof: SwapProof,
    pub body: Body,
}

impl IsAction for Swap {
    /// Compute a commitment to the value contributed to a transaction by this swap.
    /// Will subtract (v1,t1), (v2,t2), and (f,fee_token)
    fn balance_commitment(&self) -> balance::Commitment {
        let input_1 = Value {
            amount: self.body.delta_1_i,
            asset_id: self.body.trading_pair.asset_1(),
        };
        let input_1 = -Balance::from(input_1);
        let commitment_input_1 = input_1.commit(Fr::zero());
        let input_2 = Value {
            amount: self.body.delta_2_i,
            asset_id: self.body.trading_pair.asset_2(),
        };
        let input_2 = -Balance::from(input_2);
        let commitment_input_2 = input_2.commit(Fr::zero());

        commitment_input_1 + commitment_input_2 + self.body.fee_commitment
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        let commitment = self.body.payload.commitment;

        let plaintext = txp.payload_keys.get(&commitment).and_then(|payload_key| {
            // Decrypt swap ciphertext
            SwapCiphertext::decrypt_with_payload_key(&self.body.payload.encrypted_swap, payload_key)
                .ok()
        });

        ActionView::Swap(match plaintext {
            Some(swap_plaintext) => SwapView::Visible {
                swap: self.to_owned(),
                swap_plaintext,
            },
            None => SwapView::Opaque {
                swap: self.to_owned(),
            },
        })
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
            proof: s.proof[..].try_into().context("swap proof malformed")?,
            body: s
                .body
                .ok_or_else(|| anyhow::anyhow!("missing swap body"))?
                .try_into()
                .context("swap body malformed")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub trading_pair: TradingPair,
    pub delta_1_i: Amount,
    pub delta_2_i: Amount,
    pub fee_commitment: balance::Commitment,
    pub payload: SwapPayload,
}

impl Protobuf<pb::SwapBody> for Body {}

impl From<Body> for pb::SwapBody {
    fn from(s: Body) -> Self {
        pb::SwapBody {
            trading_pair: Some(s.trading_pair.into()),
            delta_1_i: Some(s.delta_1_i.into()),
            delta_2_i: Some(s.delta_2_i.into()),
            fee_commitment: s.fee_commitment.to_bytes().to_vec(),
            payload: Some(s.payload.into()),
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

            delta_1_i: s
                .delta_1_i
                .ok_or_else(|| anyhow::anyhow!("missing delta_1"))?
                .try_into()?,
            delta_2_i: s
                .delta_2_i
                .ok_or_else(|| anyhow::anyhow!("missing delta_2"))?
                .try_into()?,

            fee_commitment: (&s.fee_commitment[..]).try_into()?,
            payload: s
                .payload
                .ok_or_else(|| anyhow::anyhow!("missing payload"))?
                .try_into()?,
        })
    }
}
