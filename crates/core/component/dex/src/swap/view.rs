use penumbra_sdk_asset::{asset::Metadata, ValueView};
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use penumbra_sdk_shielded_pool::NoteView;
use penumbra_sdk_txhash::TransactionId;
use serde::{Deserialize, Serialize};

use crate::BatchSwapOutputData;

use super::{Swap, SwapPlaintext};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapView", into = "pb::SwapView")]
#[allow(clippy::large_enum_variant)]
pub enum SwapView {
    Visible {
        swap: Swap,
        swap_plaintext: SwapPlaintext,
        output_1: Option<NoteView>,
        output_2: Option<NoteView>,
        claim_tx: Option<TransactionId>,
        asset_1_metadata: Option<Metadata>,
        asset_2_metadata: Option<Metadata>,
        batch_swap_output_data: Option<BatchSwapOutputData>,
    },
    Opaque {
        swap: Swap,
        batch_swap_output_data: Option<BatchSwapOutputData>,
        output_1: Option<ValueView>,
        output_2: Option<ValueView>,
        asset_1_metadata: Option<Metadata>,
        asset_2_metadata: Option<Metadata>,
    },
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
                output_1: x.output_1.map(TryInto::try_into).transpose()?,
                output_2: x.output_2.map(TryInto::try_into).transpose()?,
                claim_tx: x.claim_tx.map(TryInto::try_into).transpose()?,
                asset_1_metadata: x.asset_1_metadata.map(TryInto::try_into).transpose()?,
                asset_2_metadata: x.asset_2_metadata.map(TryInto::try_into).transpose()?,
                batch_swap_output_data: x
                    .batch_swap_output_data
                    .map(TryInto::try_into)
                    .transpose()?,
            }),
            pb::swap_view::SwapView::Opaque(x) => Ok(SwapView::Opaque {
                swap: x
                    .swap
                    .ok_or_else(|| anyhow::anyhow!("missing swap field"))?
                    .try_into()?,
                batch_swap_output_data: x
                    .batch_swap_output_data
                    .map(TryInto::try_into)
                    .transpose()?,
                output_1: x.output_1_value.map(TryInto::try_into).transpose()?,
                output_2: x.output_2_value.map(TryInto::try_into).transpose()?,
                asset_1_metadata: x.asset_1_metadata.map(TryInto::try_into).transpose()?,
                asset_2_metadata: x.asset_2_metadata.map(TryInto::try_into).transpose()?,
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
                output_1,
                output_2,
                claim_tx,
                asset_1_metadata,
                asset_2_metadata,
                batch_swap_output_data,
            } => Self {
                swap_view: Some(sv::SwapView::Visible(sv::Visible {
                    swap: Some(swap.into()),
                    swap_plaintext: Some(swap_plaintext.into()),
                    output_1: output_1.map(Into::into),
                    output_2: output_2.map(Into::into),
                    claim_tx: claim_tx.map(Into::into),
                    asset_1_metadata: asset_1_metadata.map(Into::into),
                    asset_2_metadata: asset_2_metadata.map(Into::into),
                    batch_swap_output_data: batch_swap_output_data.map(Into::into),
                })),
            },
            SwapView::Opaque {
                swap,
                batch_swap_output_data,
                output_1,
                output_2,
                asset_1_metadata,
                asset_2_metadata,
            } => Self {
                swap_view: Some(sv::SwapView::Opaque(sv::Opaque {
                    swap: Some(swap.into()),
                    batch_swap_output_data: batch_swap_output_data.map(Into::into),
                    output_1_value: output_1.map(Into::into),
                    output_2_value: output_2.map(Into::into),
                    asset_1_metadata: asset_1_metadata.map(Into::into),
                    asset_2_metadata: asset_2_metadata.map(Into::into),
                })),
            },
        }
    }
}

impl From<SwapView> for Swap {
    fn from(v: SwapView) -> Self {
        match v {
            SwapView::Visible { swap, .. } => swap,
            SwapView::Opaque { swap, .. } => swap,
        }
    }
}
