use penumbra_crypto::{Note, PayloadKey};
use penumbra_proto::{core::transaction::v1alpha1 as pbt, Protobuf};
use serde::{Deserialize, Serialize};

use crate::action::Output;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::OutputView", into = "pbt::OutputView")]
#[allow(clippy::large_enum_variant)]
pub enum OutputView {
    Visible {
        output: Output,
        note: Note,
        payload_key: PayloadKey,
    },
    Opaque {
        output: Output,
    },
}

impl Protobuf<pbt::OutputView> for OutputView {}

impl TryFrom<pbt::OutputView> for OutputView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::OutputView) -> Result<Self, Self::Error> {
        let output = v
            .output
            .ok_or_else(|| anyhow::anyhow!("missing output field"))?
            .try_into()?;

        match v.note {
            Some(note) => Ok(OutputView::Visible {
                output,
                note: note.try_into()?,
                payload_key: v.payload_key.as_ref().try_into()?,
            }),
            None => Ok(OutputView::Opaque { output }),
        }
    }
}

impl From<OutputView> for pbt::OutputView {
    fn from(v: OutputView) -> Self {
        match v {
            OutputView::Visible {
                output,
                note,
                payload_key,
            } => Self {
                output: Some(output.into()),
                note: Some(note.into()),
                payload_key: payload_key.to_vec().into(),
            },
            OutputView::Opaque { output } => Self {
                output: Some(output.into()),
                note: None,
                payload_key: Default::default(),
            },
        }
    }
}
