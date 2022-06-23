use crate::{decryption_share::Verified, dkg, limb, DecryptionShare, DecryptionTable, Value};

/// A flow encryption ciphertext.
pub struct Ciphertext {
    pub(crate) c0: limb::Ciphertext,
    pub(crate) c1: limb::Ciphertext,
    pub(crate) c2: limb::Ciphertext,
    pub(crate) c3: limb::Ciphertext,
}

impl Ciphertext {
    /// Assumes decryption shares are verified already.
    pub async fn decrypt(
        &self,
        shares: Vec<DecryptionShare<Verified>>,
        table: &dyn DecryptionTable,
    ) -> anyhow::Result<Value> {
        todo!()
    }
}
