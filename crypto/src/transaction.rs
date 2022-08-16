use blake2b_simd::Hash;

use penumbra_proto::{crypto as pb, Protobuf};

use crate::{Value, STAKING_TOKEN_ASSET_ID};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fee(pub u64);

impl Protobuf<pb::Fee> for Fee {}

impl From<Fee> for pb::Fee {
    fn from(fee: Fee) -> Self {
        pb::Fee { amount: fee.0 }
    }
}

impl From<pb::Fee> for Fee {
    fn from(proto: pb::Fee) -> Self {
        Fee(proto.amount)
    }
}

impl Fee {
    pub fn auth_hash(&self) -> Hash {
        blake2b_simd::Params::default()
            .personal(b"PAH:fee")
            .hash(&self.0.to_le_bytes())
    }

    pub fn value(&self) -> Value {
        Value {
            amount: self.0,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        }
    }
}
