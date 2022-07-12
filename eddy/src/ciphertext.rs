use crate::{decryption_share::Verified, limb, DecryptionShare, DecryptionTable, Value};

/// A flow encryption ciphertext.
#[derive(Default, Debug, Clone, Copy)]
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
        let limb0_shares = shares.iter().map(|s| &s.share0).collect();
        let limb1_shares = shares.iter().map(|s| &s.share1).collect();
        let limb2_shares = shares.iter().map(|s| &s.share2).collect();
        let limb3_shares = shares.iter().map(|s| &s.share3).collect();

        let decryption0 = self.c0.decrypt(limb0_shares);
        let decryption1 = self.c1.decrypt(limb1_shares);
        let decryption2 = self.c2.decrypt(limb2_shares);
        let decryption3 = self.c3.decrypt(limb3_shares);

        let value0 = table
            .lookup(decryption0.compress().0)
            .await?
            .ok_or_else(|| anyhow::anyhow!("could not find value in LUT"))?;
        let value1 = table
            .lookup(decryption1.compress().0)
            .await?
            .ok_or_else(|| anyhow::anyhow!("could not find value in LUT"))?;
        let value2 = table
            .lookup(decryption2.compress().0)
            .await?
            .ok_or_else(|| anyhow::anyhow!("could not find value in LUT"))?;
        let value3 = table
            .lookup(decryption3.compress().0)
            .await?
            .ok_or_else(|| anyhow::anyhow!("could not find value in LUT"))?;

        Ok(Value::from_limbs(
            value0.into(),
            value1.into(),
            value2.into(),
            value3.into(),
        ))
    }
}
