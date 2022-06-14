use jmt::KeyHash;

pub fn slashed_validators(height: u64) -> KeyHash {
    format!("staking/slashed_validators/{}", height).into()
}
