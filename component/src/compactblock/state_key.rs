use std::string::String;

pub fn stub_state_commitment_tree() -> &'static str {
    "shielded_pool/stub/state_commitment_tree"
}
pub fn stub_compact_block() -> &'static str {
    "shielded_pool/stub/compact_block"
}
pub fn compact_block(height: u64) -> String {
    format!("shielded_pool/compact_block/{height}")
}
