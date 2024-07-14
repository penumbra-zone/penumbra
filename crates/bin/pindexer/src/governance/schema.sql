CREATE TYPE proposal_stage AS ENUM ('VOTING', 'FINISHED');
CREATE TYPE proposal_status AS ENUM ('VOTING', 'WITHDRAWN', 'FINISHED', 'CLAIMED');
CREATE TYPE payload_type AS ENUM ('SIGNALING', 'EMERGENCY', 'PARAMETER_CHANGE', 'COMMUNITY_POOL_SPEND', 'UPGRADE_PLAN', 'FREEZE_IBC_CLIENT', 'UNFREEZE_IBC_CLIENT');
CREATE TYPE vote_type AS ENUM ('ABSTAIN', 'YES', 'NO');
CREATE TYPE proposal_outcome AS ENUM ('PASSED', 'FAILED', 'SLASHED');

CREATE TABLE governance_proposals (
id SERIAL PRIMARY KEY,
proposal_id INTEGER NOT NULL,
title TEXT NOT NULL,
description TEXT,
payload_type payload_type NOT NULL,
payload_data JSONB,
start_block_height BIGINT,
end_block_height BIGINT,
start_position BIGINT,
stage proposal_stage NOT NULL,
status proposal_status NOT NULL,
proposal_deposit_amount BIGINT,
outcome proposal_outcome,
is_withdrawn BOOLEAN DEFAULT FALSE,
withdrawal_reason TEXT,
CONSTRAINT check_proposal_id CHECK (proposal_id >= 0),
CONSTRAINT check_start_block_height CHECK (start_block_height >= 0),
CONSTRAINT check_end_block_height CHECK (end_block_height >= 0),
CONSTRAINT check_start_position CHECK (start_position >= 0),
CONSTRAINT check_proposal_deposit_amount CHECK (proposal_deposit_amount >= 0),
CONSTRAINT check_withdrawal_consistency CHECK (
    (is_withdrawn = TRUE AND withdrawal_reason IS NOT NULL) OR
    (is_withdrawn = FALSE AND withdrawal_reason IS NULL)
),
CONSTRAINT check_proposal_outcome CHECK (
    (stage = 'FINISHED' AND outcome IS NOT NULL) OR
    (stage = 'VOTING' AND outcome IS NULL)
),
CONSTRAINT check_stage_status_consistency CHECK (
    (stage = 'VOTING' AND status IN ('VOTING', 'WITHDRAWN')) OR
    (stage = 'FINISHED' AND status IN ('FINISHED', 'CLAIMED'))
),
CONSTRAINT check_outcome_withdrawal_consistency CHECK (
    (is_withdrawn = FALSE AND outcome = 'PASSED') OR
    (is_withdrawn = TRUE AND (outcome != 'PASSED' OR outcome IS NULL))
)
);

CREATE TABLE validator_votes (
id SERIAL PRIMARY KEY,
proposal_id INTEGER NOT NULL,
identity_key TEXT NOT NULL,
vote vote_type NOT NULL,
voting_power BIGINT NOT NULL,
block_height BIGINT NOT NULL,
FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id),
CONSTRAINT check_voting_power CHECK (voting_power >= 0),
CONSTRAINT check_block_height CHECK (block_height >= 0)
);

CREATE TABLE delegator_votes (
id SERIAL PRIMARY KEY,
proposal_id INTEGER NOT NULL,
validator_identity_key TEXT NOT NULL,
vote vote_type NOT NULL,
voting_power BIGINT NOT NULL,
block_height BIGINT NOT NULL,
FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id),
CONSTRAINT check_voting_power CHECK (voting_power >= 0),
CONSTRAINT check_block_height CHECK (block_height >= 0)
);

CREATE TABLE current_block_height (
height BIGINT NOT NULL
);

INSERT INTO current_block_height (height) VALUES (0)
ON CONFLICT (height) DO NOTHING;