use serde::{Deserialize, Serialize};

use penumbra_asset::{
    asset::{self, DenomMetadata},
    Balance, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};
use penumbra_txhash::{EffectHash, EffectingData};

use crate::proposal_state::{Outcome, Withdrawn};

use crate::ProposalNft;

/// A claim for the initial submission deposit for a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ProposalDepositClaim",
    into = "pb::ProposalDepositClaim"
)]
pub struct ProposalDepositClaim {
    /// The proposal ID to claim the deposit for.
    pub proposal: u64,
    /// The amount of the deposit.
    pub deposit_amount: Amount,
    /// The outcome of the proposal.
    pub outcome: Outcome<()>,
}

impl EffectingData for ProposalDepositClaim {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for ProposalDepositClaim {
    type Proto = pb::ProposalDepositClaim;
}

impl From<ProposalDepositClaim> for pb::ProposalDepositClaim {
    fn from(value: ProposalDepositClaim) -> pb::ProposalDepositClaim {
        pb::ProposalDepositClaim {
            proposal: value.proposal,
            deposit_amount: Some(value.deposit_amount.into()),
            outcome: Some(value.outcome.into()),
        }
    }
}

impl TryFrom<pb::ProposalDepositClaim> for ProposalDepositClaim {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalDepositClaim) -> Result<Self, Self::Error> {
        Ok(ProposalDepositClaim {
            proposal: msg.proposal,
            deposit_amount: msg
                .deposit_amount
                .ok_or_else(|| anyhow::anyhow!("missing deposit amount in `ProposalDepositClaim`"))?
                .try_into()?,
            outcome: msg
                .outcome
                .ok_or_else(|| anyhow::anyhow!("missing outcome in `ProposalDepositClaim`"))?
                .try_into()?,
        })
    }
}

impl ProposalDepositClaim {
    /// Compute the balance contributed to the transaction by this proposal deposit claim.
    pub fn balance(&self) -> Balance {
        let deposit = Value {
            amount: self.deposit_amount,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        let (voting_or_withdrawn_proposal_denom, claimed_proposal_denom): (
            DenomMetadata,
            DenomMetadata,
        ) = match self.outcome {
            // Outcomes without withdrawal consume `deposit`:
            Outcome::Passed => (
                ProposalNft::deposit(self.proposal).denom(),
                ProposalNft::passed(self.proposal).denom(),
            ),
            Outcome::Failed {
                withdrawn: Withdrawn::No,
            } => (
                ProposalNft::deposit(self.proposal).denom(),
                ProposalNft::failed(self.proposal).denom(),
            ),
            Outcome::Slashed {
                withdrawn: Withdrawn::No,
            } => (
                ProposalNft::deposit(self.proposal).denom(),
                ProposalNft::slashed(self.proposal).denom(),
            ),
            // Outcomes after withdrawal consume `unbonding_deposit`:
            Outcome::Failed {
                withdrawn: Withdrawn::WithReason { .. },
            } => (
                ProposalNft::unbonding_deposit(self.proposal).denom(),
                ProposalNft::failed(self.proposal).denom(),
            ),
            Outcome::Slashed {
                withdrawn: Withdrawn::WithReason { .. },
            } => (
                ProposalNft::unbonding_deposit(self.proposal).denom(),
                ProposalNft::slashed(self.proposal).denom(),
            ),
        };

        // NFT to be consumed
        let voting_or_withdrawn_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: asset::Id::from(voting_or_withdrawn_proposal_denom),
        };

        // NFT to be created
        let claimed_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: asset::Id::from(claimed_proposal_denom),
        };

        // Proposal deposit claims consume the submitted or withdrawn proposal and produce a claimed
        // proposal and the deposit:
        let mut balance =
            Balance::from(claimed_proposal_nft) - Balance::from(voting_or_withdrawn_proposal_nft);

        // Only issue a refund if the proposal was not slashed
        if self.outcome.should_be_refunded() {
            balance += Balance::from(deposit);
        }

        balance
    }
}
