use penumbra_sdk_proto::penumbra::core::component::governance::v1 as pb;
use penumbra_sdk_stake::IdentityKey;

use crate::{
    DelegatorVote, Proposal, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote,
};

pub fn delegator_vote(
    delegator_vote: &DelegatorVote,
    validator_identity_key: &IdentityKey,
) -> pb::EventDelegatorVote {
    pb::EventDelegatorVote {
        vote: Some(pb::DelegatorVote::from(*delegator_vote)),
        validator_identity_key: Some(validator_identity_key.clone().into()),
    }
}

pub fn proposal_deposit_claim(
    deposit_claim: &ProposalDepositClaim,
) -> pb::EventProposalDepositClaim {
    pb::EventProposalDepositClaim {
        deposit_claim: Some(pb::ProposalDepositClaim::from(*deposit_claim)),
    }
}

pub fn validator_vote(validator_vote: &ValidatorVote, voting_power: u64) -> pb::EventValidatorVote {
    pb::EventValidatorVote {
        vote: Some(pb::ValidatorVote::from(validator_vote.clone())),
        voting_power,
    }
}

pub fn proposal_withdraw(withdraw: &ProposalWithdraw) -> pb::EventProposalWithdraw {
    pb::EventProposalWithdraw {
        withdraw: Some(pb::ProposalWithdraw::from(withdraw.clone())),
    }
}

pub fn proposal_submit(
    submit: &ProposalSubmit,
    start_height: u64,
    end_height: u64,
) -> pb::EventProposalSubmit {
    pb::EventProposalSubmit {
        submit: Some(pb::ProposalSubmit::from(submit.clone())),
        start_height,
        end_height,
    }
}

pub fn proposal_passed(proposal: &Proposal) -> pb::EventProposalPassed {
    pb::EventProposalPassed {
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
