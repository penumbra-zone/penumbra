use std::convert::TryFrom;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use super::{
    bip44::Bip44Path,
    seed_phrase::{SeedPhrase, NUM_PBKDF2_ROUNDS},
    FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey,
};
use crate::{
    keys::bip44::ckd_priv,
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

impl TypeUrl for SpendKey {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.SpendKey";
}

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
        // First derive the HD wallet master node.
        let password = format!("{seed_phrase}");
        // The salt used for deriving the master key is defined in BIP32.
        let salt = "Bitcoin seed";
        let mut m_bytes = [0u8; 32];
        pbkdf2::<Hmac<sha2::Sha512>>(
            password.as_bytes(),
            salt.as_bytes(),
            NUM_PBKDF2_ROUNDS,
            &mut m_bytes,
        )
        .expect("seed phrase hash always succeeds");

        // Now we derive the child keys from the BIP44 path. There are five levels
        // in the BIP44 path: purpose, coin type, account, change, and address index.

        // The first key is derived from the master key and the purpose constant.
        // i is set to 2^31 to indicate that hardened derivation should be used
        // for the first three levels.
        let i = 2u32.pow(31);
        let mut purpose_bytes = [0u8; 32];
        purpose_bytes.copy_from_slice(&path.purpose().to_le_bytes());
        let (k_1, _) = ckd_priv(m_bytes, purpose_bytes, i);

        // The second key is derived from the first key and the coin type constant.
        let mut coin_type_bytes = [0u8; 32];
        coin_type_bytes.copy_from_slice(&path.coin_type().to_le_bytes());
        let (k_2, _) = ckd_priv(k_1, coin_type_bytes, i);

        // The third key is derived from the second key and the account constant.
        let mut account_bytes = [0u8; 32];
        account_bytes.copy_from_slice(&path.account().to_le_bytes());
        let (k_3, _) = ckd_priv(k_2, account_bytes, i);

        // The fourth key is derived from the third key and the change constant.
        // Starting here, we set i=0 to indicate that public derivation should be used
        // for the change and address index levels.
        let i = 0;
        let mut change_bytes = [0u8; 32];
        change_bytes.copy_from_slice(&path.change().to_le_bytes());
        let (k_4, _) = ckd_priv(k_3, change_bytes, i);

        // The fifth key is derived from the fourth key and the address index.
        let mut address_index_bytes = [0u8; 32];
        address_index_bytes.copy_from_slice(&path.address_index().to_le_bytes());
        let (k_5, _) = ckd_priv(k_4, address_index_bytes, i);

        SpendKeyBytes(k_5).into()
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
        use penumbra_proto::serializers::bech32str;
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
        use penumbra_proto::serializers::bech32str;
        pb::SpendKey {
            inner: bech32str::decode(s, bech32str::spend_key::BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}
