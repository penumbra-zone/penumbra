use anyhow::Error;
use decaf377_fmd::Clue;
use penumbra_proto::core::transaction::v1alpha1 as pbt;
use penumbra_proto::DomainType;
use penumbra_txhash::{EffectHash, EffectingData};

/// Detection data used by a detection server using Fuzzy Message Detection.
///
/// Only present if outputs are present.
#[derive(Clone, Debug, Default)]
pub struct DetectionData {
    pub fmd_clues: Vec<Clue>,
}

impl EffectingData for DetectionData {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for DetectionData {
    type Proto = pbt::DetectionData;
}

impl TryFrom<pbt::DetectionData> for DetectionData {
    type Error = Error;

    fn try_from(proto: pbt::DetectionData) -> anyhow::Result<Self, Self::Error> {
        let fmd_clues = proto
            .fmd_clues
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<Clue>, Error>>()?;
        Ok(DetectionData { fmd_clues })
    }
}

impl From<DetectionData> for pbt::DetectionData {
    fn from(msg: DetectionData) -> Self {
        let fmd_clues = msg.fmd_clues.into_iter().map(|x| x.into()).collect();

        pbt::DetectionData { fmd_clues }
    }
}
