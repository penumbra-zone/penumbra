pub mod action;

pub use action::{ValidatorVote, ValidatorVoteBody, ValidatorVoteReason};

/// Allowed length of validator vote reason field.
pub const MAX_VALIDATOR_VOTE_REASON_LENGTH: usize = 1024;
