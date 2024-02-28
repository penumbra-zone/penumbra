//! Shared integration testing facilities.

// NB: Allow dead code, these are in fact shared by files in `tests/`.
#![allow(dead_code)]

use {
    async_trait::async_trait,
    cnidarium::TempStorage,
    penumbra_app::{
        app::App,
        server::consensus::{Consensus, ConsensusService},
    },
    penumbra_genesis::AppState,
    penumbra_mock_consensus::TestNode,
    std::ops::Deref,
    tap::Tap,
    tracing::{trace, warn},
};

// Installs a tracing subscriber to log events until the returned guard is dropped.
pub fn set_tracing_subscriber() -> tracing::subscriber::DefaultGuard {
    use tracing_subscriber::filter::EnvFilter;

    let filter = "info,penumbra_app=trace,penumbra_mock_consensus=trace";
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .expect("should have a valid filter directive")
        // Without explicitly disabling the `r1cs` target, the ZK proof implementations
        // will spend an enormous amount of CPU and memory building useless tracing output.
        .add_directive(
            "r1cs=off"
                .parse()
                .expect("rics=off is a valid filter directive"),
        );

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .pretty()
        .with_test_writer()
        .finish();

    tracing::subscriber::set_default(subscriber)
}

/// A [`TestNode`] coupled with Penumbra's [`Consensus`] service.
pub type PenumbraTestNode = TestNode<ConsensusService>;

/// Returns a new [`PenumbraTestNode`] backed by the given temporary storage.
pub async fn start_test_node(storage: &TempStorage) -> anyhow::Result<PenumbraTestNode> {
    use tap::TapFallible;
    let app_state = AppState::default();
    let consensus = Consensus::new(storage.as_ref().clone());
    TestNode::builder()
        .with_penumbra_single_validator()
        .with_penumbra_auto_app_state(app_state)?
        .init_chain(consensus)
        .await
        .tap_ok(|e| tracing::info!(hash = %e.last_app_hash_hex(), "finished init chain"))
}

#[async_trait]
pub trait TempStorageExt: Sized {
    async fn apply_genesis(self, genesis: AppState) -> anyhow::Result<Self>;
    async fn apply_default_genesis(self) -> anyhow::Result<Self>;
}

#[async_trait]
impl TempStorageExt for TempStorage {
    async fn apply_genesis(self, genesis: AppState) -> anyhow::Result<Self> {
        // Check that we haven't already applied a genesis state:
        if self.latest_version() != u64::MAX {
            anyhow::bail!("database already initialized");
        }

        // Apply the genesis state to the storage
        let mut app = App::new(self.latest_snapshot()).await?;
        app.init_chain(&genesis).await;
        app.commit(self.deref().clone()).await;

        Ok(self)
    }

    async fn apply_default_genesis(self) -> anyhow::Result<Self> {
        self.apply_genesis(Default::default()).await
    }
}

/// Penumbra-specific extensions to the mock consensus builder.
pub trait BuilderExt: Sized {
    /// The error thrown by [`with_penumbra_auto_app_state`]
    type Error;
    /// Add the provided Penumbra [`AppState`] to the builder.
    ///
    /// This will inject any configured validators into the state before serializing it into bytes.
    fn with_penumbra_auto_app_state(self, app_state: AppState) -> Result<Self, Self::Error>;
    /// Creates a single validator with a randomly generated key.
    ///
    /// This will set the builder's identity key.
    fn with_penumbra_single_validator(self) -> Self;
}

impl BuilderExt for penumbra_mock_consensus::builder::Builder {
    type Error = anyhow::Error;
    fn with_penumbra_auto_app_state(self, app_state: AppState) -> Result<Self, Self::Error> {
        use penumbra_proto::penumbra::core::component::stake::v1 as pb;

        // Take the list of genesis validators from the builder...
        let validators = self
            .extensions
            .get::<Vec<pb::Validator>>()
            .ok_or_else(|| {
                anyhow::anyhow!("`with_penumbra_auto_app_state` could not find validators")
            })?
            .clone()
            .tap(|v| {
                for pb::Validator {
                    name,
                    enabled,
                    sequence_number,
                    ..
                } in v
                {
                    // ...log the name of each...
                    trace!(%name, %enabled, %sequence_number, "injecting validator into app state");
                }
                // ...or print a warning if there are not any validators.
                if v.is_empty() {
                    warn!("`with_penumbra_auto_app_state` was called but builder has no validators")
                }
            });

        // Add the validators to the app state.
        let app_state: AppState = match app_state {
            AppState::Checkpoint(_) => anyhow::bail!("checkpoint app state isn't supported"),
            AppState::Content(mut content) => {
                // Inject the builder's validators into the staking component's genesis state.
                std::mem::replace(
                        &mut content.stake_content.validators,
                        validators
                    )
                    .tap(|overwritten| {
                        // Log a warning if this overwrote any validators already in the app state.
                        if !overwritten.is_empty() {
                            warn!(?overwritten, "`with_penumbra_auto_app_state` overwrote validators in the given AppState")
                        }
                    });
                AppState::Content(content)
            }
        };

        // Serialize the app state into bytes, and add it to the builder.
        serde_json::to_vec(&app_state)
            .map_err(Self::Error::from)
            .map(|s| self.app_state(s))
    }

    fn with_penumbra_single_validator(self) -> Self {
        use {
            decaf377_rdsa::VerificationKey,
            penumbra_keys::keys::{SpendKey, SpendKeyBytes},
            penumbra_proto::{
                core::keys::v1::{GovernanceKey, IdentityKey},
                penumbra::core::component::stake::v1::Validator,
            },
            rand::Rng,
            rand_core::OsRng,
        };

        // Generate a spend authoration key.
        let bytes = {
            let spend_key = SpendKey::from(SpendKeyBytes(OsRng.gen()));
            let spend_auth_key = spend_key.spend_auth_key();
            let verification_key = VerificationKey::from(spend_auth_key);
            verification_key.to_bytes()
        };

        // Generate a validator entry using the generated key.
        let validator = Validator {
            identity_key: Some(IdentityKey {
                ik: bytes.to_vec().clone(),
            }),
            governance_key: Some(GovernanceKey {
                // NB: for now, we will use the same key for governance. See the documentation of
                // `GovernanceKey` for more information about cold storage of validator keys.
                gk: bytes.to_vec().clone(),
            }),
            consensus_key: {
                let signing_key = ed25519_consensus::SigningKey::new(OsRng);
                signing_key.as_bytes().as_slice().to_vec()
            },
            enabled: true,
            sequence_number: 0,
            name: String::default(),
            website: String::default(),
            description: String::default(),
            funding_streams: Vec::default(),
        };
        let validators = vec![validator];

        // Add the generated identity key and the validator information to the builder.
        self.identity_key(bytes).extension(validators)
    }
}

pub trait TestNodeExt {
    fn penumbra_identity_key(&self) -> penumbra_stake::IdentityKey;
}

impl<C> TestNodeExt for TestNode<C> {
    fn penumbra_identity_key(&self) -> penumbra_stake::IdentityKey {
        self.identity_key()
            .try_into()
            .map(penumbra_stake::IdentityKey)
            .expect("test node should have a valid identity key")
    }
}
