//! Provides a [`Keyring`] for managing consensus keys.

use crate::TestNode;

use {
    ed25519_consensus::{SigningKey, VerificationKey},
    rand_core::{CryptoRng, OsRng, RngCore},
    std::collections::btree_map::{self, BTreeMap},
};

/// A keyring of [`VerificationKey`] and [`SigningKey`] consensus keys.
#[derive(Clone, Debug, Default)]
pub struct Keyring(BTreeMap<VerificationKey, SigningKey>);

/// An entry in a [`Keyring`].
pub type Entry = (VerificationKey, SigningKey);

// === impl Keyring ===

impl Keyring {
    /// Creates a new [`Keyring`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Keyring`] and fills with with `n` random entries.
    pub fn new_with_size(n: usize) -> Self {
        let gen = || Self::generate_key(OsRng);
        std::iter::repeat_with(gen).take(n).collect()
    }

    /// Returns the consensus signing key corresponding to the given verification key.
    pub fn get(&self, verification_key: &VerificationKey) -> Option<&SigningKey> {
        self.0.get(verification_key)
    }

    /// Returns `true` if the keyring contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of entries in the keyring.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets an iterator over the consensus verification keys in the keyring.
    pub fn verification_keys(&self) -> impl Iterator<Item = &VerificationKey> {
        self.0.keys()
    }

    /// Gets an iterator over the consensus signing keys in the keyring.
    pub fn signing_keys(&self) -> impl Iterator<Item = &SigningKey> {
        self.0.values()
    }

    /// Generates a new key using the default [`OsRng`], and inserts it into the keyring.
    ///
    /// This returns the verification key associated with this new entry.
    pub fn add_key(&mut self) -> VerificationKey {
        self.add_key_with(OsRng)
    }

    /// Generates a new key with the provided CSPRNG, and inserts it into the keyring.
    ///
    /// This returns the verification key associated with this new entry.
    pub fn add_key_with<R>(&mut self, rng: R) -> VerificationKey
    where
        R: RngCore + CryptoRng,
    {
        let (vk, sk) = Self::generate_key(rng);
        self.0.insert(vk, sk);
        vk
    }

    /// Generates a new consensus key.
    pub fn generate_key<R>(rng: R) -> Entry
    where
        R: RngCore + CryptoRng,
    {
        let sk = ed25519_consensus::SigningKey::new(rng);
        let vk = sk.verification_key();
        tracing::trace!(verification_key = ?vk, "generated consensus key");
        (vk, sk)
    }
}

type KeyringIter<'a> = btree_map::Iter<'a, VerificationKey, SigningKey>;
impl<'a> IntoIterator for &'a Keyring {
    type Item = (&'a VerificationKey, &'a SigningKey);
    type IntoIter = KeyringIter<'a>;
    fn into_iter(self) -> KeyringIter<'a> {
        self.0.iter()
    }
}

type KeyringIntoIter = btree_map::IntoIter<VerificationKey, SigningKey>;
impl IntoIterator for Keyring {
    type Item = (VerificationKey, SigningKey);
    type IntoIter = KeyringIntoIter;
    fn into_iter(self) -> KeyringIntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Entry> for Keyring {
    fn from_iter<I>(iter: I) -> Keyring
    where
        I: IntoIterator<Item = Entry>,
    {
        let k = iter.into_iter().collect();
        Self(k)
    }
}

// === impl TestNode ===

/// Keyring-related interfaces for a test node.
impl<C> TestNode<C> {
    /// Returns a reference to the test node's set of consensus keys.
    pub fn keyring(&self) -> &Keyring {
        &self.keyring
    }

    /// Returns a mutable reference to the test node's set of consensus keys.
    pub fn keyring_mut(&mut self) -> &mut Keyring {
        &mut self.keyring
    }
}
