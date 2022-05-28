use anyhow::Context;
use penumbra_crypto::{
    fmd,
    keys::{
        FullViewingKey, IncomingViewingKey, OutgoingViewingKey, SeedPhrase, SpendKey, SpendKeyBytes,
    },
    Address, Note,
};
use serde::{Deserialize, Serialize};

/// The contents of the wallet file that share a spend authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "serde_helpers::WalletHelper")]
#[serde(into = "serde_helpers::WalletHelper")]
pub struct Wallet {
    /// A list of human-readable labels for addresses.
    ///
    /// The label at index `i` is used for the address with `DiversifierIndex(i)`.
    address_labels: Vec<String>,
    spend_key: SpendKey,
}

impl Wallet {
    /// Create a new wallet.
    pub fn from_seed_phrase(seed_phrase: SeedPhrase) -> Self {
        // Currently we support a single spend authority per wallet. In the future,
        // we can derive multiple spend seeds from a single seed phrase.
        let spend_key = SpendKey::from_seed_phrase(seed_phrase, 0);

        Self {
            spend_key,
            address_labels: vec!["Default".to_string()],
        }
    }

    /// Imports a wallet from a legacy [`SpendSeed`].
    pub fn import(spend_seed: SpendKeyBytes) -> Self {
        let spend_key = spend_seed.into();
        Self {
            spend_key,
            address_labels: vec!["Default".to_string()],
        }
    }

    /// Incoming viewing key from this spend seed.
    pub fn incoming_viewing_key(&self) -> &IncomingViewingKey {
        self.spend_key.full_viewing_key().incoming()
    }

    /// Outgoing viewing key from this spend seed.
    pub fn outgoing_viewing_key(&self) -> &OutgoingViewingKey {
        self.spend_key.full_viewing_key().outgoing()
    }

    /// Returns the wallet's spend seed.
    pub fn spend_key(&self) -> &SpendKey {
        &self.spend_key
    }

    /// Get the full viewing key for this wallet.
    pub fn full_viewing_key(&self) -> &FullViewingKey {
        self.spend_key.full_viewing_key()
    }

    /// Generate a new diversified `Address` and its corresponding `DetectionKey`.
    pub fn new_address(&mut self, label: String) -> (usize, Address, fmd::DetectionKey) {
        let next_index = self.address_labels.len();
        self.address_labels.push(label);
        let (address, dtk) = self
            .incoming_viewing_key()
            .payment_address(next_index.into());
        (next_index, address, dtk)
    }

    /// Get address by index.
    pub fn address_by_index(&self, index: usize) -> Result<(String, Address), anyhow::Error> {
        let label = self
            .address_labels
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("no address with index {}", index))?;
        let (address, _dtk) = self.incoming_viewing_key().payment_address(index.into());
        Ok((label.clone(), address))
    }

    /// Iterate through the addresses in this wallet.
    pub fn addresses(&self) -> impl Iterator<Item = (usize, String, Address)> {
        let incoming = self.incoming_viewing_key().clone();
        self.address_labels
            .clone()
            .into_iter()
            .enumerate()
            .map(move |(index, label)| {
                let (address, _dtk) = incoming.payment_address(index.into());

                (index, label, address)
            })
    }

    /// Computes the change address for the given note.
    pub fn change_address(&self, note: &Note) -> Result<Address, anyhow::Error> {
        let index: u64 = self
            .incoming_viewing_key()
            .index_for_diversifier(&note.diversifier())
            .try_into()
            .context("cannot convert DiversifierIndex to u64")?;

        let (_label, address) = self.address_by_index(index as usize)?;
        Ok(address)
    }
}

mod serde_helpers {
    use penumbra_crypto::keys::SpendKeyBytes;
    use serde_with::serde_as;

    use super::*;

    #[serde_as]
    #[derive(Deserialize, Serialize)]
    pub struct WalletHelper {
        address_labels: Vec<String>,
        #[serde_as(as = "serde_with::hex::Hex")]
        spend_seed: [u8; 32],
    }

    impl From<WalletHelper> for Wallet {
        fn from(w: WalletHelper) -> Self {
            Self {
                address_labels: w.address_labels,
                spend_key: SpendKey::from(SpendKeyBytes(w.spend_seed)),
            }
        }
    }

    impl From<Wallet> for WalletHelper {
        fn from(w: Wallet) -> Self {
            Self {
                address_labels: w.address_labels,
                spend_seed: w.spend_key.to_bytes().clone().0,
            }
        }
    }
}
