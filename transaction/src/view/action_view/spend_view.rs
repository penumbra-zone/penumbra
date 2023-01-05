use penumbra_crypto::Note;
use penumbra_proto::{core::transaction::v1alpha1 as pbt, Protobuf};
use serde::{Deserialize, Serialize};

use crate::action::Spend;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::SpendView", into = "pbt::SpendView")]
#[allow(clippy::large_enum_variant)]
pub enum SpendView {
    Visible { spend: Spend, note: Note },
    Opaque { spend: Spend },
}

impl Protobuf<pbt::SpendView> for SpendView {}

impl TryFrom<pbt::SpendView> for SpendView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::SpendView) -> Result<Self, Self::Error> {
        match v
            .spend_view
            .ok_or_else(|| anyhow::anyhow!("missing spend field"))?
        {
            pbt::spend_view::SpendView::Visible(x) => Ok(SpendView::Visible {
                spend: x
                    .spend
                    .ok_or_else(|| anyhow::anyhow!("missing spend field"))?
                    .try_into()?,
                note: x
                    .note
                    .ok_or_else(|| anyhow::anyhow!("missing note field"))?
                    .try_into()?,
            }),
            pbt::spend_view::SpendView::Opaque(x) => Ok(SpendView::Opaque {
                spend: x
                    .spend
                    .ok_or_else(|| anyhow::anyhow!("missing spend field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<SpendView> for pbt::SpendView {
    fn from(v: SpendView) -> Self {
        use pbt::spend_view as sv;
        match v {
            SpendView::Visible { spend, note } => Self {
                spend_view: Some(sv::SpendView::Visible(sv::Visible {
                    spend: Some(spend.into()),
                    note: Some(note.into()),
                })),
            },
            SpendView::Opaque { spend } => Self {
                spend_view: Some(sv::SpendView::Opaque(sv::Opaque {
                    spend: Some(spend.into()),
                })),
            },
        }
    }
}
