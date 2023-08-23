pub fn compact_block(height: u64) -> String {
    format!(
        "{}{}",
        crate::state_key::prefix(),
        crate::state_key::height(height)
    )
}
pub fn prefix() -> &'static str {
    "compactblock/"
}

pub fn height(height: u64) -> String {
    format!("{height:020}")
}
