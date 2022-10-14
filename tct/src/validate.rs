//! Validation checks to ensure that [`Tree`]s are well-formed.

use std::{
    collections::BTreeMap,
    fmt::{Display, Write},
};

use crate::prelude::*;

/// Verify that the inner index of the tree is correct with respect to the tree structure
/// itself.
///
/// This is an expensive operation that requires traversing the entire tree structure,
/// building an auxiliary reverse index, and re-hashing every leaf of the tree.
///
/// If this ever returns `Err`, it indicates either a bug in this crate, or a tree that was
/// deserialized from an untrustworthy source.
pub fn index(tree: &Tree) -> Result<(), IndexMalformed> {
    // A reverse index from positions back to the commitments that are supposed to map to their
    // hashes
    let reverse_index: BTreeMap<Position, Commitment> = tree
        .commitments()
        .map(|(commitment, position)| (position, commitment))
        .collect();

    let mut errors = vec![];

    structure::traverse(tree.structure(), &mut |node| {
        if let Kind::Leaf {
            commitment: Some(actual_commitment),
        } = node.kind()
        {
            // We're at a leaf, so check it:
            if let Some(&expected_commitment) = reverse_index.get(&node.position()) {
                if actual_commitment != expected_commitment {
                    errors.push(IndexError::CommitmentMismatch {
                        position: node.position(),
                        expected_commitment,
                        actual_commitment,
                    });
                }
                let expected_hash = Hash::of(actual_commitment);
                if expected_hash != node.hash() {
                    errors.push(IndexError::HashMismatch {
                        commitment: expected_commitment,
                        position: node.position(),
                        expected_hash,
                        found_hash: node.hash(),
                    });
                }
            } else {
                // It's okay for there to be an unindexed witness on the frontier (because the
                // frontier is always represented, even if it's marked for later forgetting),
                // but otherwise we want to ensure that all witnesses are indexed
                errors.push(IndexError::UnindexedWitness {
                    position: node.position(),
                    found_hash: node.hash(),
                });
            };
        }
    });

    // Return an error if any were discovered
    if errors.is_empty() {
        Ok(())
    } else {
        Err(IndexMalformed { errors })
    }
}

/// The index for the tree contained at least one error.
#[derive(Clone, Debug, Error)]
#[error("malformed index:{}", display_errors(.errors))]
pub struct IndexMalformed {
    /// The errors found in the index.
    pub errors: Vec<IndexError>,
}

/// An error occurred when verifying the tree's index.
#[derive(Clone, Debug, Error)]
pub enum IndexError {
    /// The index is missing a position.
    #[error("unindexed position `{position:?}` with hash {found_hash:?}")]
    UnindexedWitness {
        /// The position expected to be present in the index.
        position: Position,
        /// The hash found at that position.
        found_hash: Hash,
    },
    /// A commitment in the index points to a leaf with a different commitment
    #[error("found commitment {actual_commitment:?} at position {position:?} but expected {expected_commitment:?}")]
    CommitmentMismatch {
        /// The position of the leaf that was found to have the wrong commitment.
        position: Position,
        /// The commitment that was expected.
        expected_commitment: Commitment,
        /// The commitment that was found.
        actual_commitment: Commitment,
    },
    /// A commitment in the index doesn't match the hash in the tree at that position.
    #[error("mismatched hash for commitment {commitment:?} at position `{position:?}`: found {found_hash:?}, expected {expected_hash:?}")]
    HashMismatch {
        /// The commitment which should have the found hash.
        commitment: Commitment,
        /// The position that commitment maps to in the index.
        position: Position,
        /// The expected hash value of that commitment.
        expected_hash: Hash,
        /// The actual hash found in the tree structure at the position in the index for that commitment.
        found_hash: Hash,
    },
}

/// Verify that every witnessed commitment can be used to generate a proof of inclusion which is
/// valid with respect to the current root.
///
/// This is an expensive operation that requires traversing the entire tree structure and doing
/// a lot of hashing.
///
/// If this ever returns `Err`, it indicates either a bug in this crate, or a tree that was
/// deserialized from an untrustworthy source.
pub fn all_proofs(tree: &Tree) -> Result<(), InvalidWitnesses> {
    let root = tree.root();

    let mut errors = vec![];

    for (commitment, position) in tree.commitments() {
        if let Some(proof) = tree.witness(commitment) {
            if proof.verify(root).is_err() {
                errors.push(WitnessError::InvalidProof {
                    proof: Box::new(proof),
                });
            }
        } else {
            errors.push(WitnessError::UnwitnessedCommitment {
                commitment,
                position,
            })
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InvalidWitnesses { root, errors })
    }
}

/// At least one proof generated by the tree failed to verify against the root.
#[derive(Clone, Debug, Error)]
#[error(
    "invalid witnesses produced by tree for root {root:?}:{}",
    display_errors(errors)
)]
pub struct InvalidWitnesses {
    /// The root of the tree at which the errors were found.
    pub root: Root,
    /// The errors found.
    pub errors: Vec<WitnessError>,
}

/// An error occurred when verifying the tree's contained witnesses.
#[derive(Clone, Debug, Error)]
pub enum WitnessError {
    /// The index contains a commitment that is not witnessed.
    #[error("unwitnessed commitment {commitment:?} at position `{position:?}`")]
    UnwitnessedCommitment {
        /// The commitment that was not present in the tree.
        commitment: Commitment,
        /// The position at which it was supposed to appear.
        position: Position,
    },
    /// The proof produced by the tree does not verify against the root.
    #[error("invalid proof for commitment {:?} at position `{:?}`", .proof.commitment(), .proof.position())]
    InvalidProof {
        /// The proof which failed to verify.
        proof: Box<Proof>,
    },
}

/// Verify that every internally cached hash matches what it should be, by re-hashing all of them.
///
/// This is an expensive operation that requires traversing the entire tree structure and doing
/// a lot of hashing.
///
/// If this ever returns `Err`, it indicates a bug in this crate.
pub fn cached_hashes(tree: &Tree) -> Result<(), InvalidCachedHashes> {
    use structure::*;

    fn check_hashes(errors: &mut Vec<InvalidCachedHash>, node: Node) {
        // IMPORTANT: we need to traverse children before parent, to avoid overwriting the
        // parent's hash before we have a chance to check it! This is why we don't use
        // `structure::traverse` here, because that is a pre-order traversal.
        for child in node.children() {
            // The frontier is the only place where cached hashes occur
            if child.place() == Place::Frontier {
                check_hashes(errors, child);
            }
        }

        if let Some(cached) = node.cached_hash() {
            // IMPORTANT: we need to clear the cache to actually recompute it!
            node.clear_cached_hash();

            let recomputed = node.hash();

            if cached != recomputed {
                errors.push(InvalidCachedHash {
                    place: node.place(),
                    kind: node.kind(),
                    height: node.height(),
                    index: node.index(),
                    cached,
                    recomputed,
                })
            }
        }
    }

    let mut errors = vec![];
    check_hashes(&mut errors, tree.structure());

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InvalidCachedHashes { errors })
    }
}

/// The tree contained at least one internal cached hash that was incorrect.
#[derive(Clone, Debug, Error)]
#[error("invalid cached hashes:{}", display_errors(.errors))]
pub struct InvalidCachedHashes {
    /// The errors found in the tree.
    pub errors: Vec<InvalidCachedHash>,
}

/// An mismatch between a cached hash and the hash it ought to have been.
#[derive(Clone, Debug, Error)]
#[error("cache for `{place}::{kind}` at height {height}, index {index} is incorrect: found {cached:?}, expected {recomputed:?}")]
pub struct InvalidCachedHash {
    /// The place of the node with the error.
    pub place: Place,
    /// The kind of the node with the error.
    pub kind: Kind,
    /// The height of the node with the error.
    pub height: u8,
    /// The index of the node with the error.
    pub index: u64,
    /// The previous cached hash at that location.
    pub cached: Hash,
    /// The recomputed hash that should have been there.
    pub recomputed: Hash,
}

/// Verify that the internal forgotten versions are consistent throughout the tree.
///
/// This is a relatively expensive operation which requires traversing the entire tree structure.
///
/// If this ever returns `Err`, it indicates a bug in this crate.
pub fn forgotten(tree: &Tree) -> Result<(), InvalidForgotten> {
    use structure::*;

    fn check_forgotten(
        errors: &mut Vec<InvalidForgottenVersion>,
        expected_max: Option<Forgotten>,
        node: Node,
    ) {
        let children = node.children();
        let actual_max = node
            .children()
            .iter()
            .map(Any::forgotten)
            .max()
            .unwrap_or_default();

        if let Some(expected_max) = expected_max {
            // Check the expected forgotten version here
            if actual_max != expected_max {
                errors.push(InvalidForgottenVersion {
                    kind: node.kind(),
                    place: node.place(),
                    height: node.height(),
                    index: node.index(),
                    expected_max,
                    actual_max,
                })
            }

            // Check the children
            for child in children {
                check_forgotten(errors, Some(child.forgotten()), child);
            }
        }
    }

    let mut errors = vec![];
    check_forgotten(&mut errors, None, tree.structure());

    if errors.is_empty() {
        Ok(())
    } else {
        Err(InvalidForgotten { errors })
    }
}

/// The tree contained at least one discrepancy in the internal forgotten versions of its nodes.
#[derive(Clone, Debug, Error)]
#[error("invalid forgotten versions:{}", display_errors(.errors))]
pub struct InvalidForgotten {
    /// The errors found in the tree.
    pub errors: Vec<InvalidForgottenVersion>,
}

/// A mismatch between the expected maximum forgotten version and the actual one.
#[derive(Clone, Debug, Error)]
#[error("forgotten version mismatch for `{place}::{kind}` at height {height}, index {index}: found {actual_max:?}, expected {expected_max:?}")]
pub struct InvalidForgottenVersion {
    /// The place of the node with the error.
    pub place: Place,
    /// The kind of the node with the error.
    pub kind: Kind,
    /// The height of the node with the error.
    pub height: u8,
    /// The index of the node with the error.
    pub index: u64,
    /// The actual maximum forgotten version.
    pub actual_max: Forgotten,
    /// The expected maximum forgotten version.
    pub expected_max: Forgotten,
}

// A helper function to display a line-separated list of errors
fn display_errors(errors: impl IntoIterator<Item = impl Display>) -> String {
    let mut output = String::new();
    for error in errors.into_iter() {
        write!(&mut output, "\n  {}", error).unwrap();
    }
    output
}
