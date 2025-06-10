use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::PositionOpen;

use super::PositionMetadata;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionOpenView", into = "pb::PositionOpenView")]
pub enum PositionOpenView {
    Visible {
        action: PositionOpen,
        metadata: PositionMetadata,
    },
    Opaque {
        action: PositionOpen,
    },
}

impl DomainType for PositionOpenView {
    type Proto = pb::PositionOpenView;
}

impl From<PositionOpenView> for pb::PositionOpenView {
    fn from(view: PositionOpenView) -> Self {
        match view {
            PositionOpenView::Visible { action, metadata } => Self {
                position_open_view: Some(pb::position_open_view::PositionOpenView::Visible(
                    pb::position_open_view::Visible {
                        action: Some(action.into()),
                        metadata: Some(metadata.into()),
                    },
                )),
            },
            PositionOpenView::Opaque { action } => Self {
                position_open_view: Some(pb::position_open_view::PositionOpenView::Opaque(
                    pb::position_open_view::Opaque {
                        action: Some(action.into()),
                    },
                )),
            },
        }
    }
}

impl TryFrom<pb::PositionOpenView> for PositionOpenView {
    type Error = anyhow::Error;

    fn try_from(proto: pb::PositionOpenView) -> Result<Self, Self::Error> {
        use anyhow::Context;

        match proto.position_open_view {
            Some(pb::position_open_view::PositionOpenView::Visible(visible)) => {
                let action = visible
                    .action
                    .ok_or_else(|| anyhow::anyhow!("missing action in Visible PositionOpenView"))?
                    .try_into()?;
                let metadata = visible
                    .metadata
                    .ok_or_else(|| anyhow::anyhow!("missing metadata in Visible PositionOpenView"))?
                    .try_into()?;

                Ok(PositionOpenView::Visible { action, metadata })
            }
            Some(pb::position_open_view::PositionOpenView::Opaque(opaque)) => {
                let action = opaque
                    .action
                    .ok_or_else(|| anyhow::anyhow!("missing action in Opaque PositionOpenView"))?
                    .try_into()
                    .context("invalid action in Opaque PositionOpenView")?;

                Ok(PositionOpenView::Opaque { action })
            }
            None => Err(anyhow::anyhow!("missing position_open_view")),
        }
    }
}

impl From<PositionOpenView> for PositionOpen {
    fn from(value: PositionOpenView) -> Self {
        match value {
            PositionOpenView::Visible { action, .. } => action,
            PositionOpenView::Opaque { action } => action,
        }
    }
}
