//! [`Builder`] interfaces, for creating new [`TestNode`]s.

/// [`Builder`] interfaces for chain initialization.
///
/// Most importantly, defines [`Builder::init_chain()`].
mod init_chain;

use {
    crate::TestNode,
    bytes::Bytes,
    decaf377_rdsa::{SpendAuth, VerificationKeyBytes},
    http::Extensions,
    tap::TapOptional,
    tracing::warn,
};

/// A buider, used to prepare and instantiate a new [`TestNode`].
#[derive(Default)]
pub struct Builder {
    pub app_state: Option<Bytes>,
    pub identity_key: Option<VerificationKeyBytes<SpendAuth>>,
    pub extensions: Extensions,
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

    /// Sets the test node's identity key.
    pub fn identity_key(self, identity_key: impl Into<VerificationKeyBytes<SpendAuth>>) -> Self {
        let identity_key = Some(identity_key.into());
        Self {
            identity_key,
            ..self
        }
    }

    /// Adds an extension to this builder.
    ///
    /// This is not a part of "regular" use of this builder, but may be used to store additional
    /// state to facilitate the implementation of extension traits around this builder.
    pub fn extension<T>(mut self, value: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.extensions
            .insert(value)
            .tap_some(|_| warn!("builder overwrote an extension value, this is probably a bug!"));
        self
    }
}
