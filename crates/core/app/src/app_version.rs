/// Representation of the Penumbra application version. Notably, this is distinct
/// from the crate version(s). This number should only ever be incremented.
pub const APP_VERSION: u64 = 12;

cfg_if::cfg_if! {
    if #[cfg(feature="component")] {
        mod component;
        pub use component::{check_and_update_app_version, migrate_app_version};
    }
}
