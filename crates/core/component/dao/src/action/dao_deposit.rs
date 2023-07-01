use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use penumbra_asset::{Balance, Value};
use penumbra_crypto::{EffectHash, EffectingData};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType, TypeUrl};

#[derive(Clone, Debug)]
pub struct DaoDeposit {
    pub value: Value,
}

impl EffectingData for DaoDeposit {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:daodeposit")
            .to_state();

        state.update(&self.value.amount.to_le_bytes());
        state.update(&self.value.asset_id.to_bytes());

        EffectHash(state.finalize().as_array().clone())
    }
}

impl DaoDeposit {
    pub fn balance(&self) -> Balance {
        // Deposits into the DAO require value
        -Balance::from(self.value)
    }
}

impl TypeUrl for DaoDeposit {
    const TYPE_URL: &'static str = "/penumbra.core.governance.v1alpha1.DaoDeposit";
}

impl DomainType for DaoDeposit {
    type Proto = pb::DaoDeposit;
}

impl From<DaoDeposit> for pb::DaoDeposit {
    fn from(msg: DaoDeposit) -> Self {
        pb::DaoDeposit {
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<pb::DaoDeposit> for DaoDeposit {
    type Error = Error;

    fn try_from(proto: pb::DaoDeposit) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;

        Ok(DaoDeposit { value })
    }
}
