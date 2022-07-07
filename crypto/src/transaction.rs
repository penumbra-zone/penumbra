use blake2b_simd::Hash;

use penumbra_proto::{transaction as pbt, Protobuf};

#[derive(Clone, Debug)]
pub struct Fee(pub u64);

impl Protobuf<pbt::Fee> for Fee {}

impl From<Fee> for pbt::Fee {
    fn from(fee: Fee) -> Self {
        pbt::Fee { amount: fee.0 }
    }
}

impl From<pbt::Fee> for Fee {
    fn from(proto: pbt::Fee) -> Self {
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
