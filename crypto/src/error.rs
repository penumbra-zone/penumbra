use thiserror::Error;

#[derive(Error, Debug)]
pub enum MerkleTreeError {
    #[error("Merkle tree too large")]
    TooLarge,
    #[error("Already at root")]
    RootHasNoParent,
    #[error("Already at leaf")]
    LeafHasNoChild,
}
