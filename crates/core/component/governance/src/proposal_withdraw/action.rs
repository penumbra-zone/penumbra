use serde::{Deserialize, Serialize};

use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::governance::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};

use crate::ProposalNft;

/// A withdrawal of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalWithdraw", into = "pb::ProposalWithdraw")]
pub struct ProposalWithdraw {
    /// The proposal ID to withdraw.
    pub proposal: u64,
    // The reason the proposal was withdrawn.
    pub reason: String,
}

impl EffectingData for ProposalWithdraw {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl From<ProposalWithdraw> for pb::ProposalWithdraw {
    fn from(value: ProposalWithdraw) -> pb::ProposalWithdraw {
        pb::ProposalWithdraw {
            proposal: value.proposal,
            reason: value.reason,
        }
    }
}

impl ProposalWithdraw {
    /// Compute a commitment to the value contributed to a transaction by this proposal withdrawal.
    pub fn balance(&self) -> Balance {
        let voting_proposal_nft = self.voting_proposal_nft_value();
        let withdrawn_proposal_nft = self.withdrawn_proposal_nft();

        // Proposal withdrawals consume the submitted proposal and produce a withdrawn proposal:
        Balance::from(withdrawn_proposal_nft) - Balance::from(voting_proposal_nft)
    }

    /// Returns the [`Value`] of the proposal NFT.
    fn voting_proposal_nft_value(&self) -> Value {
        Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::deposit(self.proposal).denom().into(),
        }
    }

    /// Returns a withdrawal NFT.
    fn withdrawn_proposal_nft(&self) -> Value {
        Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::unbonding_deposit(self.proposal).denom().into(),
        }
    }
}

impl TryFrom<pb::ProposalWithdraw> for ProposalWithdraw {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalWithdraw) -> Result<Self, Self::Error> {
        Ok(ProposalWithdraw {
            proposal: msg.proposal,
            reason: msg.reason,
        })
    }
}

impl DomainType for ProposalWithdraw {
    type Proto = pb::ProposalWithdraw;
}
