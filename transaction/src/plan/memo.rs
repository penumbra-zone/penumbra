use penumbra_crypto::{
    memo::{MemoCiphertext, MemoPlaintext},
    symmetric::PayloadKey,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType};

use rand::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct MemoPlan {
    pub plaintext: MemoPlaintext,
    pub key: PayloadKey,
}

impl MemoPlan {
    /// Create a new [`MemoPlan`].
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        plaintext: MemoPlaintext,
    ) -> Result<MemoPlan, anyhow::Error> {
        let key = PayloadKey::random_key(rng);
        Ok(MemoPlan { plaintext, key })
    }

    /// Create a [`MemoCiphertext`] from the [`MemoPlan`].
    pub fn memo(&self) -> Result<MemoCiphertext, anyhow::Error> {
        MemoCiphertext::encrypt(self.key.clone(), &self.plaintext)
    }
}

impl DomainType for MemoPlan {
    type Proto = pb::MemoPlan;
}

impl From<MemoPlan> for pb::MemoPlan {
    fn from(msg: MemoPlan) -> Self {
        Self {
            plaintext: MemoPlaintext::from(msg.plaintext.to_vec()).into(),
            key: msg.key.to_vec().into(),
        }
    }
}

impl TryFrom<pb::MemoPlan> for MemoPlan {
    type Error = anyhow::Error;

    fn try_from(msg: pb::MemoPlan) -> Result<Self, Self::Error> {
        let plaintext = MemoPlaintext::try_from(msg.plaintext.to_vec())?;
        let key = PayloadKey::try_from(msg.key.to_vec())?;
        Ok(Self { plaintext, key })
    }
}
