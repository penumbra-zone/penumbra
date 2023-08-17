use anyhow::anyhow;
use penumbra_keys::FullViewingKey;
use penumbra_proto::core::dex::v1alpha1 as pb;
use serde::{Deserialize, Serialize};

use super::{SwapClaimCiphertext, SwapClaimPlaintext};

use crate::swap_claim::SwapClaim;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapClaimPayload", into = "pb::SwapClaimPayload")]
pub struct SwapClaimPayload {
    pub commitment: penumbra_tct::StateCommitment,
    pub encrypted_swap: SwapClaimCiphertext,
}

impl SwapClaimPayload {
    pub fn trial_decrypt(&self, fvk: &FullViewingKey) -> Option<SwapClaim> {
        unimplemented!()
    }
}
