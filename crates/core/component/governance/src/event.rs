use penumbra_proto::penumbra::core::component::governance::v1 as pb;

use crate::{
    DelegatorVote, Proposal, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote,
};

pub fn delegator_vote(delegator_vote: &DelegatorVote) -> pb::EventDelegatorVote {
    pb::EventDelegatorVote {
        vote: Some(pb::DelegatorVote::from(*delegator_vote)),
    }
}

pub fn proposal_deposit_claim(
    deposit_claim: &ProposalDepositClaim,
) -> pb::EventProposalDepositClaim {
    pb::EventProposalDepositClaim {
        deposit_claim: Some(pb::ProposalDepositClaim::from(*deposit_claim)),
    }
}

pub fn validator_vote(validator_vote: &ValidatorVote) -> pb::EventValidatorVote {
    pb::EventValidatorVote {
        vote: Some(pb::ValidatorVote::from(validator_vote.clone())),
    }
}

pub fn proposal_withdraw(withdraw: &ProposalWithdraw) -> pb::EventProposalWithdraw {
    pb::EventProposalWithdraw {
        withdraw: Some(pb::ProposalWithdraw::from(withdraw.clone())),
    }
}

pub fn proposal_submit(submit: &ProposalSubmit) -> pb::EventProposalSubmit {
    pb::EventProposalSubmit {
        submit: Some(pb::ProposalSubmit::from(submit.clone())),
    }
}

pub fn enact_proposal(proposal: &Proposal) -> pb::EventEnactProposal {
    pb::EventEnactProposal {
        proposal: Some(pb::Proposal::from(proposal.clone())),
    }
}

pub fn proposal_failed(proposal: &Proposal) -> pb::EventProposalFailed {
    pb::EventProposalFailed {
        proposal: Some(pb::Proposal::from(proposal.clone())),
    }
}

pub fn proposal_slashed(proposal: &Proposal) -> pb::EventProposalSlashed {
    pb::EventProposalSlashed {
        proposal: Some(pb::Proposal::from(proposal.clone())),
    }
}
