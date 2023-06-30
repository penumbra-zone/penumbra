use ark_ff::Zero;
use serde::{Deserialize, Serialize};

use penumbra_crypto::{asset::Amount, Balance, Fr, Value};
use penumbra_governance::ProposalNft;
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType, TypeUrl};

use crate::{ActionView, IsAction, TransactionPerspective};

/// A withdrawal of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalWithdraw", into = "pb::ProposalWithdraw")]
pub struct ProposalWithdraw {
    /// The proposal ID to withdraw.
    pub proposal: u64,
    // The reason the proposal was withdrawn.
    pub reason: String,
}

impl IsAction for ProposalWithdraw {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ProposalWithdraw(self.to_owned())
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
    /// Compute a commitment to the value contributed to a transaction by this proposal submission.
    pub fn balance(&self) -> Balance {
        let voting_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::deposit(self.proposal).denom().into(),
        };
        let withdrawn_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::unbonding_deposit(self.proposal).denom().into(),
        };

        // Proposal withdrawals consume the submitted proposal and produce a withdrawn proposal:
        Balance::from(withdrawn_proposal_nft) - Balance::from(voting_proposal_nft)
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

impl TypeUrl for ProposalWithdraw {
    const TYPE_URL: &'static str = "/penumbra.core.governance.v1alpha1.ProposalWithdraw";
}

impl DomainType for ProposalWithdraw {
    type Proto = pb::ProposalWithdraw;
}
