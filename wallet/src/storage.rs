use serde::{Deserialize, Serialize};

use penumbra_crypto::keys;

/// The contents of the wallet file that share a spend authority.
#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub spend_seed: keys::SpendSeed,
    // Store the last diversifier index in use.
    pub last_used_diversifier_index: keys::DiversifierIndex,
}
