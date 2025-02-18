CREATE TABLE IF NOT EXISTS lqt_votes (
    id SERIAL PRIMARY KEY,
    epoch INTEGER NOT NULL,
    incentivized BYTEA NOT NULL,
    power BIGINT NOT NULL,
    tx_id BYTEA NOT NULL,
    rewards_recipient BYTEA NOT NULL
);

CREATE INDEX ON lqt_votes (epoch, incentivized);

CREATE TABLE IF NOT EXISTS lqt_delegator_rewards (
    id SERIAL PRIMARY KEY,  
    epoch INTEGER NOT NULL,
    reward BIGINT NOT NULL,
    rewards_recipieint BYTEA NOT NULL,
    incentivized BYTEA NOT NULL
);

CREATE INDEX ON lqt_delegator_rewards (epoch);

CREATE TABLE IF NOT EXISTS lqt_position_rewards (
    id SERIAL PRIMARY KEY,  
    epoch INTEGER NOT NULL,
    reward BIGINT NOT NULL,
    position BYTEA NOT NULL,
    incentivized BYTEA NOT NULL,
    pair_volume NUMERIC(39) NOT NULL,
    position_volume NUMERIC(39) NOT NULL
);

CREATE INDEX ON lqt_position_rewards (epoch);
