use std::ops::{Add, AddAssign, Deref, DerefMut};

use penumbra_proto::{core::dex::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl Default for MockFlowCiphertext {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Add for MockFlowCiphertext {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for MockFlowCiphertext {
    fn add_assign(&mut self, other: Self) {
        *self = self.clone() + other;
    }
}

impl Protobuf for MockFlowCiphertext {
    type Proto = pb::MockFlowCiphertext;
}

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

// Tuple represents:
// ((amount of asset 1 being exchanged for asset 2),
//  (amount of asset 2 being exchanged for asset 1))
#[derive(Default, Clone)]
pub struct SwapFlow((MockFlowCiphertext, MockFlowCiphertext));

impl Deref for SwapFlow {
    type Target = (MockFlowCiphertext, MockFlowCiphertext);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SwapFlow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
