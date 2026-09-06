/// Representation of the Penumbra application version. Notably, this is distinct
/// from the crate version(s). This number should only ever be incremented.
///
/// v13: token factory (create/mint/burn actions + generic ActionBurn). These are
/// new consensus actions, so the app version must advance from the 2.1.x line (v12).
pub const APP_VERSION: u64 = 13;

cfg_if::cfg_if! {
    if #[cfg(feature="component")] {
        mod component;
        pub use component::{check_and_update_app_version, migrate_app_version};
    }
}
