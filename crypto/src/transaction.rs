use blake2b_simd::Hash;

use penumbra_proto::{core::crypto::v1alpha1 as pb, Protobuf};

use crate::{asset, balance, Balance, Fr, Value, STAKING_TOKEN_ASSET_ID};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fee(pub Value);

impl Default for Fee {
    fn default() -> Self {
        Fee::from_staking_token_amount(asset::Amount::zero())
    }
}

impl Fee {
    pub fn from_staking_token_amount(amount: asset::Amount) -> Self {
        Self(Value {
            amount,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        })
    }

    pub fn amount(&self) -> asset::Amount {
        self.0.amount
    }

    pub fn asset_id(&self) -> asset::Id {
        self.0.asset_id
    }

    pub fn balance(&self) -> balance::Balance {
        -Balance::from(self.0)
    }

    pub fn commit(&self, blinding: Fr) -> balance::Commitment {
        self.balance().commit(blinding)
    }

    pub fn format(&self, cache: &asset::Cache) -> String {
        self.0.format(cache)
    }
}

impl Protobuf<pb::Fee> for Fee {}

impl From<Fee> for pb::Fee {
    fn from(fee: Fee) -> Self {
        if fee.0.asset_id == *STAKING_TOKEN_ASSET_ID {
            pb::Fee {
                amount: Some(fee.0.amount.into()),
                asset_id: None,
            }
        } else {
            pb::Fee {
                amount: Some(fee.0.amount.into()),
                asset_id: Some(fee.0.asset_id.into()),
            }
        }
    }
}

impl TryFrom<pb::Fee> for Fee {
    type Error = anyhow::Error;

    fn try_from(proto: pb::Fee) -> anyhow::Result<Self> {
        if proto.asset_id.is_some() {
            Ok(Fee(Value {
                amount: proto.amount.unwrap().try_into()?,
                asset_id: proto.asset_id.unwrap().try_into()?,
            }))
        } else {
            Ok(Fee(Value {
                amount: proto.amount.unwrap().try_into()?,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            }))
        }
    }
}

impl Fee {
    pub fn auth_hash(&self) -> Hash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:fee")
            .to_state();
        state.update(&self.0.amount.to_le_bytes());
        state.update(&self.0.asset_id.to_bytes());

        state.finalize()
    }

    pub fn value(&self) -> Value {
        self.0
    }
}
