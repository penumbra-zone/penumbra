//! Logic for creating a new testnet configuration.
//! Used for deploying (approximately weekly) testnets
//! for Penumbra.
use crate::network::config::{get_network_dir, NetworkTendermintConfig, ValidatorKeys};
use anyhow::{Context, Result};
use penumbra_sdk_app::{
    app::{MAX_BLOCK_TXS_PAYLOAD_BYTES, MAX_EVIDENCE_SIZE_BYTES},
    params::AppParameters,
};
use penumbra_sdk_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_fee::genesis::Content as FeeContent;
use penumbra_sdk_governance::genesis::Content as GovernanceContent;
use penumbra_sdk_keys::{keys::SpendKey, Address};
use penumbra_sdk_sct::genesis::Content as SctContent;
use penumbra_sdk_sct::params::SctParameters;
use penumbra_sdk_shielded_pool::{
    genesis::{self as shielded_pool_genesis, Allocation, Content as ShieldedPoolContent},
    params::ShieldedPoolParameters,
};
use penumbra_sdk_stake::{
    genesis::Content as StakeContent, params::StakeParameters, validator::Validator,
    DelegationToken, FundingStream, FundingStreams, GovernanceKey, IdentityKey,
};
use serde::{de, Deserialize};
use std::{
    fmt,
    fs::File,
    io::Read,
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tendermint::consensus::params::AbciParams;
use tendermint::{node, public_key::Algorithm, Genesis, Time};
use tendermint_config::net::Address as TendermintAddress;

/// Represents a Penumbra network config, including initial validators
/// and allocations at genesis time.
pub struct NetworkConfig {
    /// The name of the network
    pub name: String,
    /// The Tendermint genesis for initial chain state.
    pub genesis: Genesis<penumbra_sdk_app::genesis::AppState>,
    /// Path to local directory where config files will be written to
    pub network_dir: PathBuf,
    /// Set of validators at genesis. Uses the convenient wrapper type
    /// to generate config files.
    pub network_validators: Vec<NetworkValidator>,
    /// Set of validators at genesis. This is the literal type embedded
    /// inside configs, including the keys
    pub validators: Vec<Validator>,
    /// Hostname as string for a validator's p2p service. Will have
    /// numbers affixed to it for each validator, e.g. "-0", "-1", etc.
    pub peer_address_template: Option<String>,
    /// The Tendermint `consensus.timeout_commit` value, controlling how long Tendermint should
    /// wait after committing a block, before starting on the new height. If unspecified, `5s`.
    pub tendermint_timeout_commit: Option<tendermint::Timeout>,
}

impl NetworkConfig {
    /// Create a new testnet configuration, optionally customizing the allocations and validator
    /// set. By default, will use the prepared Discord allocations and Penumbra Labs CI validator
    /// configs.
    #[allow(clippy::too_many_arguments)]
    pub fn generate(
        chain_id: &str,
        network_dir: Option<PathBuf>,
        peer_address_template: Option<String>,
        external_addresses: Option<Vec<TendermintAddress>>,
        allocations_input_file: Option<PathBuf>,
        allocation_address: Option<Address>,
        validators_input_file: Option<PathBuf>,
        tendermint_timeout_commit: Option<tendermint::Timeout>,
        active_validator_limit: Option<u64>,
        epoch_duration: Option<u64>,
        unbonding_delay: Option<u64>,
        proposal_voting_blocks: Option<u64>,
        gas_price_simple: Option<u64>,
    ) -> anyhow::Result<NetworkConfig> {
        let external_addresses = external_addresses.unwrap_or_default();

        let network_validators = Self::collect_validators(
            validators_input_file,
            peer_address_template.clone(),
            external_addresses,
        )?;

        let mut allocations = Self::collect_allocations(allocations_input_file)?;

        for v in network_validators.iter() {
            allocations.push(v.delegation_allocation()?);
        }

        // Add an extra allocation for a dynamic wallet address.
        if let Some(address) = allocation_address {
            tracing::info!(%address, "adding dynamic allocation to genesis");
            allocations.extend(NetworkAllocation::simple(address));
        }
        // Convert to domain type, for use with other Penumbra interfaces.
        // We do this conversion once and store it in the struct for convenience.
        let validators: anyhow::Result<Vec<Validator>> =
            network_validators.iter().map(|v| v.try_into()).collect();
        let validators = validators?;

        let app_state = Self::make_genesis_content(
            chain_id,
            allocations,
            validators.to_vec(),
            active_validator_limit,
            epoch_duration,
            unbonding_delay,
            proposal_voting_blocks,
            gas_price_simple,
        )?;
        let genesis = Self::make_genesis(app_state)?;

        Ok(NetworkConfig {
            name: chain_id.to_owned(),
            genesis,
            network_dir: get_network_dir(network_dir),
            network_validators,
            validators: validators.to_vec(),
            peer_address_template,
            tendermint_timeout_commit,
        })
    }

    /// Prepare set of initial validators present at genesis. Optionally reads config values from a
    /// JSON file, otherwise falls back to the Penumbra Labs CI validator configs used for
    /// testnets.
    fn collect_validators(
        validators_input_file: Option<PathBuf>,
        peer_address_template: Option<String>,
        external_addresses: Vec<TendermintAddress>,
    ) -> anyhow::Result<Vec<NetworkValidator>> {
        let testnet_validators = if let Some(validators_input_file) = validators_input_file {
            NetworkValidator::from_json(validators_input_file)?
        } else {
            static LATEST_VALIDATORS: &str = include_str!(env!("PD_LATEST_TESTNET_VALIDATORS"));
            NetworkValidator::from_reader(std::io::Cursor::new(LATEST_VALIDATORS)).with_context(
                || {
                    format!(
                        "could not parse default latest testnet validators file {:?}",
                        env!("PD_LATEST_TESTNET_VALIDATORS")
                    )
                },
            )?
        };

        if !external_addresses.is_empty() && external_addresses.len() != testnet_validators.len() {
            anyhow::bail!("Number of validators did not equal number of external addresses");
        }

        Ok(testnet_validators
            .into_iter()
            .enumerate()
            .map(|(i, v)| NetworkValidator {
                peer_address_template: peer_address_template.as_ref().map(|t| format!("{t}-{i}")),
                external_address: external_addresses.get(i).cloned(),
                ..v
            })
            .collect())
    }

    /// Prepare a set of initial [Allocation]s present at genesis. Optionally reads allocation
    /// files a CSV file, otherwise falls back to the historical requests of the testnet faucet
    /// in the Penumbra Discord channel.
    fn collect_allocations(
        allocations_input_file: Option<PathBuf>,
    ) -> anyhow::Result<Vec<Allocation>> {
        if let Some(ref allocations_input_file) = allocations_input_file {
            Ok(
                NetworkAllocation::from_csv(allocations_input_file.to_path_buf()).with_context(
                    || format!("could not parse allocations file {allocations_input_file:?}"),
                )?,
            )
        } else {
            // Default to latest testnet allocations computed in the build script.
            static LATEST_ALLOCATIONS: &str = include_str!(env!("PD_LATEST_TESTNET_ALLOCATIONS"));
            Ok(
                NetworkAllocation::from_reader(std::io::Cursor::new(LATEST_ALLOCATIONS))
                    .with_context(|| {
                        format!(
                            "could not parse default latest testnet allocations file {:?}",
                            env!("PD_LATEST_TESTNET_ALLOCATIONS")
                        )
                    })?,
            )
        }
    }

    /// Create a full genesis configuration for inclusion in the tendermint
    /// genesis config.
    fn make_genesis_content(
        chain_id: &str,
        allocations: Vec<Allocation>,
        validators: Vec<Validator>,
        active_validator_limit: Option<u64>,
        epoch_duration: Option<u64>,
        unbonding_delay: Option<u64>,
        proposal_voting_blocks: Option<u64>,
        gas_price_simple: Option<u64>,
    ) -> anyhow::Result<penumbra_sdk_app::genesis::Content> {
        let default_gov_params = penumbra_sdk_governance::params::GovernanceParameters::default();

        let gov_params = penumbra_sdk_governance::params::GovernanceParameters {
            proposal_voting_blocks: proposal_voting_blocks
                .unwrap_or(default_gov_params.proposal_voting_blocks),
            ..default_gov_params
        };

        // Look up default app params, so we can fill in defaults.
        let default_app_params = AppParameters::default();

        let gas_price_simple = gas_price_simple.unwrap_or_default();

        let app_state = penumbra_sdk_app::genesis::Content {
            chain_id: chain_id.to_string(),
            stake_content: StakeContent {
                validators: validators.into_iter().map(Into::into).collect(),
                stake_params: StakeParameters {
                    active_validator_limit: active_validator_limit
                        .unwrap_or(default_app_params.stake_params.active_validator_limit),
                    unbonding_delay: unbonding_delay
                        .unwrap_or(default_app_params.stake_params.unbonding_delay),
                    ..Default::default()
                },
            },
            fee_content: FeeContent {
                fee_params: penumbra_sdk_fee::params::FeeParameters {
                    fixed_gas_prices: penumbra_sdk_fee::GasPrices {
                        block_space_price: gas_price_simple,
                        compact_block_space_price: gas_price_simple,
                        verification_price: gas_price_simple,
                        execution_price: gas_price_simple,
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    },
                    fixed_alt_gas_prices: vec![
                        penumbra_sdk_fee::GasPrices {
                            block_space_price: 10 * gas_price_simple,
                            compact_block_space_price: 10 * gas_price_simple,
                            verification_price: 10 * gas_price_simple,
                            execution_price: 10 * gas_price_simple,
                            asset_id: asset::REGISTRY.parse_unit("gm").id(),
                        },
                        penumbra_sdk_fee::GasPrices {
                            block_space_price: 10 * gas_price_simple,
                            compact_block_space_price: 10 * gas_price_simple,
                            verification_price: 10 * gas_price_simple,
                            execution_price: 10 * gas_price_simple,
                            asset_id: asset::REGISTRY.parse_unit("gn").id(),
                        },
                    ],
                },
            },
            governance_content: GovernanceContent {
                governance_params: gov_params,
            },
            shielded_pool_content: ShieldedPoolContent {
                shielded_pool_params: ShieldedPoolParameters::default(),
                allocations: allocations.clone(),
            },
            sct_content: SctContent {
                sct_params: SctParameters {
                    epoch_duration: epoch_duration
                        .unwrap_or(default_app_params.sct_params.epoch_duration),
                },
            },
            ..Default::default()
        };
        Ok(app_state)
    }

    /// Build Tendermint genesis data, based on Penumbra initial application state.
    pub(crate) fn make_genesis(
        app_state: penumbra_sdk_app::genesis::Content,
    ) -> anyhow::Result<Genesis<penumbra_sdk_app::genesis::AppState>> {
        // Use now as genesis time
        let genesis_time = Time::from_unix_timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("expected that time travels linearly in a forward direction")?
                .as_secs() as i64,
            0,
        )
        .context("failed to convert current time into Time")?;

        // Create Tendermint genesis data shared by all nodes
        let genesis = Genesis {
            genesis_time,
            chain_id: app_state
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
            app_state: penumbra_sdk_app::genesis::AppState::Content(app_state),
            // Set empty validator set for Tendermint config, which falls back to reading
            // validators from the AppState, via ResponseInitChain:
            // https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
            validators: vec![],
        };
        Ok(genesis)
    }

    pub(crate) fn make_checkpoint(
        genesis: Genesis<penumbra_sdk_app::genesis::AppState>,
        checkpoint: Option<Vec<u8>>,
    ) -> Genesis<penumbra_sdk_app::genesis::AppState> {
        match checkpoint {
            Some(checkpoint) => Genesis {
                app_state: penumbra_sdk_app::genesis::AppState::Checkpoint(checkpoint),
                ..genesis
            },
            None => genesis,
        }
    }

    /// Generate and write to disk the Tendermint configs for each validator at genesis.
    pub fn write_configs(&self) -> anyhow::Result<()> {
        // Loop over each validator and write its config separately.
        for (n, v) in self.network_validators.iter().enumerate() {
            // Create the directory for this node
            let node_name = format!("node{n}");
            let node_dir = self.network_dir.clone().join(node_name.clone());

            // Each node should include only the IPs for *other* nodes in their peers list.
            let ips_minus_mine: anyhow::Result<Vec<TendermintAddress>> = self
                .network_validators
                .iter()
                .map(|v| v.peering_address())
                .filter(|a| {
                    *a.as_ref().expect("able to get address ref")
                        != v.peering_address()
                            .expect("able to get peering address ref")
                })
                .collect();
            let ips_minus_mine = ips_minus_mine?;
            tracing::debug!(?ips_minus_mine, "Found these peer ips");

            let external_address: Option<TendermintAddress> = v.external_address.as_ref().cloned();
            let mut tm_config = NetworkTendermintConfig::new(
                &node_name,
                ips_minus_mine,
                external_address,
                None,
                None,
            )?;
            if let Some(timeout_commit) = self.tendermint_timeout_commit {
                tm_config.0.consensus.timeout_commit = timeout_commit;
            }
            tm_config.write_config(node_dir, v, &self.genesis)?;
        }
        Ok(())
    }
}

/// Create a new testnet definition, including genesis and at least one
/// validator config. Write all configs to the target testnet dir,
/// defaulting to `~/.penumbra/<chain_id>`.
#[allow(clippy::too_many_arguments)]
pub fn network_generate(
    network_dir: Option<PathBuf>,
    chain_id: &str,
    active_validator_limit: Option<u64>,
    tendermint_timeout_commit: Option<tendermint::Timeout>,
    epoch_duration: Option<u64>,
    unbonding_delay: Option<u64>,
    peer_address_template: Option<String>,
    external_addresses: Vec<TendermintAddress>,
    validators_input_file: Option<PathBuf>,
    allocations_input_file: Option<PathBuf>,
    allocation_address: Option<Address>,
    proposal_voting_blocks: Option<u64>,
    gas_price_simple: Option<u64>,
) -> anyhow::Result<()> {
    tracing::info!(?chain_id, "Generating network config");
    let t = NetworkConfig::generate(
        chain_id,
        network_dir,
        peer_address_template,
        Some(external_addresses),
        allocations_input_file,
        allocation_address,
        validators_input_file,
        tendermint_timeout_commit,
        active_validator_limit,
        epoch_duration,
        unbonding_delay,
        proposal_voting_blocks,
        gas_price_simple,
    )?;
    tracing::info!(
        n_validators = t.validators.len(),
        chain_id = %t.genesis.chain_id,
        "Writing config files for network"
    );
    t.write_configs()?;
    Ok(())
}

/// Represents initial allocations to the testnet.
#[derive(Debug, Deserialize)]
pub struct NetworkAllocation {
    #[serde(deserialize_with = "string_u128")]
    pub amount: u128,
    pub denom: String,
    pub address: String,
}

impl NetworkAllocation {
    /// Import allocations from a CSV file. The format is simple:
    ///
    ///   amount,denom,address
    ///
    /// Typically these CSV files are generated by Galileo.
    pub fn from_csv(csv_filepath: PathBuf) -> Result<Vec<Allocation>> {
        let allocations_file = File::open(&csv_filepath)
            .with_context(|| format!("cannot open file {csv_filepath:?}"))?;
        Self::from_reader(allocations_file)
    }
    /// Import allocations from a reader object that emits CSV.
    pub fn from_reader(csv_input: impl Read) -> Result<Vec<Allocation>> {
        let mut rdr = csv::Reader::from_reader(csv_input);
        let mut res = vec![];
        for (line, result) in rdr.deserialize().enumerate() {
            let record: NetworkAllocation = result?;
            let record: shielded_pool_genesis::Allocation =
                record.try_into().with_context(|| {
                    format!("invalid allocation in entry {line} of allocations file")
                })?;
            res.push(record);
        }

        if res.is_empty() {
            anyhow::bail!("parsed no entries from allocations input file; is the file valid CSV?");
        }

        Ok(res)
    }
    /// Creates a basic set of genesis [Allocation]s for the provided [Address].
    /// Returns multiple Allocations, so that it's immediately possible to use the DEX,
    /// for basic interactive testing of swap behavior.
    /// For more control over precise allocation amounts, use [from_csv].
    pub fn simple(address: Address) -> Vec<Allocation> {
        vec![
            Allocation {
                address: address.clone(),
                raw_denom: "upenumbra".into(),
                // The `upenumbra` base denom is millionths, so `10^6 * n`
                // results in `n` `penumbra` tokens.
                raw_amount: (100_000 * 10u128.pow(6)).into(),
            },
            Allocation {
                address: address.clone(),
                raw_denom: "test_usd".into(),
                raw_amount: (1_000 as u128).into(),
            },
        ]
    }
}

/// Represents a funding stream within a testnet configuration file.
#[derive(Debug, Deserialize, Clone)]
pub struct TestnetFundingStream {
    pub rate_bps: u16,
    pub address: String,
}

/// Represents testnet validators in configuration files.
#[derive(Deserialize)]
pub struct NetworkValidator {
    pub name: String,
    pub website: String,
    pub description: String,
    pub funding_streams: Vec<TestnetFundingStream>,
    /// All validator identities
    pub sequence_number: u32,
    /// Optional `external_address` field for Tendermint config.
    /// Instructs peers to connect to this node's P2P service
    /// on this address.
    pub external_address: Option<TendermintAddress>,
    pub peer_address_template: Option<String>,
    #[serde(default)]
    pub keys: ValidatorKeys,
}

impl NetworkValidator {
    /// Import validator configs from a JSON file.
    pub fn from_json(json_filepath: PathBuf) -> Result<Vec<NetworkValidator>> {
        let validators_file = File::open(&json_filepath)
            .with_context(|| format!("cannot open file {json_filepath:?}"))?;
        Self::from_reader(validators_file)
    }
    /// Import validator configs from a reader object that emits JSON.
    pub fn from_reader(input: impl Read) -> Result<Vec<NetworkValidator>> {
        Ok(serde_json::from_reader(input)?)
    }
    /// Generate initial delegation allocation for inclusion in genesis.
    pub fn delegation_allocation(&self) -> Result<Allocation> {
        let spend_key = SpendKey::from(self.keys.validator_spend_key.clone());
        let fvk = spend_key.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let identity_key: IdentityKey = IdentityKey(fvk.spend_verification_key().clone().into());
        let delegation_denom = DelegationToken::from(&identity_key).denom();
        Ok(Allocation {
            address: dest,
            // Add an initial allocation of 25,000 delegation tokens,
            // starting them with 2.5x the individual allocations to discord users.
            // 25,000 delegation tokens * 1e6 udelegation factor
            raw_amount: (25_000 * 10u128.pow(6)).into(),
            raw_denom: delegation_denom.to_string(),
        })
    }
    /// Return a URL for Tendermint P2P service for this node.
    ///
    /// In order for the set of genesis validators to communicate with each other,
    /// they must have initial peer information seeded into their Tendermint config files.
    /// If an `external_address` was set, use that. Next, check for a `peer_address_template`.
    /// Finally, fall back to localhost.
    pub fn peering_address(&self) -> anyhow::Result<TendermintAddress> {
        let tm_node_id = node::Id::from(self.keys.node_key_pk.ed25519().expect("ed25519 key"));
        tracing::debug!(?self.name, ?self.external_address, ?self.peer_address_template, "Looking up peering_address");
        let r: TendermintAddress = match &self.external_address {
            // The `external_address` is a TendermintAddress, so unpack as enum to retrieve
            // the host/port info.
            Some(a) => match a {
                TendermintAddress::Tcp {
                    peer_id: _,
                    host,
                    port,
                } => format!("{tm_node_id}@{}:{}", host, port).parse()?,
                // The other enum type is TendermintAddress::Unix, see
                // https://docs.rs/tendermint-config/0.33.0/tendermint_config/index.html
                _ => {
                    anyhow::bail!(
                        "Only TCP format is supported for tendermint addresses: {}",
                        a
                    );
                }
            },
            None => match &self.peer_address_template {
                Some(t) => format!("{tm_node_id}@{t}:26656").parse()?,
                None => format!("{tm_node_id}@127.0.0.1:26656").parse()?,
            },
        };
        Ok(r)
    }

    /// Hardcoded initial state for Tendermint, used for writing configs.
    // Easiest to hardcode since we never change these.
    pub fn initial_state() -> String {
        r#"{
        "height": "0",
        "round": 0,
        "step": 0
    }
    "#
        .to_string()
    }
}

impl Default for NetworkValidator {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            website: "".to_string(),
            description: "".to_string(),
            funding_streams: Vec::<TestnetFundingStream>::new(),
            sequence_number: 0,
            external_address: None,
            peer_address_template: None,
            keys: ValidatorKeys::generate(),
        }
    }
}

// The core Penumbra logic deals with `Validator`s, to make sure our convenient
// wrapper type can resolve as a `Validator` when needed.
impl TryFrom<&NetworkValidator> for Validator {
    type Error = anyhow::Error;
    fn try_from(tv: &NetworkValidator) -> anyhow::Result<Validator> {
        // Validation:
        // - Website has a max length of 70 bytes
        if tv.website.len() > 70 {
            anyhow::bail!("validator website field must be less than 70 bytes");
        }

        // - Name has a max length of 140 bytes
        if tv.name.len() > 140 {
            anyhow::bail!("validator name must be less than 140 bytes");
        }

        // - Description has a max length of 280 bytes
        if tv.description.len() > 280 {
            anyhow::bail!("validator description must be less than 280 bytes");
        }

        Ok(Validator {
            // Currently there's no way to set validator keys beyond
            // manually editing the genesis.json. Otherwise they
            // will be randomly generated keys.
            identity_key: IdentityKey(tv.keys.validator_id_vk.into()),
            governance_key: GovernanceKey(tv.keys.validator_id_vk),
            consensus_key: tv.keys.validator_cons_pk,
            name: tv.name.clone(),
            website: tv.website.clone(),
            description: tv.description.clone(),
            enabled: true,
            funding_streams: FundingStreams::try_from(
                tv.funding_streams
                    .iter()
                    .map(|fs| {
                        Ok(FundingStream::ToAddress {
                            address: Address::from_str(&fs.address)
                                .context("invalid funding stream address in validators.json")?,
                            rate_bps: fs.rate_bps,
                        })
                    })
                    .collect::<Result<Vec<FundingStream>, anyhow::Error>>()?,
            )
            .context("unable to construct funding streams from validators.json")?,
            sequence_number: tv.sequence_number,
        })
    }
}

impl TryFrom<NetworkAllocation> for shielded_pool_genesis::Allocation {
    type Error = anyhow::Error;

    fn try_from(a: NetworkAllocation) -> anyhow::Result<shielded_pool_genesis::Allocation> {
        Ok(shielded_pool_genesis::Allocation {
            raw_amount: a.amount.into(),
            raw_denom: a.denom.clone(),
            address: Address::from_str(&a.address).with_context(|| {
                format!(
                    "invalid address format in genesis allocations: {}",
                    &a.address
                )
            })?,
        })
    }
}

fn string_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct U128StringVisitor;

    impl<'de> de::Visitor<'de> for U128StringVisitor {
        type Value = u128;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a u128 with optional underscores")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let r = v.replace('_', "");
            r.parse::<u128>().map_err(E::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v as u128)
        }

        fn visit_u128<E>(self, v: u128) -> std::prelude::v1::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }

    deserializer.deserialize_any(U128StringVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_allocations_from_good_csv() -> anyhow::Result<()> {
        let csv_content = r#"
"amount","denom","address"
"100000","udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex","penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk"
"100000","upenumbra","penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk"
"100000","udelegation_penumbravalid1p2hfuch2p8rshyc90qa23gqk82s74tqcu3x2x3y5tfwpzth4vvrq2gv283","penumbra1xq2e9x7uhfzezwunvazdamlxepf4jr5htsuqnzlsahuayyqxjjwg9lk0aytwm6wfj3jy29rv2kdpen57903s8wxv3jmqwj6m6v5jgn6y2cypfd03rke652k8wmavxra7e9wkrg"
"100000","upenumbra","penumbra1xq2e9x7uhfzezwunvazdamlxepf4jr5htsuqnzlsahuayyqxjjwg9lk0aytwm6wfj3jy29rv2kdpen57903s8wxv3jmqwj6m6v5jgn6y2cypfd03rke652k8wmavxra7e9wkrg"
"100000","udelegation_penumbravalid182k8x46hg5vx3ez8ec58ze5yd6a3q4q3fkx45ddt5jahnzz0xyyqdtz7hc","penumbra100zd92fg6x27wc0mlu48cd6phq420u7ep59kzdalg2cq66mjkyl0xr54z0c64gectnj44mv5k2vyjjsz5gyd5gq33a6wnqzvgu2fz7namz7usazsl6p8wza83gcpwt8q76rc4y"
"100000","upenumbra","penumbra100zd92fg6x27wc0mlu48cd6phq420u7ep59kzdalg2cq66mjkyl0xr54z0c64gectnj44mv5k2vyjjsz5gyd5gq33a6wnqzvgu2fz7namz7usazsl6p8wza83gcpwt8q76rc4y"
"100000","udelegation_penumbravalid1t2hr2lj5n2jt3hftzjw3strjllnakc7jtw234d229x3zakhaqsqsg9yarw","penumbra1xap8sgefy9rl2nfvsse0h4y6c25hy2n20ymr5w7hs28m9xemt3tmz88atyulswumc32sv7h937wnfhyct282de66zm75nk6ywq3d4r32p5ju0cnscj2rraesnrxr9lvk6hcazp"
"100000","upenumbra","penumbra1xap8sgefy9rl2nfvsse0h4y6c25hy2n20ymr5w7hs28m9xemt3tmz88atyulswumc32sv7h937wnfhyct282de66zm75nk6ywq3d4r32p5ju0cnscj2rraesnrxr9lvk6hcazp"
"#;
        let allos = NetworkAllocation::from_reader(csv_content.as_bytes())?;

        let a1 = &allos[0];
        assert!(a1.raw_denom == "udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex");
        assert!(a1.address == Address::from_str("penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk")?);
        assert!(a1.raw_amount.value() == 100000);

        let a2 = &allos[1];
        assert!(a2.raw_denom == "upenumbra");
        assert!(a2.address == Address::from_str("penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk")?);
        assert!(a2.raw_amount.value() == 100000);

        Ok(())
    }

    #[test]
    fn parse_allocations_from_bad_csv() -> anyhow::Result<()> {
        let csv_content = r#"
"amount","denom","address"\n"100000","udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex","penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk"\n"100000","upenumbra","penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk"\n"100000","udelegation_penumbravalid1p2hfuch2p8rshyc90qa23gqk82s74tqcu3x2x3y5tfwpzth4vvrq2gv283","penumbra1xq2e9x7uhfzezwunvazdamlxepf4jr5htsuqnzlsahuayyqxjjwg9lk0aytwm6wfj3jy29rv2kdpen57903s8wxv3jmqwj6m6v5jgn6y2cypfd03rke652k8wmavxra7e9wkrg"\n"100000","upenumbra","penumbra1xq2e9x7uhfzezwunvazdamlxepf4jr5htsuqnzlsahuayyqxjjwg9lk0aytwm6wfj3jy29rv2kdpen57903s8wxv3jmqwj6m6v5jgn6y2cypfd03rke652k8wmavxra7e9wkrg"\n"100000","udelegation_penumbravalid182k8x46hg5vx3ez8ec58ze5yd6a3q4q3fkx45ddt5jahnzz0xyyqdtz7hc","penumbra100zd92fg6x27wc0mlu48cd6phq420u7ep59kzdalg2cq66mjkyl0xr54z0c64gectnj44mv5k2vyjjsz5gyd5gq33a6wnqzvgu2fz7namz7usazsl6p8wza83gcpwt8q76rc4y"\n"100000","upenumbra","penumbra100zd92fg6x27wc0mlu48cd6phq420u7ep59kzdalg2cq66mjkyl0xr54z0c64gectnj44mv5k2vyjjsz5gyd5gq33a6wnqzvgu2fz7namz7usazsl6p8wza83gcpwt8q76rc4y"\n"100000","udelegation_penumbravalid1t2hr2lj5n2jt3hftzjw3strjllnakc7jtw234d229x3zakhaqsqsg9yarw","penumbra1xap8sgefy9rl2nfvsse0h4y6c25hy2n20ymr5w7hs28m9xemt3tmz88atyulswumc32sv7h937wnfhyct282de66zm75nk6ywq3d4r32p5ju0cnscj2rraesnrxr9lvk6hcazp"\n"100000","upenumbra","penumbra1xap8sgefy9rl2nfvsse0h4y6c25hy2n20ymr5w7hs28m9xemt3tmz88atyulswumc32sv7h937wnfhyct282de66zm75nk6ywq3d4r32p5ju0cnscj2rraesnrxr9lvk6hcazp"\n
"#;
        let result = NetworkAllocation::from_reader(csv_content.as_bytes());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    /// Generate a config suitable for local testing: no custom address information, no additional
    /// validators at genesis.
    fn generate_devnet_config() -> anyhow::Result<()> {
        let testnet_config = NetworkConfig::generate(
            "test-chain-1234",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )?;
        assert_eq!(testnet_config.name, "test-chain-1234");
        assert_eq!(testnet_config.genesis.validators.len(), 0);
        // No external address template was given, so only 1 validator will be present.
        let penumbra_sdk_app::genesis::AppState::Content(app_state) =
            testnet_config.genesis.app_state
        else {
            unimplemented!("TODO: support checkpointed app state")
        };
        assert_eq!(app_state.stake_content.validators.len(), 1);
        Ok(())
    }

    #[test]
    /// Generate a config suitable for a public testnet: custom validators input file,
    /// increasing the default validators from 1 -> 2.
    fn generate_network_config() -> anyhow::Result<()> {
        let ci_validators_filepath = PathBuf::from("../../../testnets/validators-ci.json");
        let testnet_config = NetworkConfig::generate(
            "test-chain-4567",
            None,
            Some(String::from("validator.local")),
            None,
            None,
            None,
            Some(ci_validators_filepath),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;
        assert_eq!(testnet_config.name, "test-chain-4567");
        assert_eq!(testnet_config.genesis.validators.len(), 0);
        let penumbra_sdk_app::genesis::AppState::Content(app_state) =
            testnet_config.genesis.app_state
        else {
            unimplemented!("TODO: support checkpointed app state")
        };
        assert_eq!(app_state.stake_content.validators.len(), 2);
        Ok(())
    }

    //    #[test]
    //    fn testnet_validator_to_validator_conversion() -> anyhow::Result<()> {
    //        let tv = NetworkValidator::default();
    //        let v: Validator = tv.try_into()?;
    //        assert!(v.website == "");
    //        Ok(())
    //    }
}
