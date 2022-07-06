use std::ops::Add;

use penumbra_proto::{dex as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::MockFlowCiphertext", into = "pb::MockFlowCiphertext")]
pub struct MockFlowCiphertext(u64);

// Fake implementation for now, TODO: replace w/ additively homomorphic encryption impl
// once Eddy impl available
impl MockFlowCiphertext {
    pub fn new(plaintext: u64) -> Self {
        // TODO: do encryption stuff here
        Self(plaintext)
    }

    pub fn mock_decrypt(&self) -> u64 {
        // TODO: do decryption stuff here
        self.0
    }
}

impl Add for MockFlowCiphertext {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
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
