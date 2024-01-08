use crate::DetectionData;

use super::CluePlan;
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::DetectionDataPlan", into = "pb::DetectionDataPlan")]
pub struct DetectionDataPlan {
    pub clue_plans: Vec<CluePlan>,
}

impl DetectionDataPlan {
    pub fn detection_data(&self) -> DetectionData {
        DetectionData {
            fmd_clues: self.clue_plans.iter().map(|x| x.clue()).collect::<Vec<_>>(),
        }
    }
}

impl TryFrom<pb::DetectionDataPlan> for DetectionDataPlan {
    type Error = anyhow::Error;
    fn try_from(value: pb::DetectionDataPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            clue_plans: value
                .clue_plans
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<DetectionDataPlan> for pb::DetectionDataPlan {
    fn from(msg: DetectionDataPlan) -> Self {
        Self {
            clue_plans: msg.clue_plans.into_iter().map(Into::into).collect(),
        }
    }
}

impl DomainType for DetectionDataPlan {
    type Proto = pb::DetectionDataPlan;
}
