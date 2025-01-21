use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pbd, DomainType};
use penumbra_sdk_shielded_pool::NoteView;
use penumbra_sdk_txhash::TransactionId;
use serde::{Deserialize, Serialize};

use super::SwapClaim;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbd::SwapClaimView", into = "pbd::SwapClaimView")]
#[allow(clippy::large_enum_variant)]
pub enum SwapClaimView {
    Visible {
        swap_claim: SwapClaim,
        output_1: NoteView,
        output_2: NoteView,
        swap_tx: Option<TransactionId>,
    },
    Opaque {
        swap_claim: SwapClaim,
    },
}

impl DomainType for SwapClaimView {
    type Proto = pbd::SwapClaimView;
}

impl TryFrom<pbd::SwapClaimView> for SwapClaimView {
    type Error = anyhow::Error;

    fn try_from(v: pbd::SwapClaimView) -> Result<Self, Self::Error> {
        match v
            .swap_claim_view
            .ok_or_else(|| anyhow::anyhow!("missing swap field"))?
        {
            pbd::swap_claim_view::SwapClaimView::Visible(x) => Ok(SwapClaimView::Visible {
                swap_claim: x
                    .swap_claim
                    .ok_or_else(|| anyhow::anyhow!("missing swap claim field"))?
                    .try_into()?,
                output_1: x
                    .output_1
                    .ok_or_else(|| anyhow::anyhow!("missing output_1 field"))?
                    .try_into()?,
                output_2: x
                    .output_2
                    .ok_or_else(|| anyhow::anyhow!("missing output_2 field"))?
                    .try_into()?,
                swap_tx: x.swap_tx.map(TryInto::try_into).transpose()?,
            }),
            pbd::swap_claim_view::SwapClaimView::Opaque(x) => Ok(SwapClaimView::Opaque {
                swap_claim: x
                    .swap_claim
                    .ok_or_else(|| anyhow::anyhow!("missing swap claim field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<SwapClaimView> for pbd::SwapClaimView {
    fn from(v: SwapClaimView) -> Self {
        use pbd::swap_claim_view as scv;
        match v {
            SwapClaimView::Visible {
                swap_claim,
                output_1,
                output_2,
                swap_tx,
            } => Self {
                swap_claim_view: Some(scv::SwapClaimView::Visible(scv::Visible {
                    swap_claim: Some(swap_claim.into()),
                    output_1: Some(output_1.into()),
                    output_2: Some(output_2.into()),
                    swap_tx: swap_tx.map(Into::into),
                })),
            },
            SwapClaimView::Opaque { swap_claim } => Self {
                swap_claim_view: Some(scv::SwapClaimView::Opaque(scv::Opaque {
                    swap_claim: Some(swap_claim.into()),
                })),
            },
        }
    }
}

impl From<SwapClaimView> for SwapClaim {
    fn from(v: SwapClaimView) -> Self {
        match v {
            SwapClaimView::Visible {
                swap_claim,
                output_1: _,
                output_2: _,
                swap_tx: _,
            } => swap_claim,
            SwapClaimView::Opaque { swap_claim } => swap_claim,
        }
    }
}
