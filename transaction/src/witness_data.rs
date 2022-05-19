use penumbra_proto::{transaction as pb, Protobuf};
use penumbra_tct as tct;

#[derive(Clone, Debug)]
pub struct WitnessData {
    pub anchor: tct::Root,
    pub note_commitment_proofs: Vec<tct::Proof>,
}

impl Protobuf<pb::WitnessData> for WitnessData {}

impl From<WitnessData> for pb::WitnessData {
    fn from(msg: WitnessData) -> Self {
        Self {
            anchor: Some(msg.anchor.into()),
            note_commitment_proofs: msg
                .note_commitment_proofs
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<pb::WitnessData> for WitnessData {
    type Error = anyhow::Error;

    fn try_from(msg: pb::WitnessData) -> Result<Self, Self::Error> {
        Ok(Self {
            anchor: msg
                .anchor
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            note_commitment_proofs: msg
                .note_commitment_proofs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
