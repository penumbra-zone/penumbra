use penumbra_proto::{dex as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::MockFlowCiphertext", into = "pb::MockFlowCiphertext")]
// TODO: should not be a raw u64, needs to be constant-length
pub struct MockFlowCiphertext(u64);

// Fake implementation for now, TODO: replace w/ additively homomorphic encryption impl
impl MockFlowCiphertext {
    pub fn mock_decrypt(&self) -> u64 {
        self.0
    }

    pub fn add(&mut self, amount: u64) {
        self.0 += amount
    }
}

impl Protobuf<pb::MockFlowCiphertext> for MockFlowCiphertext {}

impl From<MockFlowCiphertext> for pb::MockFlowCiphertext {
    fn from(ik: MockFlowCiphertext) -> Self {
        pb::MockFlowCiphertext { value: ik.0 }
    }
}

impl TryFrom<pb::MockFlowCiphertext> for MockFlowCiphertext {
    type Error = anyhow::Error;
    fn try_from(ct: pb::MockFlowCiphertext) -> Result<Self, Self::Error> {
        Ok(Self(ct.value))
    }
}
