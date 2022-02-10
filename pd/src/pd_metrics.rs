use metrics::register_counter;

/// Registers all metrics tracked by `pd`.
pub fn register_all_metrics() {
    register_counter!("node_spent_nullifiers_total");
    register_counter!("node_notes_total");
    register_counter!("node_transactions_total");
}

/// Represents a bundle of structured metrics data.
pub struct MetricsData {
    pub nullifier_count: u64,
    pub note_count: u64,
}
