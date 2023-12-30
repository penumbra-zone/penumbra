use anyhow::Error;
use penumbra_fee::Fee;
use penumbra_proto::core::transaction::v1alpha1 as pbt;
use penumbra_proto::DomainType;
use penumbra_txhash::{EffectHash, EffectingData};

/// Parameters determining when the transaction should be accepted to the chain.
#[derive(Clone, Debug, Default)]
pub struct TransactionParameters {
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
}

impl EffectingData for TransactionParameters {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for TransactionParameters {
    type Proto = pbt::TransactionParameters;
}

impl TryFrom<pbt::TransactionParameters> for TransactionParameters {
    type Error = Error;

    fn try_from(proto: pbt::TransactionParameters) -> anyhow::Result<Self, Self::Error> {
        Ok(TransactionParameters {
            expiry_height: proto.expiry_height,
            chain_id: proto.chain_id,
            fee: proto
                .fee
                .ok_or_else(|| anyhow::anyhow!("transaction parameters missing fee"))?
                .try_into()?,
        })
    }
}

impl From<TransactionParameters> for pbt::TransactionParameters {
    fn from(msg: TransactionParameters) -> Self {
        pbt::TransactionParameters {
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
            fee: Some(msg.fee.into()),
        }
    }
}
