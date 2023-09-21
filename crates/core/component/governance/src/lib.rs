#![deny(clippy::unwrap_used)]
pub mod delegator_vote;
pub use delegator_vote::{
    DelegatorVote, DelegatorVoteBody, DelegatorVoteCircuit, DelegatorVotePlan, DelegatorVoteProof,
    DelegatorVoteView,
};

pub mod proposal_deposit_claim;
pub use proposal_deposit_claim::ProposalDepositClaim;

pub mod validator_vote;
pub use validator_vote::{ValidatorVote, ValidatorVoteBody};

pub mod proposal_submit;
pub use proposal_submit::ProposalSubmit;

pub mod proposal_withdraw;
pub use proposal_withdraw::ProposalWithdraw;

pub mod proposal;
pub use proposal::{Proposal, ProposalKind, ProposalPayload};

pub mod proposal_nft;
pub mod proposal_state;

pub mod voting_receipt_token;

pub use proposal_nft::ProposalNft;
pub use voting_receipt_token::VotingReceiptToken;

pub(crate) mod event;
mod metrics;
pub mod state_key;
pub mod tally;
pub use tally::Tally;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
mod action_handler;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt};

pub mod vote;
pub use vote::Vote;
