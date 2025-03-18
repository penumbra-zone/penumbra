CREATE SCHEMA IF NOT EXISTS lqt;

CREATE TABLE IF NOT EXISTS lqt._params (
  epoch INTEGER PRIMARY KEY,  
  delegator_share NUMERIC(3, 2) NOT NULL,
  gauge_threshold NUMERIC(3, 2) NOT NULL
);

CREATE TABLE IF NOT EXISTS lqt._finished_epochs (
    epoch INTEGER PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS lqt._available_rewards (
  epoch INTEGER PRIMARY KEY,  
  amount NUMERIC NOT NULL
);

CREATE TABLE IF NOT EXISTS lqt._delegator_rewards (
    epoch INTEGER NOT NULL,
    address BYTEA NOT NULL,
    amount NUMERIC NOT NULL,
    PRIMARY KEY (epoch, address)
);

CREATE INDEX IF NOT EXISTS idx_lqt_delegator_rewards_address ON lqt._delegator_rewards (address);

CREATE TABLE IF NOT EXISTS lqt._lp_rewards (
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

CREATE INDEX IF NOT EXISTS idx_lqt_lp_rewards_asset_id ON lqt._lp_rewards (asset_id);

CREATE TABLE IF NOT EXISTS lqt._votes (
    id SERIAL PRIMARY KEY,
    epoch INTEGER NOT NULL,
    power NUMERIC NOT NULL,
    asset_id BYTEA NOT NULL,
    address BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lqt_votes_epoch ON lqt._votes (epoch);
CREATE INDEX IF NOT EXISTS idx_lqt_votes_address ON lqt._votes (address);

DROP VIEW IF EXISTS lqt.summary;
CREATE VIEW lqt.summary AS
WITH vote_summary AS (
    SELECT epoch, SUM(power) AS total_voting_power FROM lqt._votes GROUP BY epoch
), rewards0 AS (
    SELECT
        epoch,
        SUM(lqt._available_rewards.amount) AS epoch_rewards,
        SUM(COALESCE(lqt._delegator_rewards.amount, 0)) AS delegator_rewards,
        SUM(COALESCE(lqt._lp_rewards.amount, 0)) AS lp_rewards
    FROM lqt._available_rewards
    LEFT JOIN lqt._delegator_rewards USING (epoch)
    LEFT JOIN lqt._lp_rewards USING (epoch)
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
    COALESCE(total_voting_power, 0) AS total_voting_power,
    delegator_rewards,
    lp_rewards,
    total_rewards,
    available_rewards,
    delegator_share * available_rewards AS available_delegator_rewards,
    (1 - delegator_share) * available_rewards AS available_lp_rewards
FROM rewards1
LEFT JOIN vote_summary USING (epoch)
CROSS JOIN LATERAL (
    SELECT delegator_share
    FROM lqt._params
    WHERE lqt._params.epoch <= rewards1.epoch
    ORDER BY lqt._params.epoch DESC
    LIMIT 1
) params;
COMMENT ON VIEW lqt.summary IS
$$For each epoch / round, this contains a summary of the tournament results.

This will be updated continuously, with the values for the current
epoch / round changing until the round is over.
$$;
COMMENT ON COLUMN lqt.summary.epoch IS
$$The epoch / round of the tournament.$$;
COMMENT ON COLUMN lqt.summary.total_voting_power IS
$$The total amount of voting power used in this round.

Assets get selected for rewards based on their share of this power.

Delegators get rewarded using (among other factors) their share of this power.$$;
COMMENT ON COLUMN lqt.summary.delegator_rewards IS
$$The rewards *issued* to delegators so far.$$;
COMMENT ON COLUMN lqt.summary.lp_rewards IS
$$The rewards *issued* to Liquidity Positions so far.$$;
COMMENT ON COLUMN lqt.summary.total_rewards IS
$$The rewards *issued* to either delegators or LPs.$$;
COMMENT ON COLUMN lqt.summary.available_rewards IS
$$If the round were to end now, how many rewards could be issued?

For an ongoing round, we expect this to increase slowly over the round.$$;
COMMENT ON COLUMN lqt.summary.available_delegator_rewards IS
$$How many of the available rewards could go to delegators.$$;
COMMENT ON COLUMN lqt.summary.available_lp_rewards IS
$$How many of the available rewards could go to LPs.$$;

DROP VIEW IF EXISTS lqt.gauge;
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
  -- t + d >= (T + d) * p
  -- (1 - p) d >= p T - t
  -- d >= (p T - t) / (1 - p)
  CEIL((gauge_threshold * total_tally - tally) / (1 - gauge_threshold))::BIGINT AS missing_votes
FROM tallies
JOIN total USING (epoch)
CROSS JOIN LATERAL (
    SELECT gauge_threshold
    FROM lqt._params
    WHERE lqt._params.epoch <= tallies.epoch
    ORDER BY lqt._params.epoch DESC
    LIMIT 1
) params;
COMMENT ON VIEW lqt.gauge IS
$$For each epoch, contains information about the current tally for each asset.$$;
COMMENT ON COLUMN lqt.gauge.votes IS
$$The voting power cast for this asset, in this round.$$;
COMMENT ON COLUMN lqt.gauge.portion IS
$$The fraction of voting power cast for this asset, in this round.$$;
COMMENT ON COLUMN lqt.gauge.missing_votes IS
$$The voting power needed to reach the reward threshold.

If this value is negative, then this asset has sufficient votes
to receive rewards (which go to LPs providing liquidity on the asset,
to the extent they successfully do so,
and delegators having voted for it).

If this value is positive, then it's the amount of votes it lacks
in order to reach the threshold.
$$;


DROP VIEW IF EXISTS lqt.delegator_summary;
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
COMMENT ON VIEW lqt.delegator_summary IS
$$A summary of a delegator's rewards across all epochs.$$;
COMMENT ON COLUMN lqt.delegator_summary.address IS
$$The reported address of this delegator.

We can only track delegators by the address they report. It is
possible for a delegator to vote using different addresses in
an unlinkable way, if they so choose. There is no way for the
public to distinguish this case from multiple delegators.$$;
COMMENT ON COLUMN lqt.delegator_summary.epochs_voted_in IS
$$The number of rounds this delegator has voted in.$$;
COMMENT ON COLUMN lqt.delegator_summary.total_rewards IS
$$The total rewards this delegator has receieved.$$;
COMMENT ON COLUMN lqt.delegator_summary.total_voting_power IS
$$The total amount of voting power this delegator has exercised.$$;
COMMENT ON COLUMN lqt.delegator_summary.streak IS
$$The number of consecutive rounds voted in, starting from the last finished round.

This does not consider the current round if it has not yet ended.

If the delegator has not voted in the last finished round,
this will be 0.
$$;

DROP VIEW IF EXISTS lqt.delegator_history;
CREATE VIEW lqt.delegator_history AS
SELECT address, epoch, power, asset_id, COALESCE(amount, 0) AS reward
FROM lqt._votes
LEFT JOIN lqt._delegator_rewards USING (address, epoch);
COMMENT ON VIEW lqt.delegator_history IS
$$Contains voting and reward history for a given delegator$$;


DROP VIEW IF EXISTS lqt.lps;
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
COMMENT ON VIEW lqt.lps IS
$$A view of each relevant LP, organized by epoch, and asset.

We have metrics about the execution of the asset.

The most important such metric is "points", which govern
how many rewards the LP will receive, if that asset is selected
by the tournament.
$$;
COMMENT ON COLUMN lqt.lps.point_share IS
$$The percentage of points received in this epoch.$$;
