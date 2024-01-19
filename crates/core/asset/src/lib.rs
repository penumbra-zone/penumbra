#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
use once_cell::sync::Lazy;

pub mod asset;
pub mod balance;
mod value;

pub use balance::Balance;
pub use value::{Value, ValueVar, ValueView};

pub static STAKING_TOKEN_DENOM: Lazy<asset::DenomMetadata> = Lazy::new(|| {
    asset::Cache::with_known_assets()
        .get_unit("upenumbra")
        .expect("unable to get upenumbra denom, which should be hardcoded")
        .base()
});
pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> = Lazy::new(|| STAKING_TOKEN_DENOM.id());
