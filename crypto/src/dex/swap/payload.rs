use anyhow::anyhow;
use penumbra_proto::{core::crypto::v1alpha1 as pb_crypto, core::dex::v1alpha1 as pb, Protobuf};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

use crate::ka;

use super::SwapCiphertext;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapPayload", into = "pb::SwapPayload")]
pub struct SwapPayload {
    pub commitment: penumbra_tct::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_swap: SwapCiphertext,
}

impl From<SwapPayload> for pb::SwapPayload {
    fn from(msg: SwapPayload) -> Self {
        pb::SwapPayload {
            commitment: Some(msg.commitment.into()),
            ephemeral_key: msg.ephemeral_key.0.into(),
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
        let ephemeral_key = ka::Public(
            msg.ephemeral_key
                .try_into()
                .map_err(|_| anyhow!("expected 32 byte ephemeral key"))?,
        );
        let encrypted_swap = SwapCiphertext(
            msg.encrypted_swap
                .try_into()
                .map_err(|_| anyhow!("expected correct length swap ciphertext"))?,
        );
        Ok(Self {
            commitment,
            ephemeral_key,
            encrypted_swap,
        })
    }
}
