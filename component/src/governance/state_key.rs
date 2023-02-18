use penumbra_crypto::{stake::IdentityKey, Nullifier};

pub fn next_proposal_id() -> &'static str {
    "governance/next_proposal_id"
}

pub fn proposal_definition(proposal_id: u64) -> String {
    format!("governance/proposal/{proposal_id}/data")
}

pub fn proposal_state(proposal_id: u64) -> String {
    format!("governance/proposal/{proposal_id}/state")
}

pub fn proposal_deposit_amount(proposal_id: u64) -> String {
    format!("governance/proposal/{proposal_id}/deposit_amount")
}

pub fn proposal_voting_start(proposal_id: u64) -> String {
    format!("governance/proposal/{proposal_id}/voting_start")
}

pub fn proposal_voting_end(proposal_id: u64) -> String {
    format!("governance/proposal/{proposal_id}/voting_end")
}

pub fn unfinished_proposals() -> &'static str {
    "governance/unfinished_proposals"
}

pub fn voting_validators_list(proposal_id: u64) -> String {
    format!("governance/proposal/{proposal_id}/validator_vote/")
}

pub fn validator_vote(proposal_id: u64, identity_key: IdentityKey) -> String {
    format!("governance/proposal/{proposal_id}/validator_vote/{identity_key}")
}

pub fn per_proposal_voted_nullifier_lookup(proposal_id: u64, nullifier: &Nullifier) -> String {
    format!("governance/proposal/{proposal_id}/voted_nullifiers/{nullifier}")
}

pub fn rate_data_at_proposal_start(proposal_id: u64, identity_key: IdentityKey) -> String {
    format!("governance/proposal/{proposal_id}/rate_data_at_start/{identity_key}")
}
