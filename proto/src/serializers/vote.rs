use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serializer};

use crate::core::governance::v1alpha1::vote::Vote;

/// Deserialize hexstring into Vec<u8>
pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let vote = Vote::from_str(&string).map_err(serde::de::Error::custom)?;
    Ok(vote as i32)
}

/// Serialize from T into hexstring
pub fn serialize<S>(value: &i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(
        Vote::from_i32(*value)
            .ok_or_else(|| serde::ser::Error::custom("invalid vote"))?
            .to_string()
            .as_str(),
    )
}

impl FromStr for Vote {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Vote> {
        match s.replace(['-', '_', ' '], "").to_lowercase().as_str() {
            "yes" | "y" => Ok(Vote::Yes),
            "no" | "n" => Ok(Vote::No),
            "abstain" | "a" => Ok(Vote::Abstain),
            "veto" | "noveto" | "nowithveto" | "v" => Ok(Vote::NoWithVeto),
            _ => Err(anyhow::anyhow!("invalid vote: {}", s)),
        }
    }
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Vote::Yes => write!(f, "yes"),
            Vote::No => write!(f, "no"),
            Vote::Abstain => write!(f, "abstain"),
            Vote::NoWithVeto => write!(f, "no_with_veto"),
            // TODO(erwan): make sure this is correct, MERGEBLOCK
            _ => unreachable!(),
        }
    }
}
