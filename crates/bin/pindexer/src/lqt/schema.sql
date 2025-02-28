CREATE SCHEMA lqt;

CREATE TABLE lqt._params (
  epoch INTEGER PRIMARY KEY,  
  delegator_share NUMERIC(3, 2) NOT NULL,
  gauge_threshold NUMERIC(3, 2) NOT NULL
);

CREATE TABLE lqt._finished_epochs (
    epoch INTEGER PRIMARY KEY
);

CREATE TABLE lqt._available_rewards (
  epoch INTEGER PRIMARY KEY,  
  amount NUMERIC NOT NULL
);

CREATE TABLE lqt._delegator_rewards (
    epoch INTEGER NOT NULL,
    address BYTEA NOT NULL,
    amount NUMERIC NOT NULL,
    PRIMARY KEY (epoch, address)
);

CREATE INDEX ON lqt._delegator_rewards (address);

CREATE TABLE lqt._lp_rewards (
    epoch INTEGER NOT NULL,
    position_id BYTEA NOT NULL,
    asset_id BYTEA NOT NULL,
    amount NUMERIC NOT NULL,
    executions INTEGER NOT NULL,
    um_volume NUMERIC NOT NULL,
    asset_volume NUMERIC NOT NULL,
    um_fees NUMERIC NOT NULL,
    asset_fees NUMERIC NOT NULL,
    points NUMERIC NOT NULL,
    PRIMARY KEY (epoch, position_id)
);

CREATE INDEX ON lqt._lp_rewards (asset_id);

CREATE TABLE IF NOT EXISTS lqt._votes (
    id SERIAL PRIMARY KEY,
    epoch INTEGER NOT NULL,
    power NUMERIC NOT NULL,
    asset_id BYTEA NOT NULL,
    address BYTEA NOT NULL
);

CREATE INDEX ON lqt._votes (epoch);
CREATE INDEX ON lqt._votes (address);

CREATE VIEW lqt.summary AS
WITH vote_summary AS (
    SELECT epoch, SUM(power) AS total_voting_power FROM lqt._votes GROUP BY epoch
), rewards0 AS (
    SELECT
        epoch,
        SUM(lqt._available_rewards.amount) AS epoch_rewards,
        SUM(lqt._delegator_rewards.amount) AS delegator_rewards,
        SUM(lqt._lp_rewards.amount) AS lp_rewards
    FROM lqt._available_rewards
    JOIN lqt._delegator_rewards USING (epoch)
    JOIN lqt._lp_rewards USING (epoch)
    GROUP BY epoch
), rewards1 AS (
    SELECT
        epoch,
        delegator_rewards,
        lp_rewards,
        lp_rewards + delegator_rewards AS total_rewards,
        epoch_rewards - lp_rewards - delegator_rewards AS available_rewards
    FROM rewards0
)
SELECT
    epoch,
    total_voting_power,
    delegator_rewards,
    lp_rewards,
    total_rewards,
    available_rewards,
    delegator_share * available_rewards AS available_delegator_rewards,
    (1 - delegator_share) * available_rewards AS available_lp_rewards
FROM vote_summary
JOIN rewards1 USING (epoch)
CROSS JOIN LATERAL (
    SELECT delegator_share
    FROM lqt._params
    WHERE lqt._params.epoch <= vote_summary.epoch
    ORDER BY lqt._params.epoch DESC
    LIMIT 1
) params;

CREATE VIEW lqt.gauge AS
WITH tallies AS (
    SELECT epoch, asset_id, SUM(power) AS tally
    FROM lqt._votes
    GROUP BY epoch, asset_id
), total AS (
    SELECT epoch, SUM(tally) AS total_tally FROM tallies GROUP BY epoch
)
SELECT  
  epoch,
  asset_id,
  tally AS votes,
  tally / total_tally AS portion,
  gauge_threshold * total_tally - tally AS missing_votes
FROM tallies
JOIN total USING (epoch)
CROSS JOIN LATERAL (
    SELECT gauge_threshold
    FROM lqt._params
    WHERE lqt._params.epoch <= tallies.epoch
    ORDER BY lqt._params.epoch DESC
    LIMIT 1
) params;

CREATE VIEW lqt.delegator_summary AS
WITH delegator_streaks AS (
    WITH epochs AS (
        SELECT
            address,
            epoch,
            LEAD(epoch) OVER (PARTITION BY address ORDER BY epoch ASC) AS next_epoch,
            MAX(epoch) OVER (PARTITION BY address) AS max_epoch
        FROM lqt._votes
    ), gaps AS (
        SELECT DISTINCT ON (address)
            address,
            max_epoch,
            epoch AS gap_start,
            next_epoch AS gap_end
        FROM epochs
        ORDER BY address, next_epoch - epoch > 1 DESC, next_epoch DESC
    ) SELECT
        address,
        CASE
            WHEN max_epoch < (SELECT MAX(epoch) FROM lqt._finished_epochs) THEN 0
            WHEN gap_end - gap_start > 1 THEN max_epoch - gap_end + 1
            ELSE max_epoch - (SELECT MIN(epoch) FROM lqt._finished_epochs) + 1
        END AS streak
        FROM gaps
), stage0 AS (
    SELECT
        address,
        COUNT(*) AS epochs_voted_in,
        SUM(amount) AS total_rewards,
        SUM(power) AS total_voting_power
    FROM lqt._votes
    JOIN lqt._delegator_rewards USING (address)
    GROUP BY address
) SELECT
    address,
    epochs_voted_in,
    total_rewards,
    total_voting_power,
    streak
FROM stage0
JOIN delegator_streaks USING (address);

CREATE VIEW lqt.lps AS
SELECT
    epoch,
    position_id,
    asset_id,
    amount AS rewards,
    executions,
    um_volume,
    asset_volume,
    asset_fees,
    points,
    points / SUM(points) OVER (PARTITION BY epoch) AS point_share
FROM lqt._lp_rewards;
