use penumbra_crypto::merkle::{self, AuthPath};
use penumbra_proto::{transaction as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct WitnessData {
    pub anchor: merkle::Root,
    pub auth_paths: Vec<AuthPath>,
}

impl Protobuf<pb::WitnessData> for WitnessData {}

impl From<WitnessData> for pb::WitnessData {
    fn from(msg: WitnessData) -> Self {
        Self {
            anchor: Some(msg.anchor.into()),
            auth_paths: msg.auth_paths.into_iter().map(Into::into).collect(),
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
            auth_paths: msg
                .auth_paths
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
