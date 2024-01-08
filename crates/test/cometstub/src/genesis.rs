use {
    crate::{validator::Validators, State},
    tendermint::{block, chain, AppHash, Time},
};

/// The initial conditions for the consensus [`Engine`].
pub struct Genesis {
    /// The chain identifier.
    chain_id: chain::Id,
    /// The timestamp for the initial block.
    genesis_time: Time,
    /// The initial [`block::Height`].
    initial_height: block::Height,
    /// The set of validators.
    validators: Validators,
    /// The initial app hash.
    app_hash: AppHash,
    // TODO(kate): see `penumbra_app::genesis::AppState`
    //   - app_state: serde_json::Value,
    // TODO(kate): handle consensus parameters later.
    //   - consensus_params: (),
}

impl Default for Genesis {
    fn default() -> Self {
        let app_hash: AppHash = b"placeholder-app-hash"
            .to_vec()
            .try_into()
            .expect("infallible");
        Self {
            genesis_time: Time::unix_epoch(),
            chain_id: chain::Id::try_from("penumbra-cometstub").unwrap(),
            initial_height: 0_u32.into(),
            validators: Validators::default(),
            app_hash,
        }
    }
}

impl Genesis {
    /// Returns a new, default [`Genesis`] object.
    ///
    /// Use the builder-style `with_` methods to override particular parts of the genesis state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes this genesis information and returns a new [`State`].
    //
    //  TODO(kate): what do we need at the start? avoid recreating a whole genesis file.
    //  use a single validator key, no arbitrary genesis height, simple.
    //
    //  TODO(kate): perhaps this should also *generate* the first block. makes the state object
    //  much nicer to work with going forward.
    pub fn into_state(self) -> (State, tendermint::Block) {
        let Self {
            genesis_time: _, // XXX(kate): where do we put this? `next_block`
            chain_id,
            initial_height,
            validators,
            app_hash,
        } = self;

        let state = State {
            chain_id,
            initial_height,
            last_block: None,
            validators,
            app_hash,
            last_results_hash: None,
        };
        let block = State::generate_block();

        (state, block)
    }

    pub fn with_chain_id(self, chain_id: chain::Id) -> Self {
        Self { chain_id, ..self }
    }

    pub fn with_genesis_time(self, genesis_time: Time) -> Self {
        Self {
            genesis_time,
            ..self
        }
    }

    pub fn with_initial_height(self, initial_height: block::Height) -> Self {
        Self {
            initial_height,
            ..self
        }
    }

    pub fn with_validators(self, validators: Validators) -> Self {
        Self { validators, ..self }
    }

    pub fn with_app_hash(self, app_hash: AppHash) -> Self {
        Self { app_hash, ..self }
    }
}
