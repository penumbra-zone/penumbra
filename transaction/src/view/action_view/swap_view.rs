use penumbra_crypto::dex::swap::SwapPlaintext;
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::action::Swap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapView", into = "pb::SwapView")]
#[allow(clippy::large_enum_variant)]
pub enum SwapView {
    Visible {
        swap: Swap,
        swap_plaintext: SwapPlaintext,
    },
    Opaque {
        swap: Swap,
    },
}

impl TypeUrl for SwapView {
    const TYPE_URL: &'static str = "/penumbra.core.dex.v1alpha1.SwapView";
}

impl DomainType for SwapView {
    type Proto = pb::SwapView;
}

impl TryFrom<pb::SwapView> for SwapView {
    type Error = anyhow::Error;

    fn try_from(v: pb::SwapView) -> Result<Self, Self::Error> {
        match v
            .swap_view
            .ok_or_else(|| anyhow::anyhow!("missing swap field"))?
        {
            pb::swap_view::SwapView::Visible(x) => Ok(SwapView::Visible {
                swap: x
                    .swap
                    .ok_or_else(|| anyhow::anyhow!("missing swap field"))?
                    .try_into()?,
                swap_plaintext: x
                    .swap_plaintext
                    .ok_or_else(|| anyhow::anyhow!("missing swap plaintext field"))?
                    .try_into()?,
            }),
            pb::swap_view::SwapView::Opaque(x) => Ok(SwapView::Opaque {
                swap: x
                    .swap
                    .ok_or_else(|| anyhow::anyhow!("missing swap field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<SwapView> for pb::SwapView {
    fn from(v: SwapView) -> Self {
        use pb::swap_view as sv;
        match v {
            SwapView::Visible {
                swap,
                swap_plaintext,
            } => Self {
                swap_view: Some(sv::SwapView::Visible(sv::Visible {
                    swap: Some(swap.into()),
                    swap_plaintext: Some(swap_plaintext.into()),
                })),
            },
            SwapView::Opaque { swap } => Self {
                swap_view: Some(sv::SwapView::Opaque(sv::Opaque {
                    swap: Some(swap.into()),
                })),
            },
        }
    }
}

impl From<SwapView> for Swap {
    fn from(v: SwapView) -> Self {
        match v {
            SwapView::Visible {
                swap,
                swap_plaintext: _,
            } => swap,
            SwapView::Opaque { swap } => swap,
        }
    }
}
