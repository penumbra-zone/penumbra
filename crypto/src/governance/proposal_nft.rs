use std::str::FromStr;

use regex::Regex;

use crate::asset;

/// Unbonding tokens represent staking tokens that are currently unbonding and
/// subject to slashing.
///
/// Unbonding tokens are parameterized by the validator identity, the epoch at
/// which unbonding began, and the epoch at which unbonding ends.
pub struct ProposalNft {
    proposal_id: u64,
    proposal_state: State,
    base_denom: asset::Denom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Voting,
    Withdrawn,
    Slashed,
    Failed,
    Passed,
}

impl State {
    pub const fn display_static(&self) -> &'static str {
        match self {
            State::Voting => "voting",
            State::Withdrawn => "withdrawn",
            State::Slashed => "slashed",
            State::Failed => "failed",
            State::Passed => "passed",
        }
    }
}

impl FromStr for State {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "voting" => Ok(State::Voting),
            "withdrawn" => Ok(State::Withdrawn),
            "slashed" => Ok(State::Slashed),
            "failed" => Ok(State::Failed),
            "passed" => Ok(State::Passed),
            _ => Err(anyhow::anyhow!("invalid proposal token state")),
        }
    }
}

impl ProposalNft {
    fn new(proposal_id: u64, proposal_state: State) -> Self {
        // This format string needs to be in sync with the asset registry
        let base_denom = asset::REGISTRY
            .parse_denom(&format!(
                "proposal_{}_{}",
                proposal_id,
                proposal_state.display_static()
            ))
            .expect("base denom format is valid");
        ProposalNft {
            proposal_id,
            proposal_state,
            base_denom,
        }
    }

    /// Make a new proposal NFT in the voting state.
    pub fn voting(proposal_id: u64) -> Self {
        Self::new(proposal_id, State::Voting)
    }

    /// Make a new proposal NFT in the withdrawn state.
    pub fn withdrawn(proposal_id: u64) -> Self {
        Self::new(proposal_id, State::Withdrawn)
    }

    /// Make a new proposal NFT in the slashed state.
    pub fn slashed(proposal_id: u64) -> Self {
        Self::new(proposal_id, State::Slashed)
    }

    /// Make a new proposal NFT in the failed state.
    pub fn failed(proposal_id: u64) -> Self {
        Self::new(proposal_id, State::Failed)
    }

    /// Make a new proposal NFT in the passed state.
    pub fn passed(proposal_id: u64) -> Self {
        Self::new(proposal_id, State::Passed)
    }

    /// Get the base denomination for this delegation token.
    pub fn denom(&self) -> asset::Denom {
        self.base_denom.clone()
    }

    /// Get the default display denomination for this delegation token.
    pub fn default_unit(&self) -> asset::Unit {
        self.base_denom.default_unit()
    }

    /// Get the asset ID for this delegation token.
    pub fn id(&self) -> asset::Id {
        self.base_denom.id()
    }

    /// Get the proposal ID for this proposal token.
    pub fn proposal_id(&self) -> u64 {
        self.proposal_id
    }

    /// Get the proposal state for this proposal token.
    pub fn proposal_state(&self) -> State {
        self.proposal_state
    }
}

impl TryFrom<asset::Denom> for ProposalNft {
    type Error = anyhow::Error;

    fn try_from(base_denom: asset::Denom) -> Result<Self, Self::Error> {
        let base_string = base_denom.to_string();

        // Note: this regex must be in sync with asset::REGISTRY
        // The data capture group is used by asset::REGISTRY
        let captures = Regex::new("^proposal_(?P<data>(?P<proposal_id>[0-9]+)_(?P<proposal_state>voting|withdrawn|passed|failed|slashed))$")
            .expect("regex is valid")
            .captures(base_string.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "base denom {} is not a proposal token",
                    base_denom.to_string()
                )
            })?;

        let proposal_id = captures
            .name("proposal_id")
            .expect("proposal_id is a named capture")
            .as_str()
            .parse()?;

        let proposal_state = captures
            .name("proposal_state")
            .expect("proposal_state is a named capture")
            .as_str()
            .parse()?;

        Ok(Self {
            base_denom,
            proposal_state,
            proposal_id,
        })
    }
}

impl FromStr for ProposalNft {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        asset::REGISTRY
            .parse_denom(s)
            .ok_or_else(|| anyhow::anyhow!("could not parse {} as base denomination", s))?
            .try_into()
    }
}

impl std::fmt::Display for ProposalNft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl std::fmt::Debug for ProposalNft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl PartialEq for ProposalNft {
    fn eq(&self, other: &Self) -> bool {
        self.base_denom.eq(&other.base_denom)
    }
}

impl Eq for ProposalNft {}

impl std::hash::Hash for ProposalNft {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base_denom.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_token_denomination_round_trip() {
        let token = ProposalNft::new(1, State::Voting);

        let denom = token.to_string();
        println!("denom: {denom}");
        let token2 = ProposalNft::from_str(&denom).unwrap();
        let denom2 = token2.to_string();

        assert_eq!(denom, denom2);
        assert_eq!(token, token2);
    }
}
