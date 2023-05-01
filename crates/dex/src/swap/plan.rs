use anyhow::{anyhow, Context, Result};
use ark_ff::{UniformRand, Zero};

use penumbra_crypto::dex::swap::SwapPlaintext;
use penumbra_crypto::Balance;
use penumbra_crypto::{proofs::groth16::SwapProof, FieldExt, Fr, FullViewingKey, Value};
use penumbra_proof_params::SWAP_PROOF_PROVING_KEY;
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType};
use rand_core::{CryptoRng, OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{swap, Swap};

/// A planned [`Swap`](Swap).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapPlan", into = "pb::SwapPlan")]
pub struct SwapPlan {
    pub swap_plaintext: SwapPlaintext,
    pub fee_blinding: Fr,
}

impl SwapPlan {
    /// Create a new [`SwapPlan`] that requests a swap between the given assets and input amounts.
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R, swap_plaintext: SwapPlaintext) -> SwapPlan {
        let fee_blinding = Fr::rand(rng);

        SwapPlan {
            fee_blinding,
            swap_plaintext,
        }
    }

    /// Convenience method to construct the [`Swap`] described by this [`SwapPlan`].
    pub fn swap(&self, fvk: &FullViewingKey) -> Swap {
        Swap {
            body: self.swap_body(fvk),
            proof: self.swap_proof(),
        }
    }

    /// Construct the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_body(&self, fvk: &FullViewingKey) -> swap::Body {
        swap::Body {
            trading_pair: self.swap_plaintext.trading_pair,
            delta_1_i: self.swap_plaintext.delta_1_i,
            delta_2_i: self.swap_plaintext.delta_2_i,
            fee_commitment: self.fee_commitment(),
            payload: self.swap_plaintext.encrypt(fvk.outgoing()),
        }
    }

    /// Construct the [`SwapProof`] required by the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_proof(&self) -> SwapProof {
        let balance_commitment =
            self.transparent_balance().commit(Fr::zero()) + self.fee_commitment();
        SwapProof::prove(
            &mut OsRng,
            &SWAP_PROOF_PROVING_KEY,
            self.swap_plaintext.clone(),
            self.fee_blinding,
            balance_commitment,
            self.swap_plaintext.swap_commitment(),
            self.fee_commitment(),
        )
        .expect("can generate ZKSwapProof")
    }

    pub fn fee_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.swap_plaintext.claim_fee.commit(self.fee_blinding)
    }

    pub fn transparent_balance(&self) -> penumbra_crypto::Balance {
        let value_1 = Value {
            amount: self.swap_plaintext.delta_1_i,
            asset_id: self.swap_plaintext.trading_pair.asset_1(),
        };
        let value_2 = Value {
            amount: self.swap_plaintext.delta_2_i,
            asset_id: self.swap_plaintext.trading_pair.asset_2(),
        };

        let mut balance = Balance::default();
        balance -= value_1;
        balance -= value_2;
        balance
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        // Swaps must have spends corresponding to:
        // - the input amount of asset 1
        // - the input amount of asset 2
        // - the pre-paid swap claim fee
        let value_fee = Value {
            amount: self.swap_plaintext.claim_fee.amount(),
            asset_id: self.swap_plaintext.claim_fee.asset_id(),
        };

        let mut balance = self.transparent_balance();
        balance -= value_fee;
        balance
    }
}

impl DomainType for SwapPlan {
    type Proto = pb::SwapPlan;
}

impl From<SwapPlan> for pb::SwapPlan {
    fn from(msg: SwapPlan) -> Self {
        Self {
            swap_plaintext: Some(msg.swap_plaintext.into()),
            fee_blinding: msg.fee_blinding.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::SwapPlan> for SwapPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapPlan) -> Result<Self, Self::Error> {
        let fee_blinding_bytes: [u8; 32] = msg.fee_blinding[..]
            .try_into()
            .map_err(|_| anyhow!("expected 32 byte fee blinding"))?;
        Ok(Self {
            fee_blinding: Fr::from_bytes(fee_blinding_bytes).context("fee blinding malformed")?,
            swap_plaintext: msg
                .swap_plaintext
                .ok_or_else(|| anyhow!("missing swap_plaintext"))?
                .try_into()
                .context("swap plaintext malformed")?,
        })
    }
}
