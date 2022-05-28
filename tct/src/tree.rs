use std::collections::HashMap;
use std::fmt::Display;

use decaf377::{FieldExt, Fq};
use hash_hasher::HashedMap;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::*;
use crate::prelude::{Witness as _, *};
use crate::Witness;

#[path = "epoch.rs"]
pub(crate) mod epoch;
pub(crate) use epoch::block;

/// A sparse merkle tree witnessing up to 65,536 epochs of up to 65,536 blocks of up to 65,536
/// [`Commitment`]s.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tree {
    index: HashedMap<Commitment, index::within::Tree>,
    inner: frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>,
}

/// The root hash of a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Root(pub Hash);

impl From<Root> for Fq {
    fn from(root: Root) -> Self {
        root.0.into()
    }
}

/// An error occurred when decoding a tree root from bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("could not decode tree root")]
pub struct RootDecodeError;

impl TryFrom<pb::MerkleRoot> for Root {
    type Error = RootDecodeError;

    fn try_from(root: pb::MerkleRoot) -> Result<Root, Self::Error> {
        let bytes: [u8; 32] = (&root.inner[..]).try_into().map_err(|_| RootDecodeError)?;
        let inner = Fq::from_bytes(bytes).map_err(|_| RootDecodeError)?;
        Ok(Root(Hash::new(inner)))
    }
}

impl From<Root> for pb::MerkleRoot {
    fn from(root: Root) -> Self {
        Self {
            inner: Fq::from(root.0).to_bytes().to_vec(),
        }
    }
}

impl Protobuf<pb::MerkleRoot> for Root {}

impl Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&Fq::from(self.0).to_bytes()))
    }
}

/// The index of a [`Commitment`] within a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position(index::within::Tree);

impl Position {
    /// The index of the [`Commitment`] to which this [`Position`] refers within its own block.
    pub fn commitment(&self) -> u16 {
        self.0.commitment.into()
    }

    /// The index of the block to which this [`Position`] refers within its own epoch.
    pub fn block(&self) -> u16 {
        self.0.block.into()
    }

    /// The index of the epoch to which this [`Position`] refers within its [`Tree`].
    pub fn epoch(&self) -> u16 {
        self.0.epoch.into()
    }
}

impl From<Position> for u64 {
    fn from(position: Position) -> Self {
        position.0.into()
    }
}

impl From<u64> for Position {
    fn from(position: u64) -> Self {
        Position(position.into())
    }
}

impl Height for Tree {
    type Height = <frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>> as Height>::Height;
}

impl Tree {
    /// Create a new empty [`Tree`] for storing all commitments to the end of time.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the root hash of this [`Tree`].
    ///
    /// Internal hashing is performed lazily to prevent unnecessary intermediary hashes from being
    /// computed, so the first hash returned after a long sequence of insertions may take more time
    /// than subsequent calls.
    ///
    /// Computed hashes are cached so that subsequent calls without further modification are very
    /// fast.
    pub fn root(&self) -> Root {
        Root(self.inner.hash())
    }

    /// Add a new [`Commitment`] to the most recent block of the most recent epoch of this [`Tree`].
    ///
    /// If successful, returns the [`Position`] at which the commitment was inserted.
    ///
    /// # Errors
    ///
    /// Returns [`InsertError`] if any of:
    ///
    /// - the [`Tree`] is full,
    /// - the current epoch is full, or
    /// - the current block is full.
    pub fn insert(
        &mut self,
        witness: Witness,
        commitment: Commitment,
    ) -> Result<Position, InsertError> {
        let item = match witness {
            Witness::Keep => commitment.into(),
            Witness::Forget => Hash::of(commitment).into(),
        };

        // Get the position of the insertion, if it would succeed
        let position = (self.inner.position().ok_or(InsertError::Full)?).into();

        // Try to insert the commitment into the latest block
        self.inner
            .update(|epoch| {
                epoch
                    .update(|block| {
                        // Don't insert into a finalized block (this will fail); create a new one
                        // instead (below)
                        if block.is_finalized() {
                            return None;
                        }

                        Some(block
                            .insert(item)
                            .map_err(|_| InsertError::BlockFull))
                    })
                    .flatten()
                    // If the latest block was finalized already or doesn't exist, create a new block and
                    // insert into that block
                    .or_else(|| {
                        // Don't insert into a finalized epoch (this will fail); create a new one
                        // instead (below)
                        if epoch.is_finalized() {
                            return None;
                        }

                        Some(epoch
                            .insert(frontier::Tier::new(item))
                            .map_err(|_| InsertError::EpochFull))
                    })
            })
            .flatten()
            // If the latest epoch was finalized already or doesn't exist, create a new epoch and
            // insert into that epoch
            .unwrap_or_else(|| {
                self.inner
                    .insert(frontier::Tier::new(frontier::Tier::new(item)))
                    .expect("inserting a commitment must succeed because we already checked that the tree is not full");
                Ok(())
            })?;

        // Keep track of the position of this just-inserted commitment in the index, if it was
        // slated to be kept
        if let Witness::Keep = witness {
            if let Some(replaced) = self.index.insert(commitment, position) {
                // This case is handled for completeness, but should not happen in
                // practice because commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(Position(position))
    }

    /// Get a [`Proof`] of inclusion for the commitment at this index in the tree.
    ///
    /// If the index is not witnessed in this tree, return `None`.
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let index = *self.index.get(&commitment)?;

        let (auth_path, leaf) = match self.inner.witness(index) {
            Some(witness) => witness,
            None => panic!(
                "commitment `{:?}` at position `{:?}` must be witnessed because it is indexed",
                commitment, index
            ),
        };

        debug_assert_eq!(leaf, Hash::of(commitment));

        Some(Proof(crate::internal::proof::Proof {
            position: index.into(),
            auth_path,
            leaf: commitment,
        }))
    }

    /// Forget about the witness for the given [`Commitment`].
    ///
    /// Returns `true` if the commitment was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;

        if let Some(&within_epoch) = self.index.get(&commitment) {
            // We forgot something
            forgotten = true;
            // Forget the index for this element in the tree
            let forgotten = self.inner.forget(within_epoch);
            debug_assert!(forgotten);
            // Remove this entry from the index
            self.index.remove(&commitment);
        }

        forgotten
    }

    /// Get the position in this [`Tree`] of the given [`Commitment`], if it is currently witnessed.
    pub fn position_of(&self, commitment: Commitment) -> Option<Position> {
        self.index.get(&commitment).map(|index| Position(*index))
    }

    /// Add a new block all at once to the most recently inserted epoch of this [`Tree`].
    ///
    /// This can be used for two purposes:
    ///
    /// 1. to insert a [`block::Root`] into the tree as a stand-in for an entire un-witnessed block,
    ///    or
    /// 2. to insert a [`block::Builder`] into the tree that was constructed separately.
    ///
    /// The latter [`block::Builder`] API only accelerates tree construction when used in parallel,
    /// but the former [`block::Root`] insertion can be used to accelerate the construction of a
    /// tree even in a single thread, because if the root is already known, only one set of hashes
    /// need be performed, rather than performing hashing for each commitment in the block.
    ///
    /// This function can be called on anything that implements `Into<block::Finalized>`, in
    /// particular:
    ///
    /// - [`block::Root`] (treated as a finalized block with no witnessed commitments).
    /// - [`block::Builder`] (the block is finalized as it is inserted), and of course
    /// - [`block::Finalized`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertBlockError`] containing the inserted block without adding it to the [`Tree`]
    /// if the [`Tree`] is full or the current epoch is full.
    pub fn insert_block(
        &mut self,
        block: impl Into<block::Finalized>,
    ) -> Result<(), InsertBlockError> {
        let block::Finalized { inner, index } = block.into();

        // Convert the top level inside of the block to a tier that can be slotted into the epoch
        // We have this be an `Option` because we need to `take` out of it inside closures
        let mut inner: Option<frontier::Tier<_>> = Some(match inner {
            Insert::Keep(inner) => inner.into(),
            Insert::Hash(hash) => hash.into(),
        });

        // We have this be an `Option` because we need to `take` out of it in closures
        let mut index = Some(index);

        // Finalize the latest block, if it exists and is not yet finalized -- this means that
        // position calculations will be correct, since they will start at the next block
        self.inner
            .update(|epoch| epoch.update(|block| block.finalize()));

        // Get the epoch and block index of the next insertion
        let position = self.inner.position();

        // Insert the block into the latest epoch, or create a new epoch for it if the latest epoch
        // does not exist or is finalized
        self.inner
            .update(|epoch| {
                // If the epoch is finalized, create a new one (below) to insert the block into
                if epoch.is_finalized() {
                    return None;
                }

                if epoch.is_full() {
                    // The current epoch would be full when we tried to insert into it
                    return Some(Err(InsertBlockError::EpochFull(block::Finalized {
                        inner: inner.take().unwrap().finalize_owned().map(Into::into),
                        index: index.take().unwrap(),
                    })));
                }

                epoch
                    .insert(inner.take().unwrap())
                    .expect("inserting into the current epoch must succeed when it is not full");

                Some(Ok(()))
            })
            .flatten()
            .unwrap_or_else(|| {
                if self.inner.is_full() {
                    return Err(InsertBlockError::Full(block::Finalized {
                        inner: inner.take().unwrap().finalize_owned().map(Into::into),
                        index: index.take().unwrap(),
                    }));
                }

                // Create a new epoch and insert the block into it
                self.inner
                    .insert(frontier::Tier::new(inner.take().unwrap()))
                    .expect("inserting a new epoch must succeed when the tree is not full");

                Ok(())
            })?;

        // Extract from the position we recorded earlier what the epoch/block indexes for each
        // inserted commitment should be
        let index::within::Tree { epoch, block, .. } = position
            .expect("insertion succeeded so position must exist")
            .into();

        // Add the index of all commitments in the block to the global index
        for (c, index::within::Block { commitment }) in index.take().unwrap() {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(replaced) = self.index.insert(
                c,
                index::within::Tree {
                    epoch,
                    block,
                    commitment,
                },
            ) {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(())
    }

    /// Explicitly mark the end of the current block in this tree, advancing the position to the
    /// next block.
    pub fn end_block(&mut self) -> Result<&mut Self, InsertBlockError> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let already_finalized = self
            .inner
            .update(|epoch| epoch.update(frontier::Tier::finalize))
            .flatten()
            // If the entire tree or the latest epoch is empty or finalized, the latest block is
            // considered already finalized
            .unwrap_or(true);

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_block(block::Finalized::default())?;
        };

        Ok(self)
    }

    /// Get the root hash of the most recent block in the most recent epoch of this [`Tree`].
    pub fn current_block_root(&self) -> block::Root {
        self.inner
            .focus()
            .and_then(|epoch| {
                let block = epoch.focus()?;
                if block.is_finalized() {
                    None
                } else {
                    Some(block::Root(block.hash()))
                }
            })
            // If there is no latest unfinalized block, we return the hash of the empty unfinalized block
            .unwrap_or_else(|| block::Builder::default().root())
    }

    /// Add a new epoch all at once to this [`Tree`].
    ///
    /// This can be used for two purposes:
    ///
    /// 1. to insert an [`epoch::Root`] into the tree as a stand-in for an entire un-witnessed block,
    ///    or
    /// 2. to insert an [`epoch::Builder`] into the tree that was constructed separately.
    ///
    /// The latter [`epoch::Builder`] API only accelerates tree construction when used in parallel,
    /// but the former [`epoch::Root`] insertion can be used to accelerate the construction of a
    /// tree even in a single thread, because if the root is already known, only one set of hashes
    /// need be performed, rather than performing hashing for each commitment in the epoch.
    ///
    /// This function can be called on anything that implements `Into<epoch::Finalized>`, in
    /// particular:
    ///
    /// - [`epoch::Root`] (treated as a finalized epoch with no witnessed commitments).
    /// - [`epoch::Builder`] (the epoch is finalized as it is inserted), and of course
    /// - [`epoch::Finalized`].
    ///
    /// # Errors
    ///
    /// Returns [`InsertEpochError`] containing the epoch without adding it to the [`Tree`] if the
    /// [`Tree`] is full.
    pub fn insert_epoch(
        &mut self,
        epoch: impl Into<epoch::Finalized>,
    ) -> Result<&mut Self, InsertEpochError> {
        let epoch::Finalized { inner, index } = epoch.into();

        // If the insertion would fail, return an error
        if self.inner.is_full() {
            // There is no room for another epoch to be inserted into the tree
            return Err(InsertEpochError(epoch::Finalized { inner, index }));
        }

        // Convert the top level inside of the epoch to a tier that can be slotted into the tree
        let inner = match inner {
            Insert::Keep(inner) => inner.into(),
            Insert::Hash(hash) => hash.into(),
        };

        // Finalize the latest epoch, if it exists and is not yet finalized -- this means that
        // position calculations will be correct, since they will start at the next epoch
        self.inner.update(|epoch| epoch.finalize());

        // Get the epoch index of the next insertion
        let index::within::Tree { epoch, .. } = self
            .inner
            .position()
            .expect("tree must have a position because it is not full")
            .into();

        // Insert the inner tree of the epoch into the global tree
        self.inner
            .insert(inner)
            .expect("inserting an epoch must succeed when tree is not full");

        // Add the index of all commitments in the epoch to the global tree index
        for (c, index::within::Epoch { block, commitment }) in index {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(replaced) = self.index.insert(
                c,
                index::within::Tree {
                    epoch,
                    block,
                    commitment,
                },
            ) {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = self.inner.forget(replaced);
                debug_assert!(forgotten);
            }
        }

        Ok(self)
    }

    /// Explicitly mark the end of the current epoch in this tree, advancing the position to the
    /// next epoch.
    pub fn end_epoch(&mut self) -> Result<&mut Self, InsertEpochError> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let already_finalized = self
            .inner
            .update(frontier::Tier::finalize)
            // If there is no focused block, the latest block is considered already finalized
            .unwrap_or(true);

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_epoch(epoch::Finalized::default())?;
        };

        Ok(self)
    }

    /// Get the root hash of the most recent epoch in this [`Tree`].
    pub fn current_epoch_root(&self) -> epoch::Root {
        self.inner
            .focus()
            .and_then(|epoch| {
                if epoch.is_finalized() {
                    None
                } else {
                    Some(epoch::Root(epoch.hash()))
                }
            })
            // In the case where there is no latest unfinalized epoch, we return the hash of the
            // empty unfinalized epoch
            .unwrap_or_else(|| epoch::Builder::default().root())
    }

    /// The position in this [`Tree`] at which the next [`Commitment`] would be inserted.
    ///
    /// If the [`Tree`] is full, returns `None`.
    ///
    /// The maximum capacity of a [`Tree`] is 281,474,976,710,656 = 65,536 epochs of 65,536
    /// blocks of 65,536 [`Commitment`]s.
    ///
    /// Note that [`forget`](Tree::forget)ting a commitment does not decrease this; it only
    /// decreases the [`witnessed_count`](Tree::witnessed_count).
    pub fn position(&self) -> Option<Position> {
        Some(Position(self.inner.position()?.into()))
    }

    /// The number of [`Commitment`]s currently witnessed in this [`Tree`].
    ///
    /// Note that [`forget`](Tree::forget)ting a commitment decreases this count, but does not
    /// decrease the [`position`](Tree::position) of the next inserted [`Commitment`].
    pub fn witnessed_count(&self) -> usize {
        self.index.len()
    }

    /// Check whether this [`Tree`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Verify that the inner index of the tree is correct with respect to the tree structure
    /// itself.
    ///
    /// This is an expensive operation that requires traversing the entire tree structure,
    /// building an auxiliary reverse index, and re-hashing every leaf of the tree.
    ///
    /// If this ever returns `Err`, it indicates either a bug in this crate, or a tree that was
    /// deserialized from an untrustworthy source.
    pub fn verify_index(&self) -> Result<(), IndexMalformed> {
        let reverse_index: HashMap<index::within::Tree, Commitment> = self
            .index
            .iter()
            .map(|(commitment, position)| (*position, *commitment))
            .collect();

        let mut errors = vec![];

        self.inner.foreach_witness(|index, leaf| {
            if let Some(commitment) = reverse_index.get(&index.into()) {
                let expected_hash = Hash::of(*commitment);
                if expected_hash != leaf {
                    errors.push(IndexError::HashMismatch {
                        commitment: *commitment,
                        position: index.into(),
                        expected_hash,
                        found_hash: leaf,
                    });
                }
            } else {
                errors.push(IndexError::UnindexedWitness {
                    position: index.into(),
                    found_hash: leaf,
                });
            }
        });

        if errors.is_empty() {
            Ok(())
        } else {
            Err(IndexMalformed { errors })
        }
    }

    /// Verify that every witnessed commitment can be used to generate a proof of inclusion which is
    /// valid with respect to the current root.
    ///
    /// This is an expensive operation that requires traversing the entire tree structure and doing
    /// a lot of hashing.
    ///
    /// If this ever returns `Err`, it indicates either a bug in this crate, or a tree that was
    /// deserialized from an untrustworthy source.
    pub fn verify_all_proofs(&self) -> Result<(), InvalidWitnesses> {
        let root = self.root();

        let mut errors = vec![];

        for (&commitment, &index) in self.index.iter() {
            if let Some(proof) = self.witness(commitment) {
                if proof.verify(root).is_err() {
                    errors.push(WitnessError::InvalidProof {
                        proof: Box::new(proof),
                    });
                }
            } else {
                errors.push(WitnessError::UnwitnessedCommitment {
                    commitment,
                    position: Position(index),
                })
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(InvalidWitnesses { root, errors })
        }
    }
}

/// The index for the tree contained at least one error.
pub struct IndexMalformed {
    /// The errors found in the index.
    pub errors: Vec<IndexError>,
}

pub enum IndexError {
    /// The index is missing a position.
    UnindexedWitness {
        /// The position expected to be present in the index.
        position: Position,
        /// The hash found at that position.
        found_hash: Hash,
    },
    /// A commitment in the index doesn't match the hash in the tree at that position.
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

/// At least one proof generated by the tree failed to verify against the root.
pub struct InvalidWitnesses {
    /// The root of the tree at which the errors were found.
    pub root: Root,
    /// The errors found.
    pub errors: Vec<WitnessError>,
}

pub enum WitnessError {
    /// The index contains a commitment that is not witnessed.
    UnwitnessedCommitment {
        /// The commitment that was not present in the tree.
        commitment: Commitment,
        /// The position at which it was supposed to appear.
        position: Position,
    },
    /// The proof produced by the tree does not verify against the root.
    InvalidProof {
        /// The proof which failed to verify.
        proof: Box<Proof>,
    },
}
