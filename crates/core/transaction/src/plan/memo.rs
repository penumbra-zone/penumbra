use anyhow::Context;
use penumbra_crypto::{
    memo::{MemoCiphertext, MemoPlaintext},
    symmetric::PayloadKey,
    Address,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType, TypeUrl};

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

impl TypeUrl for MemoPlan {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.MemoPlan";
}

impl DomainType for MemoPlan {
    type Proto = pb::MemoPlan;
}

impl From<MemoPlan> for pb::MemoPlan {
    fn from(msg: MemoPlan) -> Self {
        let sender = Some(msg.plaintext.sender.into());
        let text = msg.plaintext.text;
        Self {
            plaintext: Some(pb::MemoPlaintext { sender, text }),
            key: msg.key.to_vec().into(),
        }
    }
}

impl TryFrom<pb::MemoPlan> for MemoPlan {
    type Error = anyhow::Error;

    fn try_from(msg: pb::MemoPlan) -> Result<Self, Self::Error> {
        let sender: Address = msg
            .plaintext
            .clone()
            .ok_or_else(|| anyhow::anyhow!("memo plan missing memo plaintext"))?
            .sender
            .ok_or_else(|| anyhow::anyhow!("memo plaintext missing sender address"))?
            .try_into()
            .context("sender address malformed")?;

        let text: String = msg
            .plaintext
            .ok_or_else(|| anyhow::anyhow!("memo plan missing memo plaintext"))?
            .text;

        let key = PayloadKey::try_from(msg.key.to_vec())?;

        Ok(Self {
            plaintext: MemoPlaintext { sender, text },
            key,
        })
    }
}
