use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use penumbra_crypto::{fmd, keys, Address};

/// The contents of the wallet file that share a spend authority.
#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub spend_seed: keys::SpendSeed,
    /// A list of human-readable labels for addresses.
    ///
    /// The label at index `i` is used for the address with `DiversifierIndex(i)`.
    pub address_labels: Vec<String>,
}

impl Wallet {
    pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let spend_key = keys::SpendKey::generate(rng);
        Self {
            spend_seed: spend_key.seed().clone(),
            address_labels: vec!["Default".to_string()],
        }
    }

    /// Incoming viewing key from this spend seed.
    pub fn incoming(&self) -> keys::IncomingViewingKey {
        let spend_key = keys::SpendKey::from_seed(self.spend_seed.clone());
        let fvk = spend_key.full_viewing_key();
        fvk.incoming().clone()
    }

    /// Generate a new diversified `Address` and its corresponding `DetectionKey`.
    pub fn new_address(&mut self, label: String) -> (usize, Address, fmd::DetectionKey) {
        let next_index = self.address_labels.len();
        self.address_labels.push(label);
        let (address, dtk) = self.incoming().payment_address(next_index.into());
        (next_index, address, dtk)
    }

    /// Iterate through the addresses in this wallet.
    pub fn addresses(&self) -> impl Iterator<Item = (usize, String, Address)> {
        let incoming = self.incoming();
        self.address_labels
            .clone()
            .into_iter()
            .enumerate()
            .map(move |(index, label)| {
                let (address, _dtk) = incoming.payment_address(index.into());

                (index, label, address)
            })
    }
}
