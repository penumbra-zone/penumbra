use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use penumbra_crypto::{
    fmd,
    keys::{FullViewingKey, IncomingViewingKey, OutgoingViewingKey, SpendKey},
    Address,
};

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
    pub fn generate<R: CryptoRng + RngCore>(rng: R) -> Self {
        Self {
            spend_key: SpendKey::generate(rng),
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

    /// Spend key from this spend seed.
    pub fn spend_key(&self) -> SpendKey {
        self.spend_key.clone()
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
    pub fn address_by_index(&self, index: usize) -> Result<Address, anyhow::Error> {
        if self.address_labels.get(index).is_none() {
            return Err(anyhow::anyhow!("no address with this index!"));
        }
        let (address, _dtk) = self.incoming_viewing_key().payment_address(index.into());
        Ok(address)
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
}

mod serde_helpers {
    use super::*;
    use penumbra_crypto::keys::SpendSeed;
    use serde_with::serde_as;

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
                spend_key: SpendKey::from(SpendSeed(w.spend_seed)),
            }
        }
    }

    impl From<Wallet> for WalletHelper {
        fn from(w: Wallet) -> Self {
            Self {
                address_labels: w.address_labels,
                spend_seed: w.spend_key.seed().clone().0,
            }
        }
    }
}
