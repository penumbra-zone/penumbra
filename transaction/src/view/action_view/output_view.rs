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
        match v
            .output_view
            .ok_or_else(|| anyhow::anyhow!("missing output field"))?
        {
            pbt::output_view::OutputView::Visible(x) => Ok(OutputView::Visible {
                output: x
                    .output
                    .ok_or_else(|| anyhow::anyhow!("missing output field"))?
                    .try_into()?,
                note: x
                    .note
                    .ok_or_else(|| anyhow::anyhow!("missing note field"))?
                    .try_into()?,
                payload_key: x.payload_key.as_ref().try_into()?,
            }),
            pbt::output_view::OutputView::Opaque(x) => Ok(OutputView::Opaque {
                output: x
                    .output
                    .ok_or_else(|| anyhow::anyhow!("missing output field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<OutputView> for pbt::OutputView {
    fn from(v: OutputView) -> Self {
        use pbt::output_view as ov;
        match v {
            OutputView::Visible {
                output,
                note,
                payload_key,
            } => Self {
                output_view: Some(ov::OutputView::Visible(ov::Visible {
                    output: Some(output.into()),
                    note: Some(note.into()),
                    payload_key: payload_key.to_vec().into(),
                })),
            },
            OutputView::Opaque { output } => Self {
                output_view: Some(ov::OutputView::Opaque(ov::Opaque {
                    output: Some(output.into()),
                })),
            },
        }
    }
}
