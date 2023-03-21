use std::string::String;

pub fn stub_state_commitment_tree() -> &'static str {
    "compactblock/stub/state_commitment_tree"
}
pub fn stub_compact_block() -> &'static str {
    "compactblock/stub/compact_block"
}
pub fn compact_block(height: u64) -> String {
    format!("compactblock/compact_block/{height}")
}
