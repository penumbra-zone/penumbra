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
        let spend = v
            .spend
            .ok_or_else(|| anyhow::anyhow!("missing spend field"))?
            .try_into()?;

        match v.note {
            Some(note) => Ok(SpendView::Visible {
                spend,
                note: note.try_into()?,
            }),
            None => Ok(SpendView::Opaque { spend }),
        }
    }
}

impl From<SpendView> for pbt::SpendView {
    fn from(v: SpendView) -> Self {
        match v {
            SpendView::Visible { spend, note } => Self {
                spend: Some(spend.into()),
                note: Some(note.into()),
            },
            SpendView::Opaque { spend } => Self {
                spend: Some(spend.into()),
                note: None,
            },
        }
    }
}
