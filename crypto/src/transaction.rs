use blake2b_simd::Hash;

use penumbra_proto::{crypto as pb, Protobuf};

#[derive(Clone, Debug)]
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
}
