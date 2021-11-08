use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use penumbra_crypto::{fmd, keys, Address};

/// The contents of the wallet file that share a spend authority.
#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub spend_seed: keys::SpendSeed,
    // Store addresses in use, indexed by their `DiversifierIndex`
    pub addresses: Vec<String>,
}

impl Wallet {
    pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let spend_key = keys::SpendKey::generate(rng);
        let fvk = spend_key.full_viewing_key();
        let ivk = fvk.incoming();
        let (address, _dtdk) = ivk.payment_address(0u64.into());
        Self {
            spend_seed: spend_key.seed().clone(),
            addresses: vec![address.to_string()],
        }
    }

    /// Incoming viewing key from this spend seed.
    pub fn incoming(&self) -> keys::IncomingViewingKey {
        let spend_key = keys::SpendKey::from_seed(self.spend_seed.clone());
        let fvk = spend_key.full_viewing_key();
        fvk.incoming().clone()
    }

    /// Generate a new diversified `Address` and its corresponding `DetectionKey`.
    pub fn new_address(&mut self) -> (Address, fmd::DetectionKey) {
        // The index of the `self.addresses` vector is the diversifier index.
        let new_diversifier_index: keys::DiversifierIndex =
            (self.addresses.len() as u64 + 1u64).into();

        let (new_addr, new_dtdk) = self.incoming().payment_address(new_diversifier_index);
        self.addresses.push(new_addr.to_string());

        (new_addr, new_dtdk)
    }
}
