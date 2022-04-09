mod component;

pub mod app;
pub mod ibc;
pub mod shielded_pool;
pub mod staking;

// TODO: demote this from `pub` at some point when that's
// not likely to generate conflicts
pub mod validator_set;

pub use self::ibc::IBCComponent;
pub use app::App;
pub use component::Component;
pub use shielded_pool::ShieldedPool;
pub use staking::Staking;
