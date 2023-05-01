use anyhow::anyhow;
use penumbra_proto::core::dex::v1alpha1 as pb;

use serde::{Deserialize, Serialize};

use crate::FullViewingKey;

use super::{SwapCiphertext, SwapPlaintext};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapPayload", into = "pb::SwapPayload")]
pub struct SwapPayload {
    pub commitment: penumbra_tct::Commitment,
    pub encrypted_swap: SwapCiphertext,
}

impl SwapPayload {
    pub fn trial_decrypt(&self, fvk: &FullViewingKey) -> Option<SwapPlaintext> {
        // Try to decrypt the swap ciphertext. If it doesn't decrypt, it wasn't meant for us.
        let swap = self
            .encrypted_swap
            .decrypt(fvk.outgoing(), self.commitment)
            .ok()?;
        tracing::debug!(swap_commitment = ?self.commitment, ?swap, "found swap while scanning");

        // Before returning, though, we want to perform integrity checks on the
        // swap plaintext, since it could have been sent by unseen overlords of
        // endless deceptive power. One can never be too careful.
        //
        // As in trial_decrypt for notes, we don't want to return errors, to
        // avoid the possibility of "REJECT" style attacks.

        // Check that the swap plaintext matches the swap commitment.
        if swap.swap_commitment() != self.commitment {
            // This should be a warning, because no honestly generated swap plaintext should
            // fail to match the swap commitment actually included in the chain.
            tracing::warn!("decrypted swap does not match provided swap commitment");
            return None;
        }

        // Check that the swap outputs are spendable by this fvk's spending key.
        if !fvk.incoming().views_address(&swap.claim_address) {
            // This should be a warning, because no honestly generated swap plaintext should
            // mismatch the FVK that can detect and decrypt it.
            tracing::warn!("decrypted swap that is not claimable by provided full viewing key");
            return None;
        }

        Some(swap)
    }
}

impl From<SwapPayload> for pb::SwapPayload {
    fn from(msg: SwapPayload) -> Self {
        pb::SwapPayload {
            commitment: Some(msg.commitment.into()),
            encrypted_swap: msg.encrypted_swap.0.to_vec(),
        }
    }
}

impl TryFrom<pb::SwapPayload> for SwapPayload {
    type Error = anyhow::Error;

    fn try_from(msg: pb::SwapPayload) -> Result<Self, Self::Error> {
        let commitment = msg
            .commitment
            .ok_or_else(|| anyhow!("missing commitment"))?
            .try_into()?;
        let encrypted_swap = SwapCiphertext(
            msg.encrypted_swap
                .try_into()
                .map_err(|_| anyhow!("expected correct length swap ciphertext"))?,
        );
        Ok(Self {
            commitment,
            encrypted_swap,
        })
    }
}
