//! Logic for creating a new testnet configuration.
//! Used for deploying (approximately weekly) testnets
//! for Penumbra.
use crate::testnet::config::{get_testnet_dir, TestnetTendermintConfig, ValidatorKeys};
use anyhow::{Context, Result};
use penumbra_chain::genesis;
use penumbra_chain::{genesis::Allocation, params::ChainParameters};
use penumbra_keys::{keys::SpendKey, Address};
use penumbra_stake::{
    validator::Validator, DelegationToken, FundingStream, FundingStreams, GovernanceKey,
    IdentityKey,
};
use serde::{de, Deserialize};
use tendermint::consensus::params::AbciParams;
use std::{
    fmt,
    fs::File,
    io::Read,
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tendermint::{node, public_key::Algorithm, Genesis, Time};
use tendermint_config::net::Address as TendermintAddress;

/// Represents a Penumbra network config, including initial validators
/// and allocations at genesis time.
pub struct TestnetConfig {
    /// The name of the network
    pub name: String,
    /// The Tendermint genesis for initial chain state.
    pub genesis: Genesis<genesis::AppState>,
    /// Path to local directory where config files will be written to
    pub testnet_dir: PathBuf,
    /// Set of validators at genesis. Uses the convenient wrapper type
    /// to generate config files.
    pub testnet_validators: Vec<TestnetValidator>,
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

impl TestnetConfig {
    /// Create a new testnet configuration, optionally customizing the allocations and validator
    /// set. By default, will use the prepared Discord allocations and Penumbra Labs CI validator
    /// configs.
    pub fn generate(
        chain_id: &str,
        testnet_dir: Option<PathBuf>,
        peer_address_template: Option<String>,
        external_addresses: Option<Vec<TendermintAddress>>,
        allocations_input_file: Option<PathBuf>,
        validators_input_file: Option<PathBuf>,
        tendermint_timeout_commit: Option<tendermint::Timeout>,
        active_validator_limit: Option<u64>,
        epoch_duration: Option<u64>,
        unbonding_epochs: Option<u64>,
    ) -> anyhow::Result<TestnetConfig> {
        let external_addresses = external_addresses.unwrap_or(Vec::new());

        let testnet_validators = Self::collect_validators(
            validators_input_file,
            peer_address_template.clone(),
            external_addresses,
        )?;

        let mut allocations = Self::collect_allocations(allocations_input_file)?;

        for v in testnet_validators.iter() {
            allocations.push(v.delegation_allocation()?);
        }

        // Convert to domain type, for use with other Penumbra interfaces.
        // We do this conversion once and store it in the struct for convenience.
        let validators: anyhow::Result<Vec<Validator>> =
            testnet_validators.iter().map(|v| v.try_into()).collect();
        let validators = validators?;

        let app_state = Self::make_appstate(
            chain_id,
            allocations,
            validators.to_vec(),
            active_validator_limit,
            epoch_duration,
            unbonding_epochs,
        )?;
        let genesis = Self::make_genesis(app_state)?;

        Ok(TestnetConfig {
            name: chain_id.to_owned(),
            genesis,
            testnet_dir: get_testnet_dir(testnet_dir),
            testnet_validators,
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
    ) -> anyhow::Result<Vec<TestnetValidator>> {
        let testnet_validators = if let Some(validators_input_file) = validators_input_file {
            TestnetValidator::from_json(validators_input_file)?
        } else {
            static LATEST_VALIDATORS: &str = include_str!(env!("PD_LATEST_TESTNET_VALIDATORS"));
            TestnetValidator::from_reader(std::io::Cursor::new(LATEST_VALIDATORS)).with_context(
                || {
                    format!(
                        "could not parse default latest testnet validators file {:?}",
                        env!("PD_LATEST_TESTNET_VALIDATORS")
                    )
                },
            )?
        };

        if external_addresses.len() > 0 {
            if external_addresses.len() != testnet_validators.len() {
                anyhow::bail!("Number of validators did not equal number of external addresses");
            }
        }

        Ok(testnet_validators
            .into_iter()
            .enumerate()
            .map(|(i, v)| TestnetValidator {
                peer_address_template: match &peer_address_template {
                    Some(t) => Some(format!("{t}-{i}")),
                    None => None,
                },
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
                TestnetAllocation::from_csv(allocations_input_file.to_path_buf()).with_context(
                    || format!("could not parse allocations file {allocations_input_file:?}"),
                )?,
            )
        } else {
            // Default to latest testnet allocations computed in the build script.
            static LATEST_ALLOCATIONS: &str = include_str!(env!("PD_LATEST_TESTNET_ALLOCATIONS"));
            Ok(
                TestnetAllocation::from_reader(std::io::Cursor::new(LATEST_ALLOCATIONS))
                    .with_context(|| {
                        format!(
                            "could not parse default latest testnet allocations file {:?}",
                            env!("PD_LATEST_TESTNET_ALLOCATIONS")
                        )
                    })?,
            )
        }
    }

    /// Build initial state for Penumbra application, for inclusion in Tendermint genesis.
    fn make_appstate(
        chain_id: &str,
        allocations: Vec<Allocation>,
        validators: Vec<Validator>,
        active_validator_limit: Option<u64>,
        epoch_duration: Option<u64>,
        unbonding_epochs: Option<u64>,
    ) -> anyhow::Result<genesis::AppState> {
        // Look up default chain params, so we can fill in defaults.
        let default_params = ChainParameters::default();
        let app_state = genesis::AppState {
            allocations: allocations.clone(),
            chain_params: ChainParameters {
                chain_id: chain_id.to_string(),
                // Fall back to chain param defaults
                active_validator_limit: active_validator_limit
                    .unwrap_or(default_params.active_validator_limit),
                epoch_duration: epoch_duration.unwrap_or(default_params.epoch_duration),
                unbonding_epochs: unbonding_epochs.unwrap_or(default_params.unbonding_epochs),
                ..Default::default()
            },
            // Convert to protobuf types
            validators: validators.into_iter().map(Into::into).collect(),
        };
        Ok(app_state)
    }

    /// Build Tendermint genesis data, based on Penumbra initial application state.
    pub(crate) fn make_genesis(
        app_state: genesis::AppState,
    ) -> anyhow::Result<Genesis<genesis::AppState>> {
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
                .chain_params
                .chain_id
                .parse::<tendermint::chain::Id>()
                .context("failed to parseto create chain ID")?,
            initial_height: 0,
            consensus_params: tendermint::consensus::Params {
                abci: AbciParams::default(),
                block: tendermint::block::Size {
                    max_bytes: 22020096,
                    max_gas: -1,
                    // minimum time increment between consecutive blocks
                    time_iota_ms: 500,
                },
                // TODO Should these correspond with values used within `pd` for penumbra epochs?
                evidence: tendermint::evidence::Params {
                    max_age_num_blocks: 100000,
                    // 1 day
                    max_age_duration: tendermint::evidence::Duration(Duration::new(86400, 0)),
                    max_bytes: 1048576,
                },
                validator: tendermint::consensus::params::ValidatorParams {
                    pub_key_types: vec![Algorithm::Ed25519],
                },
                version: Some(tendermint::consensus::params::VersionParams { app: 0 }),
            },
            // always empty in genesis json
            app_hash: tendermint::AppHash::default(),
            app_state,
            // Set empty validator set for Tendermint config, which falls back to reading
            // validators from the AppState, via ResponseInitChain:
            // https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
            validators: vec![],
        };
        Ok(genesis)
    }

    /// Generate and write to disk the Tendermint configs for each validator at genesis.
    pub fn write_configs(&self) -> anyhow::Result<()> {
        // Loop over each validator and write its config separately.
        for (n, v) in self.testnet_validators.iter().enumerate() {
            // Create the directory for this node
            let node_name = format!("node{n}");
            let node_dir = self.testnet_dir.clone().join(node_name.clone());

            // Each node should include only the IPs for *other* nodes in their peers list.
            let ips_minus_mine: anyhow::Result<Vec<TendermintAddress>> = self
                .testnet_validators
                .iter()
                .map(|v| v.peering_address())
                .filter(|a| *a.as_ref().unwrap() != v.peering_address().unwrap())
                .collect();
            let ips_minus_mine = ips_minus_mine?;
            tracing::debug!(?ips_minus_mine, "Found these peer ips");

            let external_address: Option<TendermintAddress> = match &v.external_address {
                Some(e) => Some(e.clone()),
                None => None,
            };
            let mut tm_config = TestnetTendermintConfig::new(
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
/// defaulting to `~/.penumbra/testnet_data`, as usual.
#[allow(clippy::too_many_arguments)]
pub fn testnet_generate(
    testnet_dir: PathBuf,
    chain_id: &str,
    active_validator_limit: Option<u64>,
    tendermint_timeout_commit: Option<tendermint::Timeout>,
    epoch_duration: Option<u64>,
    unbonding_epochs: Option<u64>,
    peer_address_template: Option<String>,
    external_addresses: Vec<TendermintAddress>,
    validators_input_file: Option<PathBuf>,
    allocations_input_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    tracing::info!(?chain_id, "Generating network config");
    let t = TestnetConfig::generate(
        chain_id,
        Some(testnet_dir),
        peer_address_template,
        Some(external_addresses),
        allocations_input_file,
        validators_input_file,
        tendermint_timeout_commit,
        active_validator_limit,
        epoch_duration,
        unbonding_epochs,
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
pub struct TestnetAllocation {
    #[serde(deserialize_with = "string_u64")]
    pub amount: u64,
    pub denom: String,
    pub address: String,
}

impl TestnetAllocation {
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
            let record: TestnetAllocation = result?;
            let record: genesis::Allocation = record.try_into().with_context(|| {
                format!("invalid allocation in entry {line} of allocations file")
            })?;
            res.push(record);
        }

        if res.is_empty() {
            anyhow::bail!("parsed no entries from allocations input file; is the file valid CSV?");
        }

        Ok(res)
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
pub struct TestnetValidator {
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

impl TestnetValidator {
    /// Import validator configs from a JSON file.
    pub fn from_json(json_filepath: PathBuf) -> Result<Vec<TestnetValidator>> {
        let validators_file = File::open(&json_filepath)
            .with_context(|| format!("cannot open file {json_filepath:?}"))?;
        Self::from_reader(validators_file)
    }
    /// Import validator configs from a reader object that emits JSON.
    pub fn from_reader(input: impl Read) -> Result<Vec<TestnetValidator>> {
        Ok(serde_json::from_reader(input)?)
    }
    /// Generate iniital delegation allocation for inclusion in genesis.
    pub fn delegation_allocation(&self) -> Result<Allocation> {
        let spend_key = SpendKey::from(self.keys.validator_spend_key.clone());
        let fvk = spend_key.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let identity_key: IdentityKey = IdentityKey(fvk.spend_verification_key().clone());
        let delegation_denom = DelegationToken::from(&identity_key).denom();
        Ok(Allocation {
            address: dest,
            // Add an initial allocation of 25,000 delegation tokens,
            // starting them with 2.5x the individual allocations to discord users.
            // 25,000 delegation tokens * 1e6 udelegation factor
            amount: (25_000 * 10u128.pow(6)).into(),
            denom: delegation_denom.to_string(),
        })
    }
    /// Return a URL for Tendermint P2P service for this node.
    ///
    /// In order for the set of genesis validators to communicate with each other,
    /// they must have initial peer information seeded into their Tendermint config files.
    /// If an `external_address` was set, use that. Next, check for a `peer_address_template`.
    /// Finally, fall back to localhost.
    pub fn peering_address(&self) -> anyhow::Result<TendermintAddress> {
        let tm_node_id = node::Id::from(self.keys.node_key_pk.ed25519().unwrap());
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

impl Default for TestnetValidator {
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
impl TryFrom<&TestnetValidator> for Validator {
    type Error = anyhow::Error;
    fn try_from(tv: &TestnetValidator) -> anyhow::Result<Validator> {
        Ok(Validator {
            // Currently there's no way to set validator keys beyond
            // manually editing the genesis.json. Otherwise they
            // will be randomly generated keys.
            identity_key: IdentityKey(tv.keys.validator_id_vk),
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

impl TryFrom<TestnetAllocation> for genesis::Allocation {
    type Error = anyhow::Error;

    fn try_from(a: TestnetAllocation) -> anyhow::Result<genesis::Allocation> {
        Ok(genesis::Allocation {
            amount: a.amount.into(),
            denom: a.denom.clone(),
            address: Address::from_str(&a.address)
                .context("invalid address format in genesis allocations")?,
        })
    }
}

fn string_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct U64StringVisitor;

    impl<'de> de::Visitor<'de> for U64StringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a u64 with optional underscores")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let r = v.replace('_', "");
            r.parse::<u64>().map_err(E::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }

    deserializer.deserialize_any(U64StringVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_allocations_from_good_csv() -> anyhow::Result<()> {
        let csv_content = r#"
"amount","denom","address"
"100000","udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex","penumbrav2t1tntrwl92wmud955405mhduuvlxqksa00d2yqe3npjafvley64pal4sre3jcjhq5xjmrs56hk2fs8u648zcaarnz57rynqa0vtaxyd6rwev225lx4kku3lrjktrcacjyw5070nj"
"100000","upenumbra","penumbrav2t1tntrwl92wmud955405mhduuvlxqksa00d2yqe3npjafvley64pal4sre3jcjhq5xjmrs56hk2fs8u648zcaarnz57rynqa0vtaxyd6rwev225lx4kku3lrjktrcacjyw5070nj"
"100000","udelegation_penumbravalid1p2hfuch2p8rshyc90qa23gqk82s74tqcu3x2x3y5tfwpzth4vvrq2gv283","penumbrav2t1ctq4cm9fjewj790alfka634n2t32vh32vqnfufw2dpegw7c2a3lw9np2f4pcthl2w2ke4a32cdmmnurd95sjreu92vey3fwj9ccjz2hpudd9kz9xqlwqp39sly8knl0e2esqjw"
"100000","upenumbra","penumbrav2t1ctq4cm9fjewj790alfka634n2t32vh32vqnfufw2dpegw7c2a3lw9np2f4pcthl2w2ke4a32cdmmnurd95sjreu92vey3fwj9ccjz2hpudd9kz9xqlwqp39sly8knl0e2esqjw"
"100000","udelegation_penumbravalid182k8x46hg5vx3ez8ec58ze5yd6a3q4q3fkx45ddt5jahnzz0xyyqdtz7hc","penumbrav2t1ks2ee68kf96zs3p2af2pzcu7p36uep5gynwc38slvs8kpcyk0t0gdseww4aulntzaq9vurahmka99c4frpgehtteur29p5kt39a2qv0nd89etty36s55knlv7e98kl93p8farz"
"100000","upenumbra","penumbrav2t1ks2ee68kf96zs3p2af2pzcu7p36uep5gynwc38slvs8kpcyk0t0gdseww4aulntzaq9vurahmka99c4frpgehtteur29p5kt39a2qv0nd89etty36s55knlv7e98kl93p8farz"
"100000","udelegation_penumbravalid1t2hr2lj5n2jt3hftzjw3strjllnakc7jtw234d229x3zakhaqsqsg9yarw","penumbrav2t1wvjml4xqa70x3ypqa530npy8ygsftyjxc2wfgh4t7yftja7cfrlr7temqcerhnkd6g7qe75r7wg0ny9vvf4wcrd9rttvuhj9fy20yx630g26khmnxd2zvjnhm2t3wqu59e7nzk"
"100000","upenumbra","penumbrav2t1wvjml4xqa70x3ypqa530npy8ygsftyjxc2wfgh4t7yftja7cfrlr7temqcerhnkd6g7qe75r7wg0ny9vvf4wcrd9rttvuhj9fy20yx630g26khmnxd2zvjnhm2t3wqu59e7nzk"
"#;
        let allos = TestnetAllocation::from_reader(csv_content.as_bytes())?;

        let a1 = &allos[0];
        assert!(a1.denom == "udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex");
        assert!(a1.address == Address::from_str("penumbrav2t1tntrwl92wmud955405mhduuvlxqksa00d2yqe3npjafvley64pal4sre3jcjhq5xjmrs56hk2fs8u648zcaarnz57rynqa0vtaxyd6rwev225lx4kku3lrjktrcacjyw5070nj")?);
        assert!(a1.amount.value() == 100000);

        let a2 = &allos[1];
        assert!(a2.denom == "upenumbra");
        assert!(a2.address == Address::from_str("penumbrav2t1tntrwl92wmud955405mhduuvlxqksa00d2yqe3npjafvley64pal4sre3jcjhq5xjmrs56hk2fs8u648zcaarnz57rynqa0vtaxyd6rwev225lx4kku3lrjktrcacjyw5070nj")?);
        assert!(a2.amount.value() == 100000);

        Ok(())
    }

    #[test]
    fn parse_allocations_from_bad_csv() -> anyhow::Result<()> {
        let csv_content = r#"
"amount","denom","address"\n"100000","udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex","penumbrav2t1tntrwl92wmud955405mhduuvlxqksa00d2yqe3npjafvley64pal4sre3jcjhq5xjmrs56hk2fs8u648zcaarnz57rynqa0vtaxyd6rwev225lx4kku3lrjktrcacjyw5070nj"\n"100000","upenumbra","penumbrav2t1tntrwl92wmud955405mhduuvlxqksa00d2yqe3npjafvley64pal4sre3jcjhq5xjmrs56hk2fs8u648zcaarnz57rynqa0vtaxyd6rwev225lx4kku3lrjktrcacjyw5070nj"\n"100000","udelegation_penumbravalid1p2hfuch2p8rshyc90qa23gqk82s74tqcu3x2x3y5tfwpzth4vvrq2gv283","penumbrav2t1ctq4cm9fjewj790alfka634n2t32vh32vqnfufw2dpegw7c2a3lw9np2f4pcthl2w2ke4a32cdmmnurd95sjreu92vey3fwj9ccjz2hpudd9kz9xqlwqp39sly8knl0e2esqjw"\n"100000","upenumbra","penumbrav2t1ctq4cm9fjewj790alfka634n2t32vh32vqnfufw2dpegw7c2a3lw9np2f4pcthl2w2ke4a32cdmmnurd95sjreu92vey3fwj9ccjz2hpudd9kz9xqlwqp39sly8knl0e2esqjw"\n"100000","udelegation_penumbravalid182k8x46hg5vx3ez8ec58ze5yd6a3q4q3fkx45ddt5jahnzz0xyyqdtz7hc","penumbrav2t1ks2ee68kf96zs3p2af2pzcu7p36uep5gynwc38slvs8kpcyk0t0gdseww4aulntzaq9vurahmka99c4frpgehtteur29p5kt39a2qv0nd89etty36s55knlv7e98kl93p8farz"\n"100000","upenumbra","penumbrav2t1ks2ee68kf96zs3p2af2pzcu7p36uep5gynwc38slvs8kpcyk0t0gdseww4aulntzaq9vurahmka99c4frpgehtteur29p5kt39a2qv0nd89etty36s55knlv7e98kl93p8farz"\n"100000","udelegation_penumbravalid1t2hr2lj5n2jt3hftzjw3strjllnakc7jtw234d229x3zakhaqsqsg9yarw","penumbrav2t1wvjml4xqa70x3ypqa530npy8ygsftyjxc2wfgh4t7yftja7cfrlr7temqcerhnkd6g7qe75r7wg0ny9vvf4wcrd9rttvuhj9fy20yx630g26khmnxd2zvjnhm2t3wqu59e7nzk"\n"100000","upenumbra","penumbrav2t1wvjml4xqa70x3ypqa530npy8ygsftyjxc2wfgh4t7yftja7cfrlr7temqcerhnkd6g7qe75r7wg0ny9vvf4wcrd9rttvuhj9fy20yx630g26khmnxd2zvjnhm2t3wqu59e7nzk"\n
"#;
        let result = TestnetAllocation::from_reader(csv_content.as_bytes());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    /// Generate a config suitable for local testing: no custom address information, no additional
    /// validators at genesis.
    fn generate_devnet_config() -> anyhow::Result<()> {
        let testnet_config = TestnetConfig::generate(
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
        )?;
        assert_eq!(testnet_config.name, "test-chain-1234");
        assert_eq!(testnet_config.genesis.validators.len(), 0);
        // No external address template was given, so only 1 validator will be present.
        assert_eq!(testnet_config.genesis.app_state.validators.len(), 1);
        Ok(())
    }

    #[test]
    /// Generate a config suitable for a public testnet: custom validators input file,
    /// increasing the default validators from 1 -> 2.
    fn generate_testnet_config() -> anyhow::Result<()> {
        let ci_validators_filepath = PathBuf::from("../../../testnets/validators-ci.json");
        let testnet_config = TestnetConfig::generate(
            "test-chain-4567",
            None,
            Some(String::from("validator.local")),
            None,
            None,
            Some(ci_validators_filepath),
            None,
            None,
            None,
            None,
        )?;
        assert_eq!(testnet_config.name, "test-chain-4567");
        assert_eq!(testnet_config.genesis.validators.len(), 0);
        assert_eq!(testnet_config.genesis.app_state.validators.len(), 2);
        Ok(())
    }

    //    #[test]
    //    fn testnet_validator_to_validator_conversion() -> anyhow::Result<()> {
    //        let tv = TestnetValidator::default();
    //        let v: Validator = tv.try_into()?;
    //        assert!(v.website == "");
    //        Ok(())
    //    }
}
