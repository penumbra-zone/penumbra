//! [`Builder`] interfaces, for creating new [`TestNode`]s.

/// [`Builder`] interfaces for chain initialization.
///
/// Most importantly, defines [`Builder::init_chain()`].
mod init_chain;

use {
    crate::{Keyring, OnBlockFn, TestNode, TsCallbackFn},
    anyhow::Result,
    bytes::Bytes,
    ed25519_consensus::{SigningKey, VerificationKey},
    std::time::Duration,
    tendermint::{Genesis, Time},
};

// Default timestamp callback will increment the time by 5 seconds.
// can't be const :(
fn default_ts_callback(t: Time) -> Time {
    t.checked_add(Duration::from_secs(5)).unwrap()
}

/// A builder, used to prepare and instantiate a new [`TestNode`].
#[derive(Default)]
pub struct Builder {
    pub app_state: Option<Bytes>,
    pub keyring: Keyring,
    pub on_block: Option<OnBlockFn>,
    pub ts_callback: Option<TsCallbackFn>,
    pub initial_timestamp: Option<Time>,
    pub chain_id: Option<String>,
    /// Hardcodes a genesis to be used for the chain.
    /// Useful if you're trying to test cometbft compatibility
    /// but should not be used typically.
    pub hardcoded_genesis: Option<Genesis>,
    /// Specifies hardcoded keys to be assigned to validators.
    /// Lowest indices are assigned first. If not enough are provided,
    /// random keys will be generated for the remaining validators.
    /// The default behavior is to generate random keys if none are
    /// supplied.
    pub keys: Vec<(SigningKey, VerificationKey)>,
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
        if self.keys.len() >= 1 {
            Self::add_key(&mut keyring, self.keys[0].clone());
        } else {
            let key = Self::generate_key();
            Self::add_key(&mut keyring, key);
        }

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
        if self.keys.len() >= 2 {
            Self::add_key(&mut keyring, self.keys[0].clone());
            Self::add_key(&mut keyring, self.keys[1].clone());
        } else {
            let key = Self::generate_key();
            Self::add_key(&mut keyring, key);
            let key = Self::generate_key();
            Self::add_key(&mut keyring, key);
        }

        Self { keyring, ..self }
    }

    /// Generates consensus keys.
    fn generate_key() -> (SigningKey, VerificationKey) {
        let sk = ed25519_consensus::SigningKey::new(rand_core::OsRng);
        let vk = sk.verification_key();
        tracing::trace!(verification_key = ?vk, "generated consensus key");

        (sk, vk)
    }

    /// Places keys in the provided keyring.
    fn add_key(keyring: &mut Keyring, key: (SigningKey, VerificationKey)) {
        let (sk, vk) = key;
        keyring.insert(vk, sk);
    }

    /// Sets a callback that will be invoked when a new block is constructed.
    pub fn on_block<F>(self, f: F) -> Self
    where
        F: FnMut(tendermint::Block) + Send + Sync + 'static,
    {
        Self {
            on_block: Some(Box::new(f)),
            ..self
        }
    }

    /// Sets a callback that will be invoked when a block is committed, to increment
    /// the timestamp.
    pub fn ts_callback<F>(self, f: F) -> Self
    where
        F: Fn(Time) -> Time + Send + Sync + 'static,
    {
        Self {
            ts_callback: Some(Box::new(f)),
            ..self
        }
    }

    /// Sets the starting time for the test node. If not called,
    /// the current timestamp will be used.
    pub fn with_initial_timestamp(self, initial_time: Time) -> Self {
        Self {
            initial_timestamp: Some(initial_time),
            ..self
        }
    }

    /// Sets the keys used by validators.
    pub fn with_keys(self, keys: Vec<(SigningKey, VerificationKey)>) -> Self {
        let Self {
            keyring: ref prev, ..
        } = self;

        // Fail with a warning if we are about to overwrite any existing keys.
        if !prev.is_empty() {
            panic!("with_keys should be called prior to constructing the keyring");
        }

        Self { keys: keys, ..self }
    }

    /// Add the provided Tendermint [`Genesis`] to the builder.
    ///
    /// This will override other configurations and hardcode the genesis data.
    pub fn with_tendermint_genesis(self, genesis: Genesis) -> Self {
        let Self { .. } = &self;
        Self {
            app_state: Some(serde_json::to_vec(&genesis.app_state).unwrap().into()),
            initial_timestamp: Some(genesis.genesis_time),
            hardcoded_genesis: Some(genesis),
            ..self
        }
    }
}
