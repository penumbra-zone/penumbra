use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Merkle tree too large")]
    TooLarge,
    #[error("Already at root")]
    RootHasNoParent,
    #[error("Already at leaf")]
    LeafHasNoChild,
}
