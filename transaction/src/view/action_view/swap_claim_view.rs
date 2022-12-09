use penumbra_crypto::Note;
use penumbra_proto::{core::dex::v1alpha1 as pbd, Protobuf};
use serde::{Deserialize, Serialize};

use crate::action::SwapClaim;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbd::SwapClaimView", into = "pbd::SwapClaimView")]
#[allow(clippy::large_enum_variant)]
pub enum SwapClaimView {
    Visible {
        swap_claim: SwapClaim,
        output_1: Note,
        output_2: Note,
    },
    Opaque {
        swap_claim: SwapClaim,
    },
}

impl Protobuf<pbd::SwapClaimView> for SwapClaimView {}

impl TryFrom<pbd::SwapClaimView> for SwapClaimView {
    type Error = anyhow::Error;

    fn try_from(v: pbd::SwapClaimView) -> Result<Self, Self::Error> {
        let swap_claim = v
            .swap_claim
            .ok_or_else(|| anyhow::anyhow!("missing swap_claim field"))?
            .try_into()?;

        match (v.output_1, v.output_2) {
            (Some(output_1), Some(output_2)) => Ok(SwapClaimView::Visible {
                swap_claim,
                output_1: output_1.try_into()?,
                output_2: output_2.try_into()?,
            }),
            (None, None) => Ok(SwapClaimView::Opaque { swap_claim }),
            _ => Err(anyhow::anyhow!("malformed swap_claim view")),
        }
    }
}

impl From<SwapClaimView> for pbd::SwapClaimView {
    fn from(v: SwapClaimView) -> Self {
        match v {
            SwapClaimView::Visible {
                swap_claim,
                output_1,
                output_2,
            } => Self {
                swap_claim: Some(swap_claim.into()),
                output_1: Some(output_1.into()),
                output_2: Some(output_2.into()),
            },
            SwapClaimView::Opaque { swap_claim } => Self {
                swap_claim: Some(swap_claim.into()),
                output_1: None,
                output_2: None,
            },
        }
    }
}
