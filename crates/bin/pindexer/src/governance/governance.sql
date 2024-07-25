CREATE TABLE IF NOT EXISTS governance_proposals (
  proposal_id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  kind JSONB NOT NULL,
  payload JSONB,
  start_block_height BIGINT NOT NULL,
  end_block_height BIGINT NOT NULL,
  state JSONB NOT NULL,
  proposal_deposit_amount BIGINT NOT NULL,
  withdrawn BOOLEAN DEFAULT FALSE,
  withdrawal_reason TEXT
);

CREATE INDEX ON governance_proposals (title text_pattern_ops);
CREATE INDEX ON governance_proposals (kind);
CREATE INDEX ON governance_proposals (start_block_height DESC);
CREATE INDEX ON governance_proposals (end_block_height DESC);
CREATE INDEX ON governance_proposals (state);
CREATE INDEX ON governance_proposals (withdrawn);


CREATE TABLE IF NOT EXISTS governance_validator_votes (
  id SERIAL PRIMARY KEY,
  proposal_id INTEGER NOT NULL,
  identity_key TEXT NOT NULL,
  vote JSONB NOT NULL,
  voting_power BIGINT NOT NULL,
  block_height BIGINT NOT NULL,
  FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX ON governance_validator_votes (proposal_id);
CREATE INDEX ON governance_validator_votes (identity_key);
CREATE INDEX ON governance_validator_votes (vote);
CREATE INDEX ON governance_validator_votes (voting_power);
CREATE INDEX ON governance_validator_votes (block_height);


CREATE TABLE IF NOT EXISTS governance_delegator_votes (
  id SERIAL PRIMARY KEY,
  proposal_id INTEGER NOT NULL,
  identity_key TEXT NOT NULL,
  vote JSONB NOT NULL,
  voting_power BIGINT NOT NULL,
  block_height BIGINT NOT NULL,
  FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX ON governance_delegator_votes (proposal_id);
CREATE INDEX ON governance_delegator_votes (identity_key);
CREATE INDEX ON governance_delegator_votes (vote);
CREATE INDEX ON governance_delegator_votes (voting_power);
CREATE INDEX ON governance_delegator_votes (block_height);

