use bytes::Bytes;
use penumbra_crypto::{memo::MemoCiphertext, symmetric::PayloadKey};
use penumbra_proto::{core::transaction::v1alpha1 as pb, Protobuf};

use rand::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct MemoPlan {
    pub plaintext: String,
    pub key: PayloadKey,
}

impl MemoPlan {
    /// Create a new [`MemoPlan`].
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        plaintext: String,
    ) -> Result<MemoPlan, anyhow::Error> {
        let key = PayloadKey::random_key(rng);
        Ok(MemoPlan { plaintext, key })
    }

    /// Create a [`MemoCiphertext`] from the [`MemoPlan`].
    pub fn memo(&self) -> Result<MemoCiphertext, anyhow::Error> {
        MemoCiphertext::encrypt(self.key.clone(), &self.plaintext)
    }
}

impl Protobuf<pb::MemoPlan> for MemoPlan {}

impl From<MemoPlan> for pb::MemoPlan {
    fn from(msg: MemoPlan) -> Self {
        Self {
            plaintext: Bytes::copy_from_slice(msg.plaintext.as_ref()),
            key: msg.key.to_vec().into(),
        }
    }
}

impl TryFrom<pb::MemoPlan> for MemoPlan {
    type Error = anyhow::Error;

    fn try_from(msg: pb::MemoPlan) -> Result<Self, Self::Error> {
        let plaintext = String::from_utf8_lossy(&msg.plaintext).to_string();
        let key = PayloadKey::try_from(msg.key.to_vec())?;
        Ok(Self { plaintext, key })
    }
}
