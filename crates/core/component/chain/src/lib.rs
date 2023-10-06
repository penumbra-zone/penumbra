#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use effect_hash::{EffectHash, EffectingData};
pub use epoch::Epoch;
pub use known_assets::KnownAssets;
pub use note_source::{NoteSource, SpendInfo};
pub use transaction::TransactionContext;

mod epoch;
mod known_assets;
mod note_source;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod genesis;
pub mod params;
pub mod state_key;

/// Hardcoded test data used by the `Default` genesis state.
pub mod test_keys {
    use once_cell::sync::Lazy;

    use penumbra_keys::{
        keys::{Bip44Path, SpendKey, WalletId},
        Address, FullViewingKey,
    };

    /// This address is for test purposes, allocations were added beginning with
    /// the 062-Iapetus testnet.
    /// Previously the test data was generated using BIP39 derivation starting with
    /// the 016-Pandia testnet.
    pub const SEED_PHRASE: &str = "comfort ten front cycle churn burger oak absent rice ice urge result art couple benefit cabbage frequent obscure hurry trick segment cool job debate";

    /// These addresses both correspond to the test wallet above.
    pub const ADDRESS_0_STR: &str = "penumbrav2t147mfall0zr6am5r45qkwht7xqqrdsp50czde7empv7yq2nk3z8yyfh9k9520ddgswkmzar22vhz9dwtuem7uxw0qytfpv7lk3q9dp8ccaw2fn5c838rfackazmgf3ahhvhypxd";
    /// These addresses both correspond to the test wallet above.
    pub const ADDRESS_1_STR: &str = "penumbrav2t1vmmz304hjlkjq6xv4al5dqumvgk3ek82rneagj07vdqkudjvl6y7zxzr5k6qq24yc7yyyekpu9qm7ef3acg2u8p950hs6hu3e73guq5pfmmvm63qudfx4qmg8h7fdweydr93ku";

    pub static ADDRESS_0: Lazy<Address> = Lazy::new(|| {
        ADDRESS_0_STR
            .parse()
            .expect("hardcoded test addresses should be valid")
    });
    pub static ADDRESS_1: Lazy<Address> = Lazy::new(|| {
        ADDRESS_1_STR
            .parse()
            .expect("hardcoded test addresses should be valid")
    });

    pub static SPEND_KEY: Lazy<SpendKey> = Lazy::new(|| {
        SpendKey::from_seed_phrase_bip44(
            SEED_PHRASE
                .parse()
                .expect("hardcoded test seed phrase should be valid"),
            &Bip44Path::new(0),
        )
    });

    pub static FULL_VIEWING_KEY: Lazy<FullViewingKey> =
        Lazy::new(|| SPEND_KEY.full_viewing_key().clone());

    pub static WALLET_ID: Lazy<WalletId> = Lazy::new(|| FULL_VIEWING_KEY.wallet_id());
}

// Located here at the bottom of the dep tree for convenience
mod effect_hash;
mod transaction;
