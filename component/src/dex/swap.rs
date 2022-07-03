use crate::ka;
use penumbra_proto::{dex as pb, Protobuf};
use transaction::Fee;

pub struct SwapPlaintext {
    // Amount of asset 1
    pub t1: u64,
    // Amount of asset 2
    pub t2: u64,
    // Fee
    pub fee: Fee,
    // Diversified basepoint
    pub b_d: decaf377::Element,
    // Diversified public key
    pub pk_d: ka::Public,
}

impl Protobuf<pb::SwapPlaintext> for SwapPlaintext {}

impl TryFrom<pb::SwapPlaintext> for SwapPlaintext {
    type Error = anyhow::Error;
    fn try_from(plaintext: pb::SwapPlaintext) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl From<SwapPlaintext> for pb::SwapPlaintext {
    fn from(plaintext: SwapPlaintext) -> Self {
        Self {}
    }
}
