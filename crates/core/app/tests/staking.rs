//! Integration tests for the Penumbra application's staking component.

#[path = "staking/app_can_define_and_delegate_to_a_validator.rs"]
mod app_can_define_and_delegate_to_a_validator;

#[path = "staking/app_tracks_uptime_for_genesis_validator_missing_blocks.rs"]
mod app_tracks_uptime_for_genesis_validator_missing_blocks;

#[path = "staking/app_tracks_uptime_for_genesis_validator_signing_blocks.rs"]
mod app_tracks_uptime_for_genesis_validator_signing_blocks;

#[path = "staking/app_tracks_uptime_for_validators_only_once_active.rs"]
mod app_tracks_uptime_for_validators_only_once_active;
