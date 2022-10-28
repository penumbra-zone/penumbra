use penumbra_crypto::{dex::swap::SwapPlaintext, Note};
use penumbra_proto::{core::transaction::v1alpha1 as pbt, Protobuf};
use serde::{Deserialize, Serialize};

use crate::action::Swap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::SwapView", into = "pbt::SwapView")]
#[allow(clippy::large_enum_variant)]
pub enum SwapView {
    Visible {
        swap: Swap,
        swap_nft: Note,
        swap_plaintext: SwapPlaintext,
    },
    Opaque {
        swap: Swap,
    },
}

impl Protobuf<pbt::SwapView> for SwapView {}

impl TryFrom<pbt::SwapView> for SwapView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::SwapView) -> Result<Self, Self::Error> {
        let swap = v
            .swap
            .ok_or_else(|| anyhow::anyhow!("missing swap field"))?
            .try_into()?;

        match (v.swap_plaintext, v.swap_nft) {
            (Some(swap_plaintext), Some(swap_nft)) => Ok(SwapView::Visible {
                swap,
                swap_nft: swap_nft.try_into()?,
                swap_plaintext: swap_plaintext.try_into()?,
            }),
            (None, None) => Ok(SwapView::Opaque { swap }),
            _ => Err(anyhow::anyhow!("malformed swap view")),
        }
    }
}

impl From<SwapView> for pbt::SwapView {
    fn from(v: SwapView) -> Self {
        match v {
            SwapView::Visible {
                swap,
                swap_nft,
                swap_plaintext,
            } => Self {
                swap: Some(swap.into()),
                swap_nft: Some(swap_nft.into()),
                swap_plaintext: Some(swap_plaintext.into()),
            },
            SwapView::Opaque { swap } => Self {
                swap: Some(swap.into()),
                swap_nft: None,
                swap_plaintext: None,
            },
        }
    }
}
