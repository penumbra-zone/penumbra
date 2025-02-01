// The amount of staking tokens issued for this epoch.
pub fn staking_token_issuance_for_epoch() -> &'static str {
    "distributions/staking_token_issuance_for_epoch"
}

// The amount of LQT rewards issued for this epoch.
pub fn lqt_reward_issuance_for_epoch(epoch_index: u64) -> String {
    format!("distributions/lqt_reward_issuance_for_epoch/{epoch_index:020}")
}

pub fn distributions_parameters() -> &'static str {
    "distributions/parameters"
}
