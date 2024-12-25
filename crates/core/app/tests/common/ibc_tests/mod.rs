use {anyhow::Result, std::time::Duration};

mod relayer;
use anyhow::Context as _;
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey};
use penumbra_sdk_app::{
    app::{MAX_BLOCK_TXS_PAYLOAD_BYTES, MAX_EVIDENCE_SIZE_BYTES},
    genesis,
};
use penumbra_sdk_keys::keys::{SpendKey, SpendKeyBytes};
use penumbra_sdk_mock_consensus::TestNode;
use penumbra_sdk_proto::core::component::stake::v1::Validator;
use penumbra_sdk_shielded_pool::genesis::Allocation;
use penumbra_sdk_stake::{DelegationToken, GovernanceKey, IdentityKey};
#[allow(unused_imports)]
pub use relayer::MockRelayer;

mod node;
pub use node::TestNodeWithIBC;
use serde::Deserialize;
use tendermint::{consensus::params::AbciParams, public_key::Algorithm, Genesis};

/// Collection of all keypairs required for a Penumbra validator.
/// Used to generate a stable identity for a [`NetworkValidator`].
/// TODO: copied this from pd crate
#[derive(Deserialize)]
pub struct ValidatorKeys {
    /// Penumbra spending key and viewing key for this node.
    /// These need to be real curve points.
    pub validator_id_sk: SigningKey<SpendAuth>,
    pub validator_id_vk: VerificationKey<SpendAuth>,
    pub validator_spend_key: SpendKeyBytes,
    /// Consensus key for tendermint.
    pub validator_cons_sk: tendermint::PrivateKey,
    pub validator_cons_pk: tendermint::PublicKey,
    /// P2P auth key for tendermint.
    pub node_key_sk: tendermint::PrivateKey,
    /// The identity key for the validator.
    pub identity_key: IdentityKey,
    #[allow(unused_variables, dead_code)]
    pub node_key_pk: tendermint::PublicKey,
}

impl ValidatorKeys {
    /// Use a hard-coded seed to generate a new set of validator keys.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        // Create the spend key for this node.
        let seed = SpendKeyBytes(seed);
        let spend_key = SpendKey::from(seed.clone());

        // Create signing key and verification key for this node.
        let validator_id_sk = spend_key.spend_auth_key();
        let validator_id_vk = VerificationKey::from(validator_id_sk);

        let validator_cons_sk = ed25519_consensus::SigningKey::from(seed.0);

        // generate consensus key for tendermint.
        let validator_cons_sk = tendermint::PrivateKey::Ed25519(
            validator_cons_sk
                .as_bytes()
                .as_slice()
                .try_into()
                .expect("32 bytes"),
        );
        let validator_cons_pk = validator_cons_sk.public_key();

        // generate P2P auth key for tendermint.
        let node_key_sk = ed25519_consensus::SigningKey::from(seed.0);
        let signing_key_bytes = node_key_sk.as_bytes().as_slice();

        // generate consensus key for tendermint.
        let node_key_sk =
            tendermint::PrivateKey::Ed25519(signing_key_bytes.try_into().expect("32 bytes"));
        let node_key_pk = node_key_sk.public_key();

        let identity_key: IdentityKey = IdentityKey(
            spend_key
                .full_viewing_key()
                .spend_verification_key()
                .clone()
                .into(),
        );
        ValidatorKeys {
            validator_id_sk: validator_id_sk.clone(),
            validator_id_vk,
            validator_cons_sk,
            validator_cons_pk,
            node_key_sk,
            node_key_pk,
            validator_spend_key: seed,
            identity_key,
        }
    }
}

/// A genesis state that can be fed into CometBFT as well,
/// for verifying compliance of the mock tendermint implementation.
pub fn get_verified_genesis() -> Result<Genesis> {
    let start_time = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;
    let vkeys_a = ValidatorKeys::from_seed([0u8; 32]);

    // TODO: make it possible to flag exporting the app state, keys, etc.
    // to files possible on the builder
    // genesis contents need to contain validator information in the app state
    let mut genesis_contents =
        genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string());

    let spend_key_a = SpendKey::from(vkeys_a.validator_spend_key.clone());
    let validator_a = Validator {
        identity_key: Some(IdentityKey(vkeys_a.validator_id_vk.into()).into()),
        governance_key: Some(GovernanceKey(spend_key_a.spend_auth_key().into()).into()),
        consensus_key: vkeys_a.validator_cons_pk.to_bytes(),
        name: "test".to_string(),
        website: "https://example.com".to_string(),
        description: "test".to_string(),
        enabled: true,
        funding_streams: vec![],
        sequence_number: 0,
    };

    // let's only do one validator per chain for now
    // since it's easier to validate against cometbft
    genesis_contents
        .stake_content
        .validators
        .push(validator_a.clone());

    // the validator needs some initial delegations
    let identity_key_a: IdentityKey = IdentityKey(
        spend_key_a
            .full_viewing_key()
            .spend_verification_key()
            .clone()
            .into(),
    );
    let delegation_id_a = DelegationToken::from(&identity_key_a).denom();
    let ivk_a = spend_key_a.incoming_viewing_key();
    genesis_contents
        .shielded_pool_content
        .allocations
        .push(Allocation {
            address: ivk_a.payment_address(0u32.into()).0,
            raw_amount: (25_000 * 10u128.pow(6)).into(),
            raw_denom: delegation_id_a.to_string(),
        });

    let genesis = Genesis {
        genesis_time: start_time.clone(),
        chain_id: genesis_contents
            .chain_id
            .parse::<tendermint::chain::Id>()
            .context("failed to parse chain ID")?,
        initial_height: 0,
        consensus_params: tendermint::consensus::Params {
            abci: AbciParams::default(),
            block: tendermint::block::Size {
                // 1MB
                max_bytes: MAX_BLOCK_TXS_PAYLOAD_BYTES as u64,
                // Set to infinity since a chain running Penumbra won't use
                // cometbft's notion of gas.
                max_gas: -1,
                // Minimum time increment between consecutive blocks.
                time_iota_ms: 500,
            },
            evidence: tendermint::evidence::Params {
                // We should keep this in approximate sync with the recommended default for
                // `StakeParameters::unbonding_delay`, this is roughly a week.
                max_age_num_blocks: 130000,
                // Similarly, we set the max age duration for evidence to be a little over a week.
                max_age_duration: tendermint::evidence::Duration(Duration::from_secs(650000)),
                // 30KB
                max_bytes: MAX_EVIDENCE_SIZE_BYTES as i64,
            },
            validator: tendermint::consensus::params::ValidatorParams {
                pub_key_types: vec![Algorithm::Ed25519],
            },
            version: Some(tendermint::consensus::params::VersionParams { app: 0 }),
        },
        // always empty in genesis json
        app_hash: tendermint::AppHash::default(),
        // app_state: genesis_contents.into(),
        app_state: serde_json::value::to_value(penumbra_sdk_app::genesis::AppState::Content(
            genesis_contents,
        ))
        .unwrap(),
        // Set empty validator set for Tendermint config, which falls back to reading
        // validators from the AppState, via ResponseInitChain:
        // https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
        validators: vec![],
    };

    Ok(genesis)
}
