use bytes::Bytes;
use penumbra_crypto::{
    memo::{MemoCiphertext, MemoPlaintext, MEMO_LEN_BYTES},
    symmetric::PayloadKey,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, Protobuf};

use rand::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct MemoPlan {
    pub plaintext: MemoPlaintext,
    pub key: PayloadKey,
}

impl MemoPlan {
    /// Create a new [`MemoPlan`].
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R, plaintext: MemoPlaintext) -> MemoPlan {
        let key = PayloadKey::random_key(rng);
        MemoPlan { plaintext, key }
    }

    /// Create a [`MemoCiphertext`] from the [`MemoPlan`].
    pub fn memo(&self) -> MemoCiphertext {
        self.plaintext.encrypt(self.key.clone())
    }
}

impl Protobuf<pb::MemoPlan> for MemoPlan {}

impl From<MemoPlan> for pb::MemoPlan {
    fn from(msg: MemoPlan) -> Self {
        Self {
            plaintext: Bytes::copy_from_slice(&msg.plaintext.0),
            key: msg.key.to_vec().into(),
        }
    }
}

impl TryFrom<pb::MemoPlan> for MemoPlan {
    type Error = anyhow::Error;

    fn try_from(msg: pb::MemoPlan) -> Result<Self, Self::Error> {
        let plaintext_bytes: [u8; MEMO_LEN_BYTES] = msg.plaintext.as_ref().try_into()?;
        Ok(Self {
            plaintext: MemoPlaintext(plaintext_bytes),
            key: PayloadKey::try_from(msg.key.to_vec())?,
        })
    }
}
