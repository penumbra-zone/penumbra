use anyhow::Context;
use penumbra_crypto::{
    fmd,
    keys::{
        DiversifierIndex, FullViewingKey, IncomingViewingKey, OutgoingViewingKey, SeedPhrase,
        SpendKey, SpendSeed,
    },
    Address, Note,
};
use serde::{Deserialize, Serialize};

/// The contents of the wallet file that share a spend authority.
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Wallet {
    viewing_key: FullViewingKey,
}

impl Wallet {
    /// Create a new wallet.
    pub fn from_seed_phrase(seed_phrase: SeedPhrase) -> Self {
        // Currently we support a single spend authority per wallet. In the future,
        // we can derive multiple spend seeds from a single seed phrase.
        let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
        let spend_key = SpendKey::new(spend_seed);

        Self {
            viewing_key: spend_key.full_viewing_key().to_owned(),
        }
    }

    /// Imports a wallet from a legacy [`SpendSeed`].
    pub fn import(spend_seed: SpendSeed) -> Self {
        let viewing_key = SpendKey::new(spend_seed).full_viewing_key().to_owned();
        Self { viewing_key }
    }

    /// Incoming viewing key from this spend seed.
    pub fn incoming_viewing_key(&self) -> &IncomingViewingKey {
        self.viewing_key.incoming()
    }

    /// Outgoing viewing key from this spend seed.
    pub fn outgoing_viewing_key(&self) -> &OutgoingViewingKey {
        self.viewing_key.outgoing()
    }

    /// Get the full viewing key for this wallet.
    pub fn full_viewing_key(&self) -> &FullViewingKey {
        &self.viewing_key
    }

    /// Generate a new diversified `Address` and its corresponding `DetectionKey`.
    pub fn new_address(&mut self, index: DiversifierIndex) -> (Address, fmd::DetectionKey) {
        self.incoming_viewing_key().payment_address(index)
    }

    /// Get address by index.
    pub fn address_by_index(&self, index: usize) -> Result<Address, anyhow::Error> {
        let (address, _dtk) = self.incoming_viewing_key().payment_address(index.into());
        Ok(address)
    }

    /// Computes the change address for the given note.
    pub fn change_address(&self, note: &Note) -> Result<Address, anyhow::Error> {
        let index: u64 = self
            .incoming_viewing_key()
            .index_for_diversifier(&note.diversifier())
            .try_into()
            .context("cannot convert DiversifierIndex to u64")?;
        self.address_by_index(index as usize)
    }
}
