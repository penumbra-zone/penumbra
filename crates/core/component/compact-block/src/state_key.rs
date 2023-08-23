pub fn compact_block(height: u64) -> String {
    format!("compactblock/{height:020}")
}
pub fn prefix() -> &'static str {
    "compactblock/"
}

pub fn height(height: u64) -> String {
    format!("{height:020}")
}
