//! [`Builder`] interfaces, for creating new [`TestNode`]s.

/// [`Builder`] interfaces for chain initialization.
///
/// Most importantly, defines [`Builder::init_chain()`].
mod init_chain;

use {
    crate::{keyring::Keyring, TestNode},
    bytes::Bytes,
};

/// A buider, used to prepare and instantiate a new [`TestNode`].
#[derive(Default)]
pub struct Builder {
    pub app_state: Option<Bytes>,
    pub keyring: Keyring,
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
        let Self {
            app_state: prev, ..
        } = self;

        // Log a warning if we are about to overwrite a previous value.
        if let Some(prev) = prev {
            tracing::warn!(
                ?prev,
                "builder overwriting a previously set `app_state`, this may be a bug!"
            );
        }

        Self {
            app_state: Some(app_state.into()),
            ..self
        }
    }

    /// Generates a single set of validator keys.
    pub fn single_validator(self) -> Self {
        let Self { keyring: prev, .. } = self;

        // Log a warning if we are about to overwrite any existing keys.
        if !prev.is_empty() {
            tracing::warn!(
                count = %prev.len(),
                "builder overwriting entries in keyring, this may be a bug!"
            );
        }

        Self {
            keyring: Keyring::new_with_size(1),
            ..self
        }
    }
}
