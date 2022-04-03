use penumbra_proto::{ibc as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct IBCAction {
    pub action: pb::ibc_action::Action,
}

impl Protobuf<pb::IbcAction> for IBCAction {}

impl From<IBCAction> for pb::IbcAction {
    fn from(i: IBCAction) -> Self {
        pb::IbcAction {
            action: Some(i.action),
        }
    }
}

impl TryFrom<pb::IbcAction> for IBCAction {
    type Error = anyhow::Error;
    fn try_from(d: pb::IbcAction) -> Result<Self, Self::Error> {
        Ok(Self {
            action: d
                .action
                .ok_or(anyhow::Error::msg("IBCAction.action is missing"))?,
        })
    }
}
