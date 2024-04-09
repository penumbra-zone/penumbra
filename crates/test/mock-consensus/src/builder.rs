//! [`Builder`] interfaces, for creating new [`TestNode`]s.

/// [`Builder`] interfaces for chain initialization.
///
/// Most importantly, defines [`Builder::init_chain()`].
mod init_chain;

use {
    crate::{Keyring, TestNode},
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

        // Generate a key and place it in the keyring.
        let mut keyring = Keyring::new();
        Self::add_key(&mut keyring);

        Self { keyring, ..self }
    }

    /// Generates a pair of validator keys.
    pub fn two_validators(self) -> Self {
        let Self { keyring: prev, .. } = self;

        // Log a warning if we are about to overwrite any existing keys.
        if !prev.is_empty() {
            tracing::warn!(
                count = %prev.len(),
                "builder overwriting entries in keyring, this may be a bug!"
            );
        }

        // Generate two keys and place them in the keyring.
        let mut keyring = Keyring::new();
        Self::add_key(&mut keyring);
        Self::add_key(&mut keyring);

        Self { keyring, ..self }
    }

    /// Generates consensus keys and places them in the provided keyring.
    fn add_key(keyring: &mut Keyring) {
        let sk = ed25519_consensus::SigningKey::new(rand_core::OsRng);
        let vk = sk.verification_key();
        tracing::trace!(verification_key = ?vk, "generated consensus key");
        keyring.insert(vk, sk);
    }
}
