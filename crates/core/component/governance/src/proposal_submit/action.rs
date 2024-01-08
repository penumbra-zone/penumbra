use serde::{Deserialize, Serialize};

use penumbra_asset::{Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};
use penumbra_txhash::{EffectHash, EffectingData};

use crate::proposal::Proposal;

use crate::ProposalNft;

/// A proposal submission describes the proposal to propose, and the (transparent, ephemeral) refund
/// address for the proposal deposit, along with a key to be used to verify the signature for a
/// withdrawal of that proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalSubmit", into = "pb::ProposalSubmit")]
pub struct ProposalSubmit {
    /// The proposal to propose.
    pub proposal: Proposal,
    /// The amount deposited for the proposal.
    pub deposit_amount: Amount,
}

impl EffectingData for ProposalSubmit {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl ProposalSubmit {
    /// Compute a commitment to the value contributed to a transaction by this proposal submission.
    pub fn balance(&self) -> Balance {
        let deposit = Value {
            amount: self.deposit_amount,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        let proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::deposit(self.proposal.id).denom().into(),
        };

        // Proposal submissions *require* the deposit amount in order to be accepted, so they
        // contribute (-deposit) to the value balance of the transaction, and they contribute a
        // single proposal NFT to the value balance:
        Balance::from(proposal_nft) - Balance::from(deposit)
    }
}

impl From<ProposalSubmit> for pb::ProposalSubmit {
    fn from(value: ProposalSubmit) -> pb::ProposalSubmit {
        pb::ProposalSubmit {
            proposal: Some(value.proposal.into()),
            deposit_amount: Some(value.deposit_amount.into()),
        }
    }
}

impl TryFrom<pb::ProposalSubmit> for ProposalSubmit {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalSubmit) -> Result<Self, Self::Error> {
        Ok(ProposalSubmit {
            proposal: msg
                .proposal
                .ok_or_else(|| anyhow::anyhow!("missing proposal in `Propose`"))?
                .try_into()?,
            deposit_amount: msg
                .deposit_amount
                .ok_or_else(|| anyhow::anyhow!("missing deposit amount in `Propose`"))?
                .try_into()?,
        })
    }
}

impl DomainType for ProposalSubmit {
    type Proto = pb::ProposalSubmit;
}
