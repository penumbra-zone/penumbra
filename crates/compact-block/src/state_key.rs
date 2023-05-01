pub fn stub_compact_block() -> &'static str {
    "compactblock/stub/compact_block"
}
pub fn compact_block(height: u64) -> String {
    format!("compactblock/compact_block/{height}")
}
