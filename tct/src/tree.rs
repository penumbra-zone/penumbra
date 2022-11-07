use std::fmt::{Debug, Display};

use archery::{SharedPointer, SharedPointerKind};
use decaf377::{FieldExt, Fq};
use penumbra_proto::{core::crypto::v1alpha1 as pb, Protobuf};

use crate::error::*;
use crate::prelude::{Witness as _, *};
use crate::structure::{Traversal, Traverse};
use crate::Witness;

#[path = "epoch.rs"]
pub(crate) mod epoch;
pub(crate) use epoch::block;

/// A sparse merkle tree witnessing up to 65,536 epochs of up to 65,536 blocks of up to 65,536
/// [`Commitment`]s.
#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Tree<RefKind: SharedPointerKind = archery::ArcK> {
    index: HashedMap<Commitment, index::within::Tree, RefKind>,
    #[allow(clippy::type_complexity)]
    inner: SharedPointer<
        frontier::Top<frontier::Tier<frontier::Tier<frontier::Item, RefKind>, RefKind>, RefKind>,
        RefKind,
    >,
}

impl<RefKind: SharedPointerKind> Default for Tree<RefKind> {
    fn default() -> Self {
        Self {
            index: HashedMap::default(),
            inner: SharedPointer::new(frontier::Top::new(frontier::TrackForgotten::Yes)),
        }
    }
}

impl<RefKind: SharedPointerKind + 'static> PartialEq for Tree<RefKind> {
    fn eq(&self, other: &Tree<RefKind>) -> bool {
        SharedPointer::ptr_eq(&self.inner, &other.inner) || // if the trees are pointer-equal, then they're definitely equal, so no need to check anything else
        (self.position() == other.position() // two trees could have identical contents but different positions
            && self.root() == other.root() // if the roots match, they represent the same commitments, but may witness different ones
            && self.index == other.index) // we ensure they witness the same commitments by checking equality of indices
    }
}

impl<RefKind: SharedPointerKind + 'static> Eq for Tree<RefKind> {}

/// The root hash of a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "pb::MerkleRoot", into = "pb::MerkleRoot")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Root(pub Hash);

impl Root {
    /// Check if this is the root of an empty tree.
    pub fn is_empty(&self) -> bool {
        self.0 == Hash::zero()
    }
}

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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
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

impl From<(u16, u16, u16)> for Position {
    fn from((epoch, block, commitment): (u16, u16, u16)) -> Self {
        Position(index::within::Tree {
            epoch: epoch.into(),
            block: block.into(),
            commitment: commitment.into(),
        })
    }
}

impl From<Position> for (u16, u16, u16) {
    fn from(position: Position) -> Self {
        (position.epoch(), position.block(), position.commitment())
    }
}

impl<RefKind: SharedPointerKind + 'static> Tree<RefKind> {
    /// Create a new empty [`Tree`] for storing all commitments to the end of time.
    pub fn new() -> Self {
        Self::default()
    }

    // Assemble a tree from its two parts without checking any invariants.
    pub(crate) fn unchecked_from_parts(
        index: HashedMap<Commitment, index::within::Tree, RefKind>,
        inner: frontier::Top<
            frontier::Tier<frontier::Tier<frontier::Item, RefKind>, RefKind>,
            RefKind,
        >,
    ) -> Self {
        Self {
            index,
            inner: SharedPointer::new(inner),
        }
    }

    /// Get the root hash of this [`Tree`].
    ///
    /// Internal hashing is performed lazily to prevent unnecessary intermediary hashes from being
    /// computed, so the first hash returned after a long sequence of insertions may take more time
    /// than subsequent calls.
    ///
    /// Computed hashes are cached so that subsequent calls without further modification are very
    /// fast.
    #[instrument(level = "trace", skip(self))]
    pub fn root(&self) -> Root {
        let root = Root(self.inner.hash());
        trace!(?root);
        root
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
    #[instrument(level = "trace", skip(self))]
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
        SharedPointer::make_mut(&mut self.inner)
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
                SharedPointer::make_mut(&mut self.inner)
                    .insert(frontier::Tier::new(frontier::Tier::new(item)))
                    .expect("inserting a commitment must succeed because we already checked that the tree is not full");
                Ok(())
            })
            .map_err(|error| {
                error!(%error); error
            })?;

        // Keep track of the position of this just-inserted commitment in the index, if it was
        // slated to be kept
        if let Witness::Keep = witness {
            // This case is handled for completeness, but should not happen in
            // practice because commitments should be unique:
            if let Some(&replaced) = self.index.get(&commitment) {
                let forgotten = SharedPointer::make_mut(&mut self.inner).forget(replaced);
                debug_assert!(forgotten);
            }

            // After handling that, do the insertion
            self.index.insert_mut(commitment, position);
        }

        let position = Position(position);
        trace!(?position);
        Ok(position)
    }

    /// Get a [`Proof`] of inclusion for the commitment at this index in the tree.
    ///
    /// If the index is not witnessed in this tree, return `None`.
    #[instrument(level = "trace", skip(self))]
    pub fn witness(&self, commitment: Commitment) -> Option<Proof> {
        let &index = if let Some(index) = self.index.get(&commitment) {
            index
        } else {
            trace!("not witnessed");
            return None;
        };

        let (auth_path, leaf) = match self.inner.witness(index) {
            Some(witness) => witness,
            None => panic!(
                "commitment `{:?}` at position `{:?}` must be witnessed because it is indexed",
                commitment, index
            ),
        };

        debug_assert_eq!(leaf, Hash::of(commitment));

        let proof = Proof(crate::internal::proof::Proof {
            position: index.into(),
            auth_path,
            leaf: commitment,
        });

        trace!(?index, ?proof);
        Some(proof)
    }

    /// Forget about the witness for the given [`Commitment`].
    ///
    /// Returns `true` if the commitment was previously witnessed (and now is forgotten), and `false` if
    /// it was not witnessed.
    #[instrument(level = "trace", skip(self))]
    pub fn forget(&mut self, commitment: Commitment) -> bool {
        let mut forgotten = false;

        if let Some(&within_epoch) = self.index.get(&commitment) {
            // We forgot something
            forgotten = true;
            // Forget the index for this element in the tree
            let forgotten = SharedPointer::make_mut(&mut self.inner).forget(within_epoch);
            debug_assert!(forgotten);
            // Remove this entry from the index
            self.index.remove_mut(&commitment);
        }

        trace!(?forgotten);
        forgotten
    }

    /// Get the position in this [`Tree`] of the given [`Commitment`], if it is currently witnessed.
    #[instrument(level = "trace", skip(self))]
    pub fn position_of(&self, commitment: Commitment) -> Option<Position> {
        let position = self.index.get(&commitment).map(|index| Position(*index));
        trace!(?position);
        position
    }

    /// Add a new block all at once to the most recently inserted epoch of this [`Tree`], returning
    /// the block root of the finalized block.
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
    #[instrument(level = "trace", skip(self, block))]
    pub fn insert_block(
        &mut self,
        block: impl Into<block::Finalized<RefKind>>,
    ) -> Result<block::Root, InsertBlockError<RefKind>> {
        // We split apart the inside so that we get the right instrumention when this is called as
        // an inner function in `end_block`
        let block_root = self.insert_block_uninstrumented(block).map_err(|error| {
            error!(%error);
            error
        })?;
        trace!(?block_root);
        Ok(block_root)
    }

    fn insert_block_uninstrumented(
        &mut self,
        block: impl Into<block::Finalized<RefKind>>,
    ) -> Result<block::Root, InsertBlockError<RefKind>> {
        let block::Finalized { inner, index } = block.into();

        // Convert the top level inside of the block to a tier that can be slotted into the epoch
        // We have this be an `Option` because we need to `take` out of it inside closures
        let mut inner: Option<frontier::Tier<_, _>> = Some(match inner {
            Insert::Keep(inner) => inner.into(),
            Insert::Hash(hash) => hash.into(),
        });

        // We have this be an `Option` because we need to `take` out of it in closures
        let mut index = Some(index);

        // Finalize the latest block, if it exists and is not yet finalized -- this means that
        // position calculations will be correct, since they will start at the next block
        SharedPointer::make_mut(&mut self.inner)
            .update(|epoch| epoch.update(|block| block.finalize()));

        // Get the epoch and block index of the next insertion
        let position = self.inner.position();

        // Insert the block into the latest epoch, or create a new epoch for it if the latest epoch
        // does not exist or is finalized
        let block_root = SharedPointer::make_mut(&mut self.inner)
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

                // Get the inner thing from the `Option` storage
                let inner = inner.take().unwrap();

                // Calculate the block root
                let block_root = block::Root(inner.hash());

                epoch
                    .insert(inner)
                    .expect("inserting into the current epoch must succeed when it is not full");

                Some(Ok(block_root))
            })
            .flatten()
            .unwrap_or_else(|| {
                if self.inner.is_full() {
                    return Err(InsertBlockError::Full(block::Finalized {
                        inner: inner.take().unwrap().finalize_owned().map(Into::into),
                        index: index.take().unwrap(),
                    }));
                }

                // Get the inner thing from the `Option` storage
                let inner = inner.take().unwrap();

                // Calculate the block root
                let block_root = block::Root(inner.hash());

                // Create a new epoch and insert the block into it
                SharedPointer::make_mut(&mut self.inner)
                    .insert(frontier::Tier::new(inner))
                    .expect("inserting a new epoch must succeed when the tree is not full");

                Ok(block_root)
            })?;

        // Extract from the position we recorded earlier what the epoch/block indexes for each
        // inserted commitment should be
        let index::within::Tree { epoch, block, .. } = position
            .expect("insertion succeeded so position must exist")
            .into();

        // Add the index of all commitments in the block to the global index
        for (&c, index::within::Block { commitment }) in &index.take().unwrap() {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(&replaced) = self.index.get(&c) {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = SharedPointer::make_mut(&mut self.inner).forget(replaced);
                debug_assert!(forgotten);
            }

            // After handling that case, do the insertion
            self.index.insert_mut(
                c,
                index::within::Tree {
                    epoch,
                    block,
                    commitment: *commitment,
                },
            );
        }

        Ok(block_root)
    }

    /// Explicitly mark the end of the current block in this tree, advancing the position to the
    /// next block, and returning the root of the block which was just finalized.
    #[instrument(level = "trace", skip(self))]
    pub fn end_block(&mut self) -> Result<block::Root, InsertBlockError<RefKind>> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let (already_finalized, finalized_root) = SharedPointer::make_mut(&mut self.inner)
            .update(|epoch| {
                epoch.update(|tier| match tier.finalize() {
                    true => (true, block::Finalized::<RefKind>::default().root()),
                    false => (false, block::Root(tier.hash())),
                })
            })
            .flatten()
            // If the entire tree or the latest epoch is empty or finalized, the latest block is
            // considered already finalized
            .unwrap_or((true, block::Finalized::<RefKind>::default().root()));

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_block_uninstrumented(block::Finalized::default())
                .map_err(|error| {
                    error!(%error);
                    error
                })?;
        };

        trace!(finalized_block_root = ?finalized_root);
        Ok(finalized_root)
    }

    /// Get the root hash of the most recent block in the most recent epoch of this [`Tree`].
    #[instrument(level = "trace", skip(self))]
    pub fn current_block_root(&self) -> block::Root {
        let root = self
            .inner
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
            .unwrap_or_else(|| block::Builder::<RefKind>::default().root());
        trace!(?root);
        root
    }

    /// Add a new epoch all at once to this [`Tree`], returning the root of the finalized epoch
    /// which was inserted.
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
    #[instrument(level = "trace", skip(self, epoch))]
    pub fn insert_epoch(
        &mut self,
        epoch: impl Into<epoch::Finalized<RefKind>>,
    ) -> Result<epoch::Root, InsertEpochError<RefKind>> {
        // We split apart the inside so that we get the right instrumention when this is called as
        // an inner function in `end_epoch`
        let epoch_root = self.insert_epoch_uninstrumented(epoch).map_err(|error| {
            error!(%error);
            error
        })?;
        trace!(?epoch_root);
        Ok(epoch_root)
    }

    fn insert_epoch_uninstrumented(
        &mut self,
        epoch: impl Into<epoch::Finalized<RefKind>>,
    ) -> Result<epoch::Root, InsertEpochError<RefKind>> {
        let epoch::Finalized { inner, index } = epoch.into();

        // If the insertion would fail, return an error
        if self.inner.is_full() {
            // There is no room for another epoch to be inserted into the tree
            return Err(InsertEpochError(epoch::Finalized { inner, index }));
        }

        // Convert the top level inside of the epoch to a tier that can be slotted into the tree
        let inner: frontier::Tier<frontier::Tier<frontier::Item, RefKind>, RefKind> = match inner {
            Insert::Keep(inner) => inner.into(),
            Insert::Hash(hash) => hash.into(),
        };

        // Finalize the latest epoch, if it exists and is not yet finalized -- this means that
        // position calculations will be correct, since they will start at the next epoch
        SharedPointer::make_mut(&mut self.inner).update(|epoch| epoch.finalize());

        // Get the epoch index of the next insertion
        let index::within::Tree { epoch, .. } = self
            .inner
            .position()
            .expect("tree must have a position because it is not full")
            .into();

        // Calculate the root of the finalized epoch we're about to insert
        let epoch_root = epoch::Root(inner.hash());

        // Insert the inner tree of the epoch into the global tree
        SharedPointer::make_mut(&mut self.inner)
            .insert(inner)
            .expect("inserting an epoch must succeed when tree is not full");

        // Add the index of all commitments in the epoch to the global tree index
        for (&c, index::within::Epoch { block, commitment }) in &index {
            // If any commitment is repeated, forget the previous one within the tree, since it is
            // now inaccessible
            if let Some(&replaced) = self.index.get(&c) {
                // This case is handled for completeness, but should not happen in practice because
                // commitments should be unique
                let forgotten = SharedPointer::make_mut(&mut self.inner).forget(replaced);
                debug_assert!(forgotten);
            }

            // After handling the above, do the insertion
            self.index.insert_mut(
                c,
                index::within::Tree {
                    epoch,
                    block: *block,
                    commitment: *commitment,
                },
            );
        }

        Ok(epoch_root)
    }

    /// Explicitly mark the end of the current epoch in this tree, advancing the position to the
    /// next epoch, and returning the root of the epoch which was just finalized.
    #[instrument(level = "trace", skip(self))]
    pub fn end_epoch(&mut self) -> Result<epoch::Root, InsertEpochError<RefKind>> {
        // Check to see if the latest block is already finalized, and finalize it if
        // it is not
        let (already_finalized, finalized_root) = SharedPointer::make_mut(&mut self.inner)
            .update(|tier| match tier.finalize() {
                true => (true, epoch::Finalized::<RefKind>::default().root()),
                false => (false, epoch::Root(tier.hash())),
            })
            // If there is no focused block, the latest block is considered already finalized
            .unwrap_or((true, epoch::Finalized::<RefKind>::default().root()));

        // If the latest block was already finalized (i.e. we are at the start of an unfinalized
        // empty block), insert an empty finalized block
        if already_finalized {
            self.insert_epoch_uninstrumented(epoch::Finalized::default())
                .map_err(|error| {
                    error!(%error);
                    error
                })?;
        };

        trace!(finalized_epoch_root = ?finalized_root);
        Ok(finalized_root)
    }

    /// Get the root hash of the most recent epoch in this [`Tree`].
    #[instrument(level = "trace", skip(self))]
    pub fn current_epoch_root(&self) -> epoch::Root {
        let root = self
            .inner
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
            .unwrap_or_else(|| epoch::Builder::<RefKind>::default().root());
        trace!(?root);
        root
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
    #[instrument(level = "trace", skip(self))]
    pub fn position(&self) -> Option<Position> {
        let position = self.inner.position().map(|p| Position(p.into()));
        trace!(?position);
        position
    }

    /// The count of how many commitments have been forgotten explicitly using
    /// [`forget`](Tree::forget), or implicitly by being overwritten by a subsequent insertion of
    /// the _same_ commitment (this case is rare in practice).
    ///
    /// This does not include commitments that were inserted using [`Witness::Forget`], only those
    /// forgotten subsequent to their insertion.
    #[instrument(level = "trace", skip(self))]
    pub fn forgotten(&self) -> Forgotten {
        let forgotten = self
            .inner
            .forgotten()
            .expect("inner `Top` of `Tree` must always be in forgotten-tracking mode");
        trace!(?forgotten);
        forgotten
    }

    /// The number of [`Commitment`]s currently witnessed in this [`Tree`].
    ///
    /// Note that [`forget`](Tree::forget)ting a commitment decreases this count, but does not
    /// decrease the [`position`](Tree::position) of the next inserted [`Commitment`].
    #[instrument(level = "trace", skip(self))]
    pub fn witnessed_count(&self) -> usize {
        let count = self.index.size();
        trace!(?count);
        count
    }

    /// Check whether this [`Tree`] is empty.
    #[instrument(level = "trace", skip(self))]
    pub fn is_empty(&self) -> bool {
        let is_empty = self.inner.is_empty();
        trace!(?is_empty);
        is_empty
    }

    /// Get an iterator over all commitments currently witnessed in the tree.
    ///
    /// Unlike [`commitments_ordered`](Tree::commitments_ordered), this **does not** guarantee that
    /// commitments will be returned in order, but it may be faster by a constant factor.
    #[instrument(level = "trace", skip(self))]
    pub fn commitments(&self) -> impl Iterator<Item = (Commitment, Position)> + '_ {
        self.index.iter().map(|(c, p)| (*c, Position(*p)))
    }

    /// Get an iterator over all commitments currently witnessed in the tree, **ordered by
    /// position**.
    ///
    /// Unlike [`commitments`](Tree::commitments), this guarantees that commitments will be returned
    /// in order, but it may be slower by a constant factor.
    #[instrument(level = "trace", skip(self))]
    pub fn commitments_ordered(
        &self,
    ) -> Traversal<impl FnMut(&mut Traverse<RefKind>) -> Option<(Position, Commitment)>, RefKind>
    {
        crate::storage::serialize::Serializer::default().commitments(self)
    }

    /// Get a dynamic representation of the internal structure of the tree, which can be traversed
    /// and inspected arbitrarily.
    pub fn structure(&self) -> structure::Node<RefKind> {
        let _structure_span = trace_span!("structure");
        // TODO: use the structure span for instrumenting methods of the structure, as it is traversed
        Node::root(Box::new((*self.inner).clone()))
    }

    /// Deserialize a tree from a [`storage::Read`] backend.
    ///
    /// While trees can be [`serialize`]d incrementally, they can only be deserialized all at once.
    pub async fn deserialize<R: Read>(reader: &mut R) -> Result<Tree<RefKind>, R::Error> {
        storage::from_reader(reader).await
    }

    /// Serialize the tree incrementally to a [`storage::Write`] backend.
    ///
    /// This performs only the operations necessary to serialize the changes to the tree,
    /// synchronizing the in-memory representation with what is stored.
    ///
    /// # Errors
    ///
    /// If the tree stored in the writer is not a prior version of this tree, the writer may throw
    /// errors, in addition to any backend-specific errors related to the storage medium.
    pub async fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), W::Error> {
        storage::to_writer(writer, self).await
    }
}

impl<RefKind: SharedPointerKind + 'static>
    From<frontier::Top<frontier::Tier<frontier::Tier<frontier::Item, RefKind>, RefKind>, RefKind>>
    for Tree<RefKind>
{
    fn from(
        inner: frontier::Top<
            frontier::Tier<frontier::Tier<frontier::Item, RefKind>, RefKind>,
            RefKind,
        >,
    ) -> Self {
        let root = Node::root(Box::new(inner.clone()));

        // Traverse the tree to reconstruct the index
        let index = root
            .traverse(move |traverse: &mut Traverse<RefKind>| {
                let output = if let structure::Kind::Leaf {
                    commitment: Some(commitment),
                } = traverse.here().kind()
                {
                    Some((commitment, traverse.here().position().0))
                } else {
                    None
                };

                if !traverse.down() && !traverse.next_right() {
                    traverse.stop();
                }

                output
            })
            .collect();

        Self {
            inner: SharedPointer::new(inner),
            index,
        }
    }
}
