use std::str::FromStr;

use regex::Regex;

use crate::asset;

/// Unbonding tokens represent staking tokens that are currently unbonding and
/// subject to slashing.
///
/// Unbonding tokens are parameterized by the validator identity, the epoch at
/// which unbonding began, and the epoch at which unbonding ends.
pub struct VotingReceiptToken {
    proposal_id: u64,
    base_denom: asset::Denom,
}

impl VotingReceiptToken {
    pub fn new(proposal_id: u64) -> Self {
        // This format string needs to be in sync with the asset registry
        let base_denom = asset::REGISTRY
            .parse_denom(&format!("uvoted_on_{proposal_id}"))
            .expect("base denom format is valid");
        VotingReceiptToken {
            proposal_id,
            base_denom,
        }
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

    /// Get the proposal ID this delegation token is associated with.
    pub fn proposal_id(&self) -> u64 {
        self.proposal_id
    }
}

impl TryFrom<asset::Denom> for VotingReceiptToken {
    type Error = anyhow::Error;

    fn try_from(base_denom: asset::Denom) -> Result<Self, Self::Error> {
        let base_string = base_denom.to_string();

        // Note: this regex must be in sync with both asset::REGISTRY
        // and VALIDATOR_IDENTITY_BECH32_PREFIX
        // The data capture group is used by asset::REGISTRY
        let captures = Regex::new("^uvoted_on_(?P<data>(?P<proposal_id>[0-9]+))$")
            .expect("regex is valid")
            .captures(base_string.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "base denom {} is not an unbonding token",
                    base_denom.to_string()
                )
            })?;

        let proposal_id: u64 = captures
            .name("proposal_id")
            .expect("proposal_id is a named capture")
            .as_str()
            .parse()?;

        Ok(Self {
            proposal_id,
            base_denom,
        })
    }
}

impl FromStr for VotingReceiptToken {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        asset::REGISTRY
            .parse_denom(s)
            .ok_or_else(|| anyhow::anyhow!("could not parse {} as base denomination", s))?
            .try_into()
    }
}

impl std::fmt::Display for VotingReceiptToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl std::fmt::Debug for VotingReceiptToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl PartialEq for VotingReceiptToken {
    fn eq(&self, other: &Self) -> bool {
        self.base_denom.eq(&other.base_denom)
    }
}

impl Eq for VotingReceiptToken {}

impl std::hash::Hash for VotingReceiptToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base_denom.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbonding_token_denomination_round_trip() {
        let proposal_id: u64 = 1;

        let token = VotingReceiptToken::new(proposal_id);

        let denom = token.to_string();
        println!("denom: {denom}");
        let token2 = VotingReceiptToken::from_str(&denom).unwrap();
        let denom2 = token2.to_string();

        assert_eq!(denom, denom2);
        assert_eq!(token, token2);
    }
}
