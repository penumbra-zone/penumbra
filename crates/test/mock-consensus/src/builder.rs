//! [`Builder`] interfaces, for creating new [`TestNode`]s.

/// [`Builder`] interfaces for chain initialization.
///
/// Most importantly, defines [`Builder::init_chain()`].
mod init_chain;

use {
    crate::{keyring::Keys, TestNode},
    bytes::Bytes,
};

/// A buider, used to prepare and instantiate a new [`TestNode`].
#[derive(Default)]
pub struct Builder {
    pub app_state: Option<Bytes>,
    pub keys: Option<Keys>,
}

impl TestNode<()> {
    /// Returns a new [`Builder`].
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl Builder {
    /// Sets the `app_state_bytes` to send the ABCI application upon chain initialization.
    pub fn app_state(self, app_state: impl Into<Bytes>) -> Self {
        let app_state = Some(app_state.into());
        Self { app_state, ..self }
    }

    /// Generates a single set of validator keys.
    pub fn single_validator(self) -> Self {
        Self {
            keys: Some(Keys::generate()),
            ..self
        }
    }
}
