pub mod state_key {
    pub fn compact_block(height: u64) -> String {
        format!("compactblock/{height:020}")
    }
    pub fn prefix() -> &'static str {
        "compactblock/"
    }

    pub fn key_component(height: u64) -> String {
        format!("{height:020}", height)
    }
}
