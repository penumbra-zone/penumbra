-- Governance Proposals Table
CREATE TABLE governance_proposals (
    id SERIAL PRIMARY KEY,
    -- The on-chain proposal ID
    proposal_id INTEGER NOT NULL,
    -- The proposal title
    title TEXT NOT NULL,
    -- The proposal description
    description TEXT,
    -- The kind of the proposal
    kind TEXT NOT NULL,
    -- The proposal payload
    payload JSONB,
    -- The height at which voting starts
    start_block_height BIGINT,
    -- The height at which voting ends
    end_block_height BIGINT,
    -- The position of the Tiered Commitment Tree at the start of the proposal
    start_position BIGINT,
    -- The status of the proposal
    status TEXT NOT NULL,
    -- The amount of the deposit which will be slashed if the proposal is rejected
    proposal_deposit_amount BIGINT,
    -- The outcome of the proposal (null if the proposal is still in progress)
    outcome TEXT,
    -- Whether the proposal has been withdrawn
    withdrawn BOOLEAN DEFAULT FALSE,
    -- The reason for the withdrawal (null if the proposal has not been withdrawn)
    withdrawal_reason TEXT
);

CREATE INDEX idx_governance_proposals_id ON governance_proposals(proposal_id);
CREATE INDEX idx_governance_proposals_title ON governance_proposals(title text_pattern_ops);
CREATE INDEX idx_governance_proposals_kind ON governance_proposals(kind);
CREATE INDEX idx_governance_proposals_start_block_height ON governance_proposals(start_block_height DESC);
CREATE INDEX idx_governance_proposals_end_block_height ON governance_proposals(end_block_height DESC);
CREATE INDEX idx_governance_proposals_status ON governance_proposals(status);
CREATE INDEX idx_governance_proposals_outcome ON governance_proposals(outcome);
CREATE INDEX idx_governance_proposals_withdrawn ON governance_proposals(withdrawn);

-- Validator Votes Table
CREATE TABLE governance_validator_votes (
    id SERIAL PRIMARY KEY,
    -- The on-chain proposal ID
    proposal_id INTEGER NOT NULL,
    -- The identity key of the validator
    identity_key TEXT NOT NULL,
    -- The vote of the validator
    vote TEXT NOT NULL,
    -- The voting power of the validator
    voting_power BIGINT NOT NULL,
    -- The height at which the vote was cast
    block_height BIGINT NOT NULL,
    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX idx_governance_validator_votes_proposal_id ON governance_validator_votes(proposal_id);
CREATE INDEX idx_governance_validator_votes_identity_key ON governance_validator_votes(identity_key);
CREATE INDEX idx_governance_validator_votes_vote ON governance_validator_votes(vote);
CREATE INDEX idx_governance_validator_votes_voting_power ON governance_validator_votes(voting_power);
CREATE INDEX idx_governance_validator_votes_block_height ON governance_validator_votes(block_height);

-- Delegator Votes Table
CREATE TABLE governance_delegator_votes (
    id SERIAL PRIMARY KEY,
    -- The on-chain proposal ID
    proposal_id INTEGER NOT NULL,
    -- The identity key of the validator to which the delegator is delegating
    identity_key TEXT NOT NULL,
    -- The vote of the delegator
    vote TEXT NOT NULL,
    -- The voting power of the delegator
    voting_power BIGINT NOT NULL,
    -- The height at which the vote was cast
    block_height BIGINT NOT NULL,
    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX idx_governance_delegator_votes_proposal_id ON governance_delegator_votes(proposal_id);
CREATE INDEX idx_governance_delegator_votes_identity_key ON governance_delegator_votes(identity_key);
CREATE INDEX idx_governance_delegator_votes_vote ON governance_delegator_votes(vote);
CREATE INDEX idx_governance_delegator_votes_voting_power ON governance_delegator_votes(voting_power);
CREATE INDEX idx_governance_delegator_votes_block_height ON governance_delegator_votes(block_height);
