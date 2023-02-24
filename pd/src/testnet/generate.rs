//! Logic for creating a new testnet configuration.
//! Used for deploying (approximately weekly) testnets
//! for Penumbra.
use crate::testnet::{generate_tm_config, parse_tm_address, write_configs, ValidatorKeys};
use anyhow::{Context, Result};
use penumbra_chain::genesis;
use penumbra_chain::{genesis::Allocation, params::ChainParameters};
use penumbra_component::stake::{validator::Validator, FundingStream, FundingStreams};
use penumbra_crypto::{
    keys::SpendKey,
    stake::{DelegationToken, IdentityKey},
    Address, GovernanceKey,
};
use serde::{de, Deserialize};
use std::{
    fmt,
    fs::File,
    io::Read,
    net::Ipv4Addr,
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tendermint::{node, public_key::Algorithm, Genesis, Time};

/// Create a new testnet definition, including genesis and at least one
/// validator config. Write all configs to the target testnet dir,
/// defaulting to `~/.penumbra/testnet_data`, as usual.
#[allow(clippy::too_many_arguments)]
pub fn testnet_generate(
    testnet_dir: PathBuf,
    chain_id: &str,
    active_validator_limit: Option<u64>,
    epoch_duration: Option<u64>,
    unbonding_epochs: Option<u64>,
    starting_ip: Ipv4Addr,
    validators_input_file: Option<PathBuf>,
    allocations_input_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    let genesis_time = Time::from_unix_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time travels linearly in a forward direction")
            .as_secs() as i64,
        0,
    )
    .expect("able to convert current time into Time");

    // Parse allocations from input file or default to latest testnet allocations computed
    // in the build script
    let mut allocations = if let Some(allocations_input_file) = allocations_input_file {
        let allocations_file = File::open(&allocations_input_file)
            .with_context(|| format!("cannot open file {allocations_input_file:?}"))?;
        parse_allocations(allocations_file).with_context(|| {
            format!("could not parse allocations file {allocations_input_file:?}")
        })?
    } else {
        static LATEST_ALLOCATIONS: &str = include_str!(env!("PD_LATEST_TESTNET_ALLOCATIONS"));
        parse_allocations(std::io::Cursor::new(LATEST_ALLOCATIONS)).with_context(|| {
            format!(
                "could not parse default latest testnet allocations file {:?}",
                env!("PD_LATEST_TESTNET_ALLOCATIONS")
            )
        })?
    };

    // Parse validators from input file or default to latest testnet validators computed in
    // the build script
    let testnet_validators = if let Some(validators_input_file) = validators_input_file {
        let validators_file = File::open(&validators_input_file)
            .with_context(|| format!("cannot open file {validators_input_file:?}"))?;
        parse_validators(validators_file)
            .with_context(|| format!("could not parse validators file {validators_input_file:?}"))?
    } else {
        static LATEST_VALIDATORS: &str = include_str!(env!("PD_LATEST_TESTNET_VALIDATORS"));
        parse_validators(std::io::Cursor::new(LATEST_VALIDATORS)).with_context(|| {
            format!(
                "could not parse default latest testnet validators file {:?}",
                env!("PD_LATEST_TESTNET_VALIDATORS")
            )
        })?
    };

    let mut validator_keys = Vec::<ValidatorKeys>::new();
    // Generate a keypair for each validator
    let num_validator_nodes = testnet_validators.len();
    assert!(
        num_validator_nodes > 0,
        "must have at least one validator node"
    );
    for _ in 0..num_validator_nodes {
        let vk = ValidatorKeys::generate();

        let spend_key = SpendKey::from(vk.validator_spend_key.clone());
        let fvk = spend_key.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        // Add a default 1 upenumbra allocation to the validator.
        let identity_key: IdentityKey = IdentityKey(fvk.spend_verification_key().clone());
        let delegation_denom = DelegationToken::from(&identity_key).denom();
        allocations.push(Allocation {
            address: dest,
            // Add an initial allocation of 50,000 delegation tokens,
            // starting them with 50x the individual allocations to discord users.
            // 50,000 delegation tokens * 1e6 udelegation factor
            amount: (50_000 * 10u64.pow(6)),
            denom: delegation_denom.to_string(),
        });

        validator_keys.push(vk);
    }

    let ip_addrs = validator_keys
        .iter()
        .enumerate()
        .map(|(i, _vk)| {
            let a = starting_ip.octets();
            Ipv4Addr::new(a[0], a[1], a[2], a[3] + (10 * i as u8))
        })
        .collect::<Vec<_>>();

    let validators = testnet_validators
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let vk = &validator_keys[i];
            Ok(Validator {
                // Currently there's no way to set validator keys beyond
                // manually editing the genesis.json. Otherwise they
                // will be randomly generated keys.
                identity_key: IdentityKey(vk.validator_id_vk),
                governance_key: GovernanceKey(vk.validator_id_vk),
                consensus_key: vk.validator_cons_pk,
                name: v.name.clone(),
                website: v.website.clone(),
                description: v.description.clone(),
                enabled: true,
                funding_streams: FundingStreams::try_from(
                    v.funding_streams
                        .iter()
                        .map(|fs| {
                            Ok(FundingStream {
                                address: Address::from_str(&fs.address)
                                    .context("invalid funding stream address in validators.json")?,
                                rate_bps: fs.rate_bps,
                            })
                        })
                        .collect::<Result<Vec<FundingStream>, anyhow::Error>>()?,
                )
                .context("unable to construct funding streams from validators.json")?,
                sequence_number: v.sequence_number,
            })
        })
        .collect::<Result<Vec<Validator>, anyhow::Error>>()?;

    let default_params = ChainParameters::default();
    let active_validator_limit =
        active_validator_limit.unwrap_or(default_params.active_validator_limit);
    let epoch_duration = epoch_duration.unwrap_or(default_params.epoch_duration);
    let unbonding_epochs = unbonding_epochs.unwrap_or(default_params.unbonding_epochs);

    let app_state = genesis::AppState {
        allocations: allocations.clone(),
        chain_params: ChainParameters {
            chain_id: chain_id.to_string(),
            epoch_duration,
            unbonding_epochs,
            active_validator_limit,
            ..Default::default()
        },
        validators: validators.into_iter().map(Into::into).collect(),
    };

    // Create the genesis data shared by all nodes
    let validator_genesis = Genesis {
        genesis_time,
        chain_id: chain_id
            .parse::<tendermint::chain::Id>()
            .expect("able to create chain ID"),
        initial_height: 0,
        consensus_params: tendermint::consensus::Params {
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
            version: Some(tendermint::consensus::params::VersionParams { app_version: 0 }),
        },
        // always empty in genesis json
        app_hash: tendermint::AppHash::default(),
        app_state,
        // List of initial validators. Note this may be overridden entirely by
        // the application, and may be left empty to make explicit that the
        // application will initialize the validator set with ResponseInitChain.
        // - https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
        // For penumbra, we can leave this empty since the app_state also contains Validator
        // configs.
        validators: vec![],
    };

    for (n, vk) in validator_keys.iter().enumerate() {
        let node_name = format!("node{n}");

        // Create the directory for this node
        let mut node_dir = testnet_dir.clone();
        node_dir.push(node_name.clone());

        // Write this node's config.toml
        // Note that this isn't a re-implementation of the `Config` type from
        // Tendermint (https://github.com/tendermint/tendermint/blob/6291d22f46f4c4f9121375af700dbdafa51577e7/config/config.go#L92)
        // so if they change their defaults or the available fields, that won't be reflected in our template.
        // TODO: grab all peer pubkeys instead of self pubkey
        let my_ip = &ip_addrs[n];
        // Each node should include only the IPs for *other* nodes in their peers list.
        let ips_minus_mine = ip_addrs
            .iter()
            .enumerate()
            .filter(|(_, p)| *p != my_ip)
            .map(|(n, ip)| {
                (
                    node::Id::from(validator_keys[n].node_key_pk.ed25519().unwrap()),
                    format!("{ip}:26656"),
                )
            })
            .filter_map(|(id, ip)| parse_tm_address(Some(&id), &ip).ok())
            .collect::<Vec<_>>();
        let tm_config = generate_tm_config(&node_name, ips_minus_mine, None)?;

        write_configs(node_dir, vk, &validator_genesis, tm_config)?;
    }
    Ok(())
}

fn parse_allocations(input: impl Read) -> Result<Vec<genesis::Allocation>> {
    let mut rdr = csv::Reader::from_reader(input);
    let mut res = vec![];
    for (line, result) in rdr.deserialize().enumerate() {
        let record: TestnetAllocation = result?;
        let record: genesis::Allocation = record
            .try_into()
            .with_context(|| format!("invalid address in entry {line} of allocations file"))?;
        res.push(record);
    }

    Ok(res)
}

fn parse_validators(input: impl Read) -> Result<Vec<TestnetValidator>> {
    Ok(serde_json::from_reader(input)?)
}

/// Represents initial allocations to the testnet.
#[derive(Debug, Deserialize)]
pub struct TestnetAllocation {
    #[serde(deserialize_with = "string_u64")]
    pub amount: u64,
    pub denom: String,
    pub address: String,
}

/// Represents a funding stream within a testnet configuration file.
#[derive(Debug, Deserialize)]
pub struct TestnetFundingStream {
    pub rate_bps: u16,
    pub address: String,
}

/// Represents testnet validators in configuration files.
#[derive(Debug, Deserialize)]
pub struct TestnetValidator {
    pub name: String,
    pub website: String,
    pub description: String,
    pub funding_streams: Vec<TestnetFundingStream>,
    pub sequence_number: u32,
}

impl TryFrom<TestnetAllocation> for genesis::Allocation {
    type Error = anyhow::Error;

    fn try_from(a: TestnetAllocation) -> anyhow::Result<genesis::Allocation> {
        Ok(genesis::Allocation {
            amount: a.amount,
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
