use anyhow::Context;
use penumbra_keys::{symmetric::PayloadKey, Address};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType};

use rand::{CryptoRng, RngCore};

use crate::memo::{MemoCiphertext, MemoPlaintext};

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
    ) -> anyhow::Result<MemoPlan> {
        let key = PayloadKey::random_key(rng);
        Ok(MemoPlan { plaintext, key })
    }

    /// Create a [`MemoCiphertext`] from the [`MemoPlan`].
    pub fn memo(&self) -> anyhow::Result<MemoCiphertext> {
        MemoCiphertext::encrypt(self.key.clone(), &self.plaintext)
    }
}

impl DomainType for MemoPlan {
    type Proto = pb::MemoPlan;
}

impl From<MemoPlan> for pb::MemoPlan {
    fn from(msg: MemoPlan) -> Self {
        let return_address = Some(msg.plaintext.return_address().into());
        let text = msg.plaintext.text().to_owned();
        Self {
            plaintext: Some(pb::MemoPlaintext {
                return_address,
                text,
            }),
            key: msg.key.to_vec(),
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
            .return_address
            .ok_or_else(|| anyhow::anyhow!("memo plaintext missing return address"))?
            .try_into()
            .context("return address malformed")?;

        let text: String = msg
            .plaintext
            .ok_or_else(|| anyhow::anyhow!("memo plan missing memo plaintext"))?
            .text;

        let key = PayloadKey::try_from(msg.key.to_vec())?;

        Ok(Self {
            plaintext: MemoPlaintext::new(sender, text)?,
            key,
        })
    }
}
