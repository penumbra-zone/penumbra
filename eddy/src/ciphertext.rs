use std::ops::{Add, AddAssign};

use crate::{
    decryption_share::Verified, limb, DecryptionShare, DecryptionTable, TableLookupError, Value,
};

/// An error indicating that insufficiently many decryption shares
/// were passed to [`Ciphertext::decrypt`].
#[derive(thiserror::Error, Debug)]
#[error("insufficient decryption shares")]
pub struct InsufficientSharesError {}

/// A flow encryption ciphertext.
#[derive(Default, Debug, Clone, Copy)]
pub struct Ciphertext {
    pub(crate) c0: limb::Ciphertext,
    pub(crate) c1: limb::Ciphertext,
    pub(crate) c2: limb::Ciphertext,
    pub(crate) c3: limb::Ciphertext,
}

impl Ciphertext {
    /// Use the provided [`DecryptionShare`]s to decrypt the ciphertext,
    /// recovering the value with the given [`DecryptionTable`].
    ///
    /// # Errors
    ///
    /// - [`InsufficientSharesError`] if insufficiently many decryption shares were supplied;
    /// - [`TableLookupError`] if the decrypted value was out of range for the `table`;
    /// - Underlying I/O errors from the [`DecryptionTable`] implementation.
    pub async fn decrypt(
        &self,
        shares: Vec<DecryptionShare<Verified>>,
        table: &dyn DecryptionTable,
    ) -> anyhow::Result<Value> {
        // TODO: how do we know if we have sufficient shares?
        // How do we return InsufficientSharesError?

        let limb0_shares = shares.iter().map(|s| &s.share0).collect();
        let limb1_shares = shares.iter().map(|s| &s.share1).collect();
        let limb2_shares = shares.iter().map(|s| &s.share2).collect();
        let limb3_shares = shares.iter().map(|s| &s.share3).collect();

        let decryption0 = self.c0.decrypt(limb0_shares);
        let decryption1 = self.c1.decrypt(limb1_shares);
        let decryption2 = self.c2.decrypt(limb2_shares);
        let decryption3 = self.c3.decrypt(limb3_shares);

        let value0 = table
            .lookup(decryption0.vartime_compress().0)
            .await?
            .ok_or(TableLookupError {})?;
        let value1 = table
            .lookup(decryption1.vartime_compress().0)
            .await?
            .ok_or(TableLookupError {})?;
        let value2 = table
            .lookup(decryption2.vartime_compress().0)
            .await?
            .ok_or(TableLookupError {})?;
        let value3 = table
            .lookup(decryption3.vartime_compress().0)
            .await?
            .ok_or(TableLookupError {})?;

        Ok(Value::from_limbs(
            value0.into(),
            value1.into(),
            value2.into(),
            value3.into(),
        ))
    }
}

impl Add<&Ciphertext> for &Ciphertext {
    type Output = Ciphertext;
    fn add(self, rhs: &Ciphertext) -> Self::Output {
        Ciphertext {
            c0: &self.c0 + &rhs.c0,
            c1: &self.c1 + &rhs.c1,
            c2: &self.c2 + &rhs.c2,
            c3: &self.c3 + &rhs.c3,
        }
    }
}

impl AddAssign<&Ciphertext> for Ciphertext {
    fn add_assign(&mut self, rhs: &Ciphertext) {
        self.c0 += &rhs.c0;
        self.c1 += &rhs.c1;
        self.c2 += &rhs.c2;
        self.c3 += &rhs.c3;
    }
}
