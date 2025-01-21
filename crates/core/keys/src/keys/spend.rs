use bip32::XPrv;
use std::convert::TryFrom;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use penumbra_sdk_proto::{penumbra::core::keys::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use super::{
    bip44::Bip44Path,
    seed_phrase::{SeedPhrase, NUM_PBKDF2_ROUNDS},
    FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey,
};
use crate::{
    prf,
    rdsa::{SigningKey, SpendAuth},
};

pub const SPENDKEY_LEN_BYTES: usize = 32;

/// A refinement type for a `[u8; 32]` indicating that it stores the
/// bytes of a spend key.
///
/// TODO(hdevalence): In the future, we should hide the SpendKeyBytes
/// and force everything to use the proto format / bech32 serialization.
/// But we can't do this now, because we need it to support existing wallets.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SpendKeyBytes(pub [u8; SPENDKEY_LEN_BYTES]);

/// A key representing a single spending authority.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "pb::SpendKey", into = "pb::SpendKey")]
pub struct SpendKey {
    seed: SpendKeyBytes,
    ask: SigningKey<SpendAuth>,
    fvk: FullViewingKey,
}

impl PartialEq for SpendKey {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
    }
}

impl Eq for SpendKey {}

impl DomainType for SpendKey {
    type Proto = pb::SpendKey;
}

impl TryFrom<pb::SpendKey> for SpendKey {
    type Error = anyhow::Error;

    fn try_from(msg: pb::SpendKey) -> Result<Self, Self::Error> {
        Ok(SpendKeyBytes::try_from(msg.inner.as_slice())?.into())
    }
}

impl From<SpendKey> for pb::SpendKey {
    fn from(msg: SpendKey) -> Self {
        Self {
            inner: msg.to_bytes().0.to_vec(),
        }
    }
}

impl From<SpendKeyBytes> for SpendKey {
    fn from(seed: SpendKeyBytes) -> Self {
        let ask = SigningKey::new_from_field(prf::expand_ff(b"Penumbra_ExpndSd", &seed.0, &[0; 1]));
        let nk = NullifierKey(prf::expand_ff(b"Penumbra_ExpndSd", &seed.0, &[1; 1]));
        let fvk = FullViewingKey::from_components(ask.into(), nk);

        Self { seed, ask, fvk }
    }
}

impl SpendKey {
    /// Get the [`SpendKeyBytes`] this [`SpendKey`] was derived from.
    ///
    /// This is useful for serialization.
    pub fn to_bytes(&self) -> SpendKeyBytes {
        self.seed.clone()
    }

    /// Deterministically generate a [`SpendKey`] from a [`SeedPhrase`].
    ///
    /// The choice of KDF (PBKDF2), iteration count, and PRF (HMAC-SHA512) are specified
    /// in [`BIP39`]. The salt is specified in BIP39 as the string "mnemonic" plus an optional
    /// passphrase, which we set to an index. This allows us to derive multiple spend
    /// authorities from a single seed phrase.
    ///
    /// [`BIP39`]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
    pub fn from_seed_phrase_bip39(seed_phrase: SeedPhrase, index: u64) -> Self {
        let password = format!("{seed_phrase}");
        let salt = format!("mnemonic{index}");
        let mut spend_seed_bytes = [0u8; 32];
        pbkdf2::<Hmac<sha2::Sha512>>(
            password.as_bytes(),
            salt.as_bytes(),
            NUM_PBKDF2_ROUNDS,
            &mut spend_seed_bytes,
        )
        .expect("seed phrase hash always succeeds");
        SpendKeyBytes(spend_seed_bytes).into()
    }

    pub fn from_seed_phrase_bip44(seed_phrase: SeedPhrase, path: &Bip44Path) -> Self {
        let password = format!("{seed_phrase}");
        let salt = "mnemonic";
        let mut seed_bytes = [0u8; 64];
        pbkdf2::<Hmac<sha2::Sha512>>(
            password.as_bytes(),
            salt.as_bytes(),
            NUM_PBKDF2_ROUNDS,
            &mut seed_bytes,
        )
        .expect("seed phrase hash always succeeds");

        // Now we derive the child keys from the BIP44 path. There are up five levels
        // in the BIP44 path: purpose, coin type, account, change, and address index.
        let child_key = XPrv::derive_from_path(
            &seed_bytes[..],
            &path.path().parse().expect("valid BIP44 path"),
        )
        .expect("can derive child key");
        let child_key_bytes = child_key.to_bytes();

        SpendKeyBytes(child_key_bytes).into()
    }

    // XXX how many of these do we need? leave them for now
    // but don't document until design is more settled

    pub fn spend_auth_key(&self) -> &SigningKey<SpendAuth> {
        &self.ask
    }

    pub fn full_viewing_key(&self) -> &FullViewingKey {
        &self.fvk
    }

    pub fn nullifier_key(&self) -> &NullifierKey {
        self.fvk.nullifier_key()
    }

    pub fn outgoing_viewing_key(&self) -> &OutgoingViewingKey {
        self.fvk.outgoing()
    }

    pub fn incoming_viewing_key(&self) -> &IncomingViewingKey {
        self.fvk.incoming()
    }
}

impl From<[u8; SPENDKEY_LEN_BYTES]> for SpendKeyBytes {
    fn from(bytes: [u8; SPENDKEY_LEN_BYTES]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for SpendKeyBytes {
    type Error = anyhow::Error;
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != SPENDKEY_LEN_BYTES {
            anyhow::bail!("spendseed must be 32 bytes, got {:?}", slice.len());
        }

        let mut bytes = [0u8; SPENDKEY_LEN_BYTES];
        bytes.copy_from_slice(&slice[0..32]);
        Ok(SpendKeyBytes(bytes))
    }
}

impl std::fmt::Display for SpendKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use penumbra_sdk_proto::serializers::bech32str;
        let proto = pb::SpendKey::from(self.clone());
        f.write_str(&bech32str::encode(
            &proto.inner,
            bech32str::spend_key::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::str::FromStr for SpendKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use penumbra_sdk_proto::serializers::bech32str;
        pb::SpendKey {
            inner: bech32str::decode(s, bech32str::spend_key::BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn bip44_test_ledger() {
        // Test account
        let seed = SeedPhrase::from_str("comfort ten front cycle churn burger oak absent rice ice urge result art couple benefit cabbage frequent obscure hurry trick segment cool job debate").unwrap();

        let expected_bytes =
            hex::decode("1b8113fad04f5db00e6acf541949950f85eca3e02e70254838b750b42a2caa51")
                .expect("valid");
        let expected_spendkey = SpendKeyBytes(expected_bytes.try_into().expect("fits in 32 bytes"));

        let derivation_path = Bip44Path::new(0);
        let software_spendkey = SpendKey::from_seed_phrase_bip44(seed, &derivation_path);

        assert_eq!(software_spendkey.to_bytes(), expected_spendkey);
    }
}
