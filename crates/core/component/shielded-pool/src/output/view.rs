use penumbra_sdk_keys::PayloadKey;
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pbt, DomainType};
use serde::{Deserialize, Serialize};

use crate::NoteView;
use crate::Output;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::OutputView", into = "pbt::OutputView")]
#[allow(clippy::large_enum_variant)]
pub enum OutputView {
    Visible {
        output: Output,
        note: NoteView,
        payload_key: PayloadKey,
    },
    Opaque {
        output: Output,
    },
}

impl DomainType for OutputView {
    type Proto = pbt::OutputView;
}

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
                payload_key: x
                    .payload_key
                    .ok_or_else(|| anyhow::anyhow!("missing payload key field"))?
                    .inner
                    .as_slice()
                    .try_into()?,
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
                    payload_key: Some(payload_key.into()),
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

impl From<OutputView> for Output {
    fn from(v: OutputView) -> Self {
        match v {
            OutputView::Visible {
                output,
                payload_key: _,
                note: _,
            } => output,
            OutputView::Opaque { output } => output,
        }
    }
}
