//! [`Builder`] interfaces, for creating new [`TestNode`]s.

/// [`Builder`] interfaces for chain initialization.
///
/// Most importantly, defines [`Builder::init_chain()`].
mod init_chain;

use {crate::TestNode, bytes::Bytes};

/// A buider, used to prepare and instantiate a new [`TestNode`].
#[derive(Default)]
pub struct Builder {
    app_state: Option<Bytes>,
}

impl TestNode<()> {
    /// Returns a new [`Builder`].
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl Builder {
    // TODO: add other convenience methods for validator config?

    /// Creates a single validator with a randomly generated key.
    pub fn single_validator(self) -> Self {
        // this does not do anything yet
        self
    }

    /// Sets the `app_state_bytes` to send the ABCI application upon chain initialization.
    pub fn app_state(self, app_state: impl Into<Bytes>) -> Self {
        let app_state = Some(app_state.into());
        Self { app_state, ..self }
    }
}
