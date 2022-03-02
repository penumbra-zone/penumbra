use std::{env::current_dir, fmt, fs::File, path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use directories::UserDirs;
use penumbra_crypto::Address;
use regex::{Captures, Regex};
use serde::{de, Deserialize};
use tendermint::PrivateKey;

use crate::genesis;

/// Methods and types used for generating testnet configurations.

pub fn parse_allocations_file(input_file: PathBuf) -> Result<Vec<genesis::Allocation>> {
    let file = File::open(&input_file)
        .with_context(|| format!("couldn't open allocations file {:?}", input_file))?;

    let mut rdr = csv::Reader::from_reader(file);
    let mut res = vec![];
    for (line, result) in rdr.deserialize().enumerate() {
        let record: TestnetAllocation = result?;
        let record: genesis::Allocation = record.try_into().with_context(|| {
            format!(
                "invalid address in entry {} of allocations file {:?}",
                line, input_file
            )
        })?;
        res.push(record);
    }

    Ok(res)
}

pub fn parse_validators_file(input_file: PathBuf) -> Result<Vec<TestnetValidator>> {
    let file = File::open(&input_file).context("couldn't open validators file")?;

    let validators: Vec<TestnetValidator> = serde_json::from_reader(file)?;

    Ok(validators)
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
            let r = v.replace("_", "");
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

/// Hardcoded Tendermint config template. Should produce tendermint config similar to
/// https://github.com/tendermint/tendermint/blob/6291d22f46f4c4f9121375af700dbdafa51577e7/cmd/tendermint/commands/init.go#L45
/// There exists https://github.com/informalsystems/tendermint-rs/blob/a12118978f2ffea4042d6d38ebfb290d12611314/config/src/config.rs#L23 but
/// this seemed more straightforward as only the moniker is changed right now.
pub fn generate_tm_config(node_name: &str) -> String {
    format!(
        include_str!("../../testnets/tm_config_template.toml"),
        node_name
    )
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
    pub voting_power: u32,
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

#[derive(Deserialize)]
pub struct TendermintNodeKey {
    pub id: String,
    pub priv_key: TendermintPrivKey,
}

#[derive(Deserialize)]
pub struct TendermintPrivKey {
    #[serde(rename(serialize = "type"))]
    pub key_type: String,
    pub value: PrivateKey,
}

// Easiest to hardcode since we never change these.
pub fn get_validator_state() -> String {
    r#"{
    "height": "0",
    "round": 0,
    "step": 0
}
"#
    .to_string()
}

/// Expand tildes in a path.
/// Modified from https://stackoverflow.com/a/68233480
pub fn canonicalize_path(input: &str) -> PathBuf {
    let tilde = Regex::new(r"^~(/|$)").unwrap();
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is
        input.into()
    } else if tilde.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory
        PathBuf::from(&*tilde.replace(input, |c: &Captures| {
            if let Some(user_dirs) = UserDirs::new() {
                format!("{}{}", user_dirs.home_dir().to_string_lossy(), &c[1],)
            } else {
                c[0].to_string()
            }
        }))
    } else {
        PathBuf::from(format!("{}/{}", current_dir().unwrap().display(), input))
    }
}
