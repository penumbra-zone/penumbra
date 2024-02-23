//! Create genesis configs and corresponding genesis validators.
use crate::config::get_testnet_dir;
use rand::Rng;
use rand_core::OsRng;

use crate::fullnode::PenumbraNode;
use crate::validator::PenumbraValidator;
use anyhow::{Context, Result};
use penumbra_app::params::AppParameters;
use penumbra_governance::genesis::Content as GovernanceContent;
use penumbra_keys::Address;
use penumbra_sct::genesis::Content as SctContent;
use penumbra_sct::params::SctParameters;
use penumbra_shielded_pool::{
    genesis::{self as shielded_pool_genesis, Allocation, Content as ShieldedPoolContent},
    params::ShieldedPoolParameters,
};
use penumbra_stake::{
    genesis::Content as StakeContent, params::StakeParameters, validator::Validator,
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
use tendermint::{public_key::Algorithm, Genesis as CometBFTGenesis, Time};
use tendermint_config::net::Address as TendermintAddress;

/// Represents a Penumbra network config, including initial validators
/// and allocations at genesis time.
pub struct PenumbraNetwork {
    /// The name of the network.
    pub chain_id: String,
    /// Path to local directory where config files will be written to
    pub testnet_dir: PathBuf,
    /// The set of initial validators at genesis time.
    pub genesis_validators: Vec<PenumbraValidator>,
    /// Chain parameters and other settings, for determining chain state.
    pub net_config: PenumbraNetworkConfig,
}

/// Overridable behavior for a [PenumbraNetwork].
pub struct PenumbraNetworkConfig {
    /// The CometBFT `consensus.timeout_commit` value, controlling how long CometBFT should
    /// wait after committing a block, before starting on the new height. If unspecified, `5s`.
    pub tendermint_timeout_commit: tendermint::Timeout,
    /// Maximum number of validators that can occupy the active set.
    pub active_validator_limit: u64,
    /// How many blocks constitute an epoch.
    pub epoch_duration: u64,
    /// How many epochs must elapse in order for staked tokens to be released after undelegation.
    pub unbonding_delay: u64,
    /// How many blocks must elapse during voting on a governance proposal.
    pub proposal_voting_blocks: u64,
    /// Optional path to JSON file containing validator information.
    pub validators_input_file: Option<PathBuf>,
    /// Optional path to CSV file containing initial allocations for inclusion in genesis.
    pub allocations_input_file: Option<PathBuf>,

    /// Hostname as string for a validator's p2p service. Will have
    /// numbers affixed to it for each validator, e.g. "-0", "-1", etc.
    // TODO: is this used anywhere?
    pub peer_address_template: Option<String>,
    pub external_addresses: Vec<TendermintAddress>,
}

impl Default for PenumbraNetwork {
    fn default() -> Self {
        let chain_id = Self::generate_chain_id(None, true);
        let net_config = PenumbraNetworkConfig::default();
        let genesis_validators = Self::generate_genesis_validators(&net_config)
            .expect("failed to generate genesis validators for PenumbraNetwork defaults");
        Self {
            chain_id: chain_id.to_owned(),
            testnet_dir: get_testnet_dir(None),
            genesis_validators,
            net_config,
        }
    }
}

impl Default for PenumbraNetworkConfig {
    fn default() -> Self {
        // Look up default app params, so we can retrieve nested default parameters.
        let default_app_params = AppParameters::default();
        Self {
            // Default commit timeout is 5s.
            tendermint_timeout_commit: tendermint::Timeout::from(std::time::Duration::new(5, 0)),
            proposal_voting_blocks: penumbra_governance::params::GovernanceParameters::default()
                .proposal_voting_blocks,
            active_validator_limit: default_app_params.stake_params.active_validator_limit,
            unbonding_delay: default_app_params.stake_params.unbonding_delay,
            epoch_duration: default_app_params.sct_params.epoch_duration,
            validators_input_file: None,
            allocations_input_file: None,
            peer_address_template: None,
            external_addresses: Vec::<TendermintAddress>::new(),
        }
    }
}

impl PenumbraNetwork {
    /// Create a new [PenumbraNetwork], based on a [PenumbraNetworkConfig].
    pub fn new(
        chain_id: Option<String>,
        net_config: PenumbraNetworkConfig,
    ) -> anyhow::Result<Self> {
        let c = chain_id.unwrap_or(Self::generate_chain_id(None, true));
        let genesis_validators = Self::generate_genesis_validators(&net_config)
            .context("failed to generate genesis validators for PenumbraNetwork constructor")?;
        Ok(Self {
            chain_id: c,
            net_config,
            genesis_validators,
            ..Default::default()
        })
    }

    /// Create a unique chain_id for the network, based on testnet names read at build time.
    /// Will take the form `penumbra-testnet-<name>-<uuid>`.
    pub fn generate_chain_id(chain_name: Option<String>, randomize: bool) -> String {
        let mut c = match chain_name {
            Some(c) => c,
            None => Self::latest_testnet_name(),
        };

        if randomize {
            let randomizer = OsRng.gen::<u32>();
            c = format!("{}-{}", c, hex::encode(randomizer.to_le_bytes()))
        }
        c
    }

    /// Return the chain_id for the most recent Penumbra testnet.
    pub fn latest_testnet_name() -> String {
        // Build script computes the latest testnet name and sets it as an env variable
        env!("PD_LATEST_TESTNET_NAME").to_string()
    }

    /// Prepare set of initial validators present at genesis. If `validators_input_file` was set,
    /// will use that JSON filepath to load validator info. Otherwise, falls back to
    /// the Penumbra Labs CI validator configs used for testnets.
    pub fn generate_genesis_validators(
        net_config: &PenumbraNetworkConfig,
    ) -> anyhow::Result<Vec<PenumbraValidator>> {
        let vals = match net_config.validators_input_file.clone() {
            Some(f) => {
                tracing::debug!(?f, "importing genesis validators from json file");
                PenumbraValidator::from_json_file(f)?
            }
            None => {
                tracing::debug!(
                    "no json file, reusing testnet validator info for genesis validators"
                );
                static LATEST_VALIDATORS: &str = include_str!(env!("PD_LATEST_TESTNET_VALIDATORS"));
                PenumbraValidator::from_reader(std::io::Cursor::new(LATEST_VALIDATORS))
                    .with_context(|| {
                        format!(
                            "could not parse default latest testnet validators file {:?}",
                            env!("PD_LATEST_TESTNET_VALIDATORS")
                        )
                    })?
            }
        };

        if !net_config.external_addresses.is_empty()
            && net_config.external_addresses.len() != vals.len()
        {
            anyhow::bail!("Number of validators did not equal number of external addresses");
        }

        Ok(vals
            .into_iter()
            .enumerate()
            .map(|(i, v)| PenumbraValidator {
                peer_address_template: net_config
                    .peer_address_template
                    .as_ref()
                    .map(|t| format!("{t}-{i}")),
                fullnode: PenumbraNode {
                    external_address: net_config.external_addresses.get(i).cloned(),
                    ..v.fullnode
                },
                ..v
            })
            .collect())
    }

    /// Returns the CometBFT genesis for initial chain state, based on network config.
    pub fn genesis(&self) -> anyhow::Result<CometBFTGenesis<penumbra_genesis::AppState>> {
        let content = Self::make_genesis_content(
            &self.chain_id,
            self.allocations()?,
            self.genesis_validators
                .iter()
                .map(|v| v.try_into().expect("failed to convert PenumbraValidator"))
                .collect(),
            &self.net_config,
        )?;
        Self::make_genesis(content)
    }

    /// Prepare a set of initial [Allocation]s present at genesis. Optionally reads allocation
    /// files a CSV file, otherwise falls back to the historical requests of the testnet faucet
    /// in the Penumbra Discord channel.
    fn allocations(&self) -> anyhow::Result<Vec<Allocation>> {
        let mut allos: Vec<Allocation> = match &self.net_config.allocations_input_file {
            Some(a) => GenesisAllocation::from_csv(a.to_path_buf())
                .context(format!("could not parse allocations file {a:?}"))?,
            None => {
                // Default to latest testnet allocations computed in the build script.
                static LATEST_ALLOCATIONS: &str =
                    include_str!(env!("PD_LATEST_TESTNET_ALLOCATIONS"));
                GenesisAllocation::from_reader(std::io::Cursor::new(LATEST_ALLOCATIONS))
                    .with_context(|| {
                        format!(
                            "could not parse default latest testnet allocations file {:?}",
                            env!("PD_LATEST_TESTNET_ALLOCATIONS")
                        )
                    })?
            }
        };
        for v in &self.genesis_validators {
            allos.push(v.delegation_allocation()?);
        }
        Ok(allos)
    }

    /// Create a full genesis configuration for inclusion in the tendermint
    /// genesis config.
    pub fn make_genesis_content(
        chain_id: &str,
        allocations: Vec<Allocation>,
        validators: Vec<Validator>,
        net_config: &PenumbraNetworkConfig,
    ) -> anyhow::Result<penumbra_genesis::Content> {
        let gov_params = penumbra_governance::params::GovernanceParameters {
            proposal_voting_blocks: net_config.proposal_voting_blocks,
            ..Default::default()
        };

        let app_state = penumbra_genesis::Content {
            chain_id: chain_id.to_string(),
            stake_content: StakeContent {
                validators: validators.into_iter().map(Into::into).collect(),
                stake_params: StakeParameters {
                    active_validator_limit: net_config.active_validator_limit,
                    unbonding_delay: net_config.unbonding_delay,
                    ..Default::default()
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
                    epoch_duration: net_config.epoch_duration,
                },
            },
            ..Default::default()
        };
        Ok(app_state)
    }

    /// Build Tendermint genesis data, based on Penumbra initial application state.
    pub fn make_genesis(
        app_state: penumbra_genesis::Content,
    ) -> anyhow::Result<CometBFTGenesis<penumbra_genesis::AppState>> {
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
        let genesis = CometBFTGenesis {
            genesis_time,
            chain_id: app_state
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
            app_state: penumbra_genesis::AppState::Content(app_state),
            // Set empty validator set for Tendermint config, which falls back to reading
            // validators from the AppState, via ResponseInitChain:
            // https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
            validators: vec![],
        };
        Ok(genesis)
    }

    pub fn make_checkpoint(
        genesis: CometBFTGenesis<penumbra_genesis::AppState>,
        checkpoint: Option<Vec<u8>>,
    ) -> CometBFTGenesis<penumbra_genesis::AppState> {
        match checkpoint {
            Some(checkpoint) => CometBFTGenesis {
                app_state: penumbra_genesis::AppState::Checkpoint(checkpoint),
                ..genesis
            },
            None => genesis,
        }
    }

    /// Generate and write to disk the CometBFT config and key files for each validator at genesis.
    pub fn write_configs(&self) -> anyhow::Result<()> {
        // Loop over each validator and write its config separately.
        for (n, v) in self.genesis_validators.iter().enumerate() {
            // Create the directory for this node
            let node_name = format!("node{n}");
            let node_dir = self.testnet_dir.clone().join(node_name.clone());

            // Each node should include only the IPs for *other* nodes in their peers list.
            let mine = v.peering_address()?;
            let ips_minus_mine: Vec<TendermintAddress> = self
                .genesis_validators
                .iter()
                .filter(|v1| v1.peering_address().unwrap() != mine)
                .map(|v2| v2.peering_address().unwrap())
                .collect();
            tracing::debug!(?ips_minus_mine, "Found these peer ips");

            // TODO: figure out how to pass in custom cometbft config
            // cmt_config.0.consensus.timeout_commit = self.net_config.tendermint_timeout_commit;
            //
            v.fullnode
                .write_config(node_dir, &self.genesis()?, Some(ips_minus_mine))?;
        }
        Ok(())
    }
}

/// Represents initial allocations at genesis.
#[derive(Debug, Deserialize)]
pub struct GenesisAllocation {
    // Use custom deserializer to support reading integers with underscores as separators,
    // e.g. `1_000_000` -> `1000000`.
    #[serde(deserialize_with = "string_u64")]
    pub amount: u64,
    pub denom: String,
    pub address: String,
}

impl GenesisAllocation {
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
            let record: GenesisAllocation = result?;
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
}

/// Represents a funding stream for a [PenumbraValidator].
// Wraps [penumbra_stake::FundingStream::ToAddress] for compatibility's sake.
// Would be nice to ditch this intermediate struct and use the stake crate's logic.
#[derive(Debug, Deserialize, Clone)]
pub struct FundingStream {
    pub rate_bps: u16,
    pub address: String,
}

impl TryFrom<GenesisAllocation> for shielded_pool_genesis::Allocation {
    type Error = anyhow::Error;

    fn try_from(a: GenesisAllocation) -> anyhow::Result<shielded_pool_genesis::Allocation> {
        Ok(shielded_pool_genesis::Allocation {
            raw_amount: a.amount.into(),
            raw_denom: a.denom.clone(),
            address: Address::from_str(&a.address)
                .context("invalid address format in genesis allocations")?,
        })
    }
}

/// Custom deserializer, to support reading allocation amounts from CSV
/// that are formatted with underscores for readability, e.g.
/// `1_000_000` should be parsed as `1000000`.
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
"100000","udelegation_penumbravalid1jzcc6vsm29am9ggs8z0d7s9jk9uf8tfrqg7hglc9ufs7r23nu5yqtw77ex","penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk"
"100000","upenumbra","penumbra1rqcd3hfvkvc04c4c9vc0ac87lh4y0z8l28k4xp6d0cnd5jc6f6k0neuzp6zdwtpwyfpswtdzv9jzqtpjn5t6wh96pfx3flq2dhqgc42u7c06kj57dl39w2xm6tg0wh4zc8kjjk"
"100000","udelegation_penumbravalid1p2hfuch2p8rshyc90qa23gqk82s74tqcu3x2x3y5tfwpzth4vvrq2gv283","penumbra1xq2e9x7uhfzezwunvazdamlxepf4jr5htsuqnzlsahuayyqxjjwg9lk0aytwm6wfj3jy29rv2kdpen57903s8wxv3jmqwj6m6v5jgn6y2cypfd03rke652k8wmavxra7e9wkrg"
"100000","upenumbra","penumbra1xq2e9x7uhfzezwunvazdamlxepf4jr5htsuqnzlsahuayyqxjjwg9lk0aytwm6wfj3jy29rv2kdpen57903s8wxv3jmqwj6m6v5jgn6y2cypfd03rke652k8wmavxra7e9wkrg"
"100000","udelegation_penumbravalid182k8x46hg5vx3ez8ec58ze5yd6a3q4q3fkx45ddt5jahnzz0xyyqdtz7hc","penumbra100zd92fg6x27wc0mlu48cd6phq420u7ep59kzdalg2cq66mjkyl0xr54z0c64gectnj44mv5k2vyjjsz5gyd5gq33a6wnqzvgu2fz7namz7usazsl6p8wza83gcpwt8q76rc4y"
"100000","upenumbra","penumbra100zd92fg6x27wc0mlu48cd6phq420u7ep59kzdalg2cq66mjkyl0xr54z0c64gectnj44mv5k2vyjjsz5gyd5gq33a6wnqzvgu2fz7namz7usazsl6p8wza83gcpwt8q76rc4y"
"100000","udelegation_penumbravalid1t2hr2lj5n2jt3hftzjw3strjllnakc7jtw234d229x3zakhaqsqsg9yarw","penumbra1xap8sgefy9rl2nfvsse0h4y6c25hy2n20ymr5w7hs28m9xemt3tmz88atyulswumc32sv7h937wnfhyct282de66zm75nk6ywq3d4r32p5ju0cnscj2rraesnrxr9lvk6hcazp"
"100000","upenumbra","penumbra1xap8sgefy9rl2nfvsse0h4y6c25hy2n20ymr5w7hs28m9xemt3tmz88atyulswumc32sv7h937wnfhyct282de66zm75nk6ywq3d4r32p5ju0cnscj2rraesnrxr9lvk6hcazp"
"#;
        let allos = GenesisAllocation::from_reader(csv_content.as_bytes())?;

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
        let result = GenesisAllocation::from_reader(csv_content.as_bytes());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    /// Generate a config suitable for local testing: no custom address information, no additional
    /// validators at genesis.
    fn generate_devnet_config() -> anyhow::Result<()> {
        let testnet_config = PenumbraNetwork {
            chain_id: String::from("test-chain-1234"),
            ..Default::default()
        };
        assert_eq!(testnet_config.chain_id, "test-chain-1234");
        // The Genesis struct will have empty validators; validators will be read from AppState.
        assert_eq!(testnet_config.genesis()?.validators.len(), 0);
        // No external address template was given, so only 1 validator will be present.
        let penumbra_genesis::AppState::Content(app_state) = testnet_config.genesis()?.app_state
        else {
            unimplemented!("TODO: support checkpointed app state")
        };
        assert_eq!(app_state.stake_content.validators.len(), 1);
        Ok(())
    }

    #[test]
    /// Generate a config suitable for a public testnet: custom validators input file,
    /// increasing the default validators from 1 -> 2.
    fn generate_testnet_config() -> anyhow::Result<()> {
        let ci_validators_filepath = PathBuf::from("../../../testnets/validators-ci.json");
        let testnet_config = PenumbraNetwork::new(
            Some(String::from("test-chain-4567")),
            PenumbraNetworkConfig {
                validators_input_file: Some(ci_validators_filepath),
                peer_address_template: Some(String::from("validator.local")),
                ..Default::default()
            },
        )?;
        assert_eq!(testnet_config.chain_id, "test-chain-4567");
        assert_eq!(testnet_config.genesis()?.validators.len(), 0);
        let penumbra_genesis::AppState::Content(app_state) = testnet_config.genesis()?.app_state
        else {
            unimplemented!("TODO: support checkpointed app state")
        };
        // These values should be the same, and the number of validators should be 2.
        assert_eq!(testnet_config.genesis_validators.len(), 2);
        assert_eq!(app_state.stake_content.validators.len(), 2);
        Ok(())
    }

    #[test]
    fn testnet_validator_to_validator_conversion() -> anyhow::Result<()> {
        let v = PenumbraValidator::default();
        let stake_validator: Validator = v.try_into()?;
        assert!(stake_validator.website == "");
        Ok(())
    }
}
