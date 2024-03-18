use {
    penumbra_genesis::AppState,
    penumbra_mock_consensus::builder::Builder,
    penumbra_proto::{
        core::keys::v1::{GovernanceKey, IdentityKey},
        penumbra::core::component::stake::v1::Validator as PenumbraValidator,
    },
    penumbra_shielded_pool::genesis::Allocation,
    tracing::trace,
};

/// Penumbra-specific extensions to the mock consensus builder.
pub trait BuilderExt: Sized {
    /// The error thrown by [`with_penumbra_auto_app_state`]
    type Error;
    /// Add the provided Penumbra [`AppState`] to the builder.
    ///
    /// This will inject any configured validators into the state before serializing it into bytes.
    fn with_penumbra_auto_app_state(self, app_state: AppState) -> Result<Self, Self::Error>;
}

impl BuilderExt for Builder {
    type Error = anyhow::Error;
    fn with_penumbra_auto_app_state(self, app_state: AppState) -> Result<Self, Self::Error> {
        let Self { keyring, .. } = &self;
        let mut content = match app_state {
            AppState::Content(c) => c,
            AppState::Checkpoint(_) => anyhow::bail!("checkpointed state is not supported"),
        };

        for (consensus_vk, _) in keyring {
            // Generate a penumbra validator with this consensus key, and a corresponding
            // allocation of delegation tokens.
            let (validator, allocation) = generate_penumbra_validator(consensus_vk);

            // Add the validator to the staking component's genesis content.
            trace!(?validator, "adding validator to staking genesis content");
            content.stake_content.validators.push(validator);

            // Add an allocation of delegation tokens to the shielded pool content.
            trace!(
                ?allocation,
                "adding allocation to shielded pool genesis content"
            );
            content.shielded_pool_content.allocations.push(allocation);
        }

        // Serialize the app state into bytes, and add it to the builder.
        let app_state = AppState::Content(content);
        serde_json::to_vec(&app_state)
            .map_err(Self::Error::from)
            .map(|s| self.app_state(s))
    }
}

/// Generates a [`Validator`][PenumbraValidator] given a consensus verification key.
fn generate_penumbra_validator(
    consensus_key: &ed25519_consensus::VerificationKey,
) -> (PenumbraValidator, Allocation) {
    use decaf377_rdsa::VerificationKey;
    use penumbra_keys::keys::{SpendKey, SpendKeyBytes};
    use penumbra_stake::DelegationToken;
    use rand::Rng;
    use rand_core::OsRng;

    let seed = SpendKeyBytes(OsRng.gen());
    let spend_key = SpendKey::from(seed.clone());
    let validator_id_sk = spend_key.spend_auth_key();
    let validator_id_vk = VerificationKey::from(validator_id_sk);

    let v = PenumbraValidator {
        identity_key: Some(IdentityKey {
            ik: validator_id_vk.to_bytes().to_vec(),
        }),
        // NB: for now, we will use the same key for governance. See the documentation of
        // `GovernanceKey` for more information about cold storage of validator keys.
        governance_key: Some(GovernanceKey {
            gk: validator_id_vk.to_bytes().to_vec(),
        }),
        consensus_key: consensus_key.as_bytes().to_vec(),
        enabled: true,
        sequence_number: 0,
        name: String::default(),
        website: String::default(),
        description: String::default(),
        funding_streams: Vec::default(),
    };

    let (address, _) = spend_key
        .full_viewing_key()
        .incoming()
        .payment_address(0u32.into());

    let ik = penumbra_stake::IdentityKey(validator_id_vk);
    let delegation_denom = DelegationToken::from(ik).denom();

    let allocation = Allocation {
        raw_amount: 1000u128.into(),
        raw_denom: delegation_denom.to_string(),
        address,
    };

    (v, allocation)
}

fn log_validator(
    PenumbraValidator {
        name,
        enabled,
        sequence_number,
        ..
    }: &PenumbraValidator,
) {
    tracing::trace!(
        %name,
        %enabled,
        %sequence_number,
        "injecting validator into app state"
    )
}
