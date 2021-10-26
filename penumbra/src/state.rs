//! In-memory storage of state for MVP1 of the Penumbra node software.

use penumbra_crypto::{merkle, note, Nullifier};

pub struct FullNodeState {
    node_commitment_tree: merkle::BridgeTree<note::Commitment, 32>,
    nullifier_set: Vec<Nullifier>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
