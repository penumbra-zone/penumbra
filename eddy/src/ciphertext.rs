use crate::{decryption_share::Verified, limb, DecryptionShare, DecryptionTable, Value};

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
        let share0 = shares
            .iter()
            .map(|s| (s.participant_index, s.share0.clone()))
            .collect::<Vec<_>>();
        let share1 = shares
            .iter()
            .map(|s| (s.participant_index, s.share1.clone()))
            .collect::<Vec<_>>();
        let share2 = shares
            .iter()
            .map(|s| (s.participant_index, s.share2.clone()))
            .collect::<Vec<_>>();
        let share3 = shares
            .iter()
            .map(|s| (s.participant_index, s.share3.clone()))
            .collect::<Vec<_>>();

        let decryption0 = self.c0.decrypt(share0);
        let decryption1 = self.c1.decrypt(share1);
        let decryption2 = self.c2.decrypt(share2);
        let decryption3 = self.c3.decrypt(share3);

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
