use once_cell::sync::Lazy;

pub mod asset;
pub mod balance;
mod value;

pub use balance::Balance;
pub use value::{Value, ValueVar, ValueView};

pub static STAKING_TOKEN_DENOM: Lazy<asset::DenomMetadata> = Lazy::new(|| {
    asset::Cache::with_known_assets()
        .get_unit("upenumbra")
        .unwrap()
        .base()
});
pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> = Lazy::new(|| STAKING_TOKEN_DENOM.id());
