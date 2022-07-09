//! Incremental serialization for the [`Tree`](crate::Tree).

use decaf377::FieldExt;
use futures::{Stream, StreamExt};
use poseidon377::Fq;
use serde::de::Visitor;
use std::pin::Pin;

use crate::prelude::*;
use crate::storage::Write;
use crate::structure::{Kind, Place};
use crate::tree::Position;

use super::StoredPosition;

pub(crate) mod fq;

/// Options for serializing a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Serializer {
    /// The options for the serialization.
    options: Options,
    /// The last position stored in storage, to allow for incremental serialization.
    last_stored_position: StoredPosition,
    /// The minimum forgotten version which should be reported for deletion.
    last_forgotten: Forgotten,
}

impl Serializer {
    fn is_node_fresh(&self, node: &structure::Node) -> bool {
        match self.last_stored_position {
            StoredPosition::Full => false,
            StoredPosition::Position(last_stored_position) => {
                let node_position: u64 = node.position().into();
                let last_stored_position: u64 = last_stored_position.into();

                // If the node is ahead of the last stored position, we need to serialize it
                node_position >= last_stored_position
                // The harder part: if the node is not ahead of the last stored position, we omitted
                // serializing it if it was at that time on the frontier, but we can't skip that now
                    || if let Some(last_frontier_tip) = last_stored_position.checked_sub(1) {
                         let height = node.height();
                        // If the height is 0 then we don't need to care, because the node already
                        // would have been serialized, since the tip of the frontier is always
                        // serialized
                        height > 0
                        // This is true precisely when the node *was* on the frontier at the time
                        // when the position was `last_stored_position`: because frontier nodes are
                        // not serialized unless they are the leaf, we need to take care of these
                        // also: Shift by height * 2 and compare to compare the leading prefixes of
                        // the position of the hypothetical frontier tip node as of the last stored
                        // position, but only *down to* the height, indicating whether the node we
                        // are examining was on the frontier
                        && node_position >> (height * 2) == last_frontier_tip >> (height * 2)
                    } else {
                        false
                    }
            }
        }
    }

    fn should_keep_hash(&self, node: &structure::Node, children: usize) -> bool {
        // A node's hash is recalculable if it has children or if it has a witnessed commitment
        let is_recalculable = children > 0
            || matches!(
                node.kind(),
                Kind::Leaf {
                    commitment: Some(_)
                }
            );
        // A node's hash is essential (cannot be recalculated from other information) if it is not
        // recalculable
        let is_essential = !is_recalculable;
        // A node is on the frontier if its place matches `Place::Frontier`
        let is_frontier = matches!(node.place(), Place::Frontier);
        // A node is complete if it's not on the frontier
        let is_complete = !is_frontier;

        // A node is fresh if we wouldn't have previously serialized its hash (and commitment, if it
        // has one) by the last stored position
        let is_fresh = self.is_node_fresh(node);

        is_fresh && (is_essential || (is_complete && self.options.keep_internal))
    }

    fn node_has_fresh_children(&self, node: &structure::Node) -> bool {
        self.is_node_fresh(node)
            || match self.last_stored_position {
                StoredPosition::Position(last_stored_position) => {
                    node.range().contains(&last_stored_position)
                }
                StoredPosition::Full => false,
            }
    }

    /// Create a new default serializer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum position to include in the serialization.
    pub fn position(&mut self, position: StoredPosition) -> &mut Self {
        self.last_stored_position = position;
        self
    }

    /// Set the last forgotten version to include in the serialization of forgettable locations.
    pub fn last_forgotten(&mut self, forgotten: Forgotten) -> &mut Self {
        self.last_forgotten = forgotten;
        self
    }

    /// Set the serializer to keep internal complete hashes in the output (this is the default).
    ///
    /// If complete internal hashes are kept, this significantly reduces the amount of computation
    /// upon deserialization, since if they are not cached, a number of hashes proportionate to the
    /// number of witnessed commitments need to be recomputed. However, this also imposes a linear
    /// space overhead on the total amount of serialized data.
    pub fn keep_internal(&mut self) -> &mut Self {
        self.options.keep_internal();
        self
    }

    /// Set the serializer to omit internal complete hashes in the output.
    ///
    /// If complete internal hashes are kept, this significantly reduces the amount of computation
    /// upon deserialization, since if they are not cached, a number of hashes proportionate to the
    /// number of witnessed commitments need to be recomputed. However, this also imposes a linear
    /// space overhead on the total amount of serialized data.
    pub fn omit_internal(&mut self) -> &mut Self {
        self.options.omit_internal();
        self
    }

    /// Serialize a tree's structure into a depth-first pre-order traversal of hashes within it.
    pub fn hashes_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = (Position, u8, Hash)> + Unpin + 'tree {
        fn hashes_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = (Position, u8, Hash)> + '_>> {
            Box::pin(stream! {
                let position = node.position();
                let height = node.height();
                let children = node.children();

                // If the minimum position is too high, then don't keep this node (but maybe some of
                // its children will be kept)
                if options.should_keep_hash(&node, children.len()) {
                    if let Some(hash) = node.cached_hash() {
                        // Optimization: don't write any complete hashes that are equal to
                        // `Hash::one()`, because they will be filled in automatically
                        if !(hash == Hash::one() && node.place() == Place::Complete) {
                            yield (position, height, hash);
                        }
                    }
                }

                // Traverse the children in order, provided that the minimum position doesn't preclude this
                if options.node_has_fresh_children(&node) {
                    for child in children {
                        let mut stream = hashes_inner(options, child);
                        while let Some(point) = stream.next().await {
                            yield point;
                        }
                    }
                }
            })
        }

        hashes_inner(*self, tree.structure())
    }

    /// Serialize a tree's structure into an iterator of hashes within it, for use in synchronous
    /// contexts.
    pub fn hashes_iter<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Iterator<Item = (Position, u8, Hash)> + 'tree {
        futures::executor::block_on_stream(self.hashes_stream(tree))
    }

    /// Serialize a tree's structure into its commitments, in right-to-left order.
    pub fn commitments_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = (Position, Commitment)> + Unpin + 'tree {
        fn commitments_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = (Position, Commitment)> + '_>> {
            Box::pin(stream! {
                let position = node.position();
                let children = node.children();

                // If the minimum position is too high, then don't keep this node (but maybe some of
                // its children will be kept)
                if options.is_node_fresh(&node) {
                    // If we're at a witnessed commitment, yield it
                    if let Kind::Leaf {
                        commitment: Some(commitment),
                    } = node.kind()
                    {
                        yield (position, commitment);
                    }
                }

                // Traverse the children in order, provided that the minimum position doesn't preclude this
                if options.node_has_fresh_children(&node) {
                    for child in children {
                        let mut stream = commitments_inner(options, child);
                        while let Some(point) = stream.next().await {
                            yield point;
                        }
                    }
                }
            })
        }

        commitments_inner(*self, tree.structure())
    }

    /// Serialize a tree's structure into an iterator of commitments within it, for use in
    /// synchronous contexts.
    pub fn commitments_iter<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Iterator<Item = (Position, Commitment)> + 'tree {
        futures::executor::block_on_stream(self.commitments_stream(tree))
    }

    /// Get a stream of forgotten locations, which can be deleted from incremental storage.
    pub fn forgotten_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = (Position, u8, Hash)> + Unpin + 'tree {
        fn forgotten_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = (Position, u8, Hash)> + '_>> {
            Box::pin(stream! {
                // Only report nodes (and their children) which are less than the last stored position
                // (because those greater will not have yet been serialized to storage) and greater
                // than or equal to the minimum forgotten version (because those lesser will already
                // have been deleted from storage)
                let before_last_stored_position = match options.last_stored_position {
                    StoredPosition::Full => true,
                    StoredPosition::Position(last_stored_position) =>
                        // We don't do anything at all if the node position is greater than or equal
                        // to the last stored position, because in that case, it, *as well as its
                        // children* have never been persisted into storage, so no deletions are
                        // necessary to deal with any things that have been forgotten within them
                        node.position() < last_stored_position,
                };

                if before_last_stored_position && node.forgotten() > options.last_forgotten {
                    let children = node.children();
                    if children.is_empty() {
                        // If there are no children, report the point
                        // A node with no children definitely has a precalculated hash, so this
                        // is not evaluating any extra hashes
                        let hash = node.hash().into();
                        // Optimization: don't write any complete hashes that are equal to
                        // `Hash::one()`, because they will be filled in automatically
                        if !(hash == Hash::one() && node.place() == Place::Complete) {
                            yield (
                                node.position().into(),
                                node.height(),
                                hash,
                            );
                        }
                    } else {
                        // If there are children, this node was not yet forgotten, but because the
                        // node's forgotten version is greater than the minimum forgotten specified
                        // in the options, we know there is some child which needs to be accounted for
                        for child in children {
                            let mut stream = forgotten_inner(options, child);
                            while let Some(point) = stream.next().await {
                                yield point;
                            }
                        }
                    }
                }
            })
        }

        forgotten_inner(*self, tree.structure())
    }

    /// Get an iterator of forgotten locations, which can be deleted from incremental storage., for
    /// use in synchronous contexts.
    pub fn forgotten_iter<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Iterator<Item = (Position, u8, Hash)> + 'tree {
        futures::executor::block_on_stream(self.forgotten_stream(tree))
    }
}

/// Options for serializing a tree to a writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Options {
    /// Should the internal hashes of complete nodes be preserved?
    keep_internal: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            keep_internal: true,
        }
    }
}

impl Options {
    /// Set the serializer to keep internal complete hashes in the output (this is the default).
    ///
    /// If complete internal hashes are kept, this significantly reduces the amount of computation
    /// upon deserialization, since if they are not cached, a number of hashes proportionate to the
    /// number of witnessed commitments need to be recomputed. However, this also imposes a linear
    /// space overhead on the total amount of serialized data.
    pub fn keep_internal(&mut self) -> &mut Self {
        self.keep_internal = true;
        self
    }

    /// Set the serializer to omit internal complete hashes in the output.
    ///
    /// If complete internal hashes are kept, this significantly reduces the amount of computation
    /// upon deserialization, since if they are not cached, a number of hashes proportionate to the
    /// number of witnessed commitments need to be recomputed. However, this also imposes a linear
    /// space overhead on the total amount of serialized data.
    pub fn omit_internal(&mut self) -> &mut Self {
        self.keep_internal = false;
        self
    }
}

/// Serialize the changes to a [`Tree`](crate::Tree) into a writer, deleting all forgotten nodes and
/// adding all new nodes.
pub async fn to_writer<W: Write>(
    options: Options,
    last_forgotten: Forgotten,
    writer: &mut W,
    tree: &crate::Tree,
) -> Result<(), W::Error> {
    // If the tree is empty, skip doing anything
    if tree.is_empty() {
        return Ok(());
    }

    // Grab the current position stored in storage
    let last_stored_position = writer.position().await?;

    let serializer = Serializer {
        options,
        last_forgotten,
        last_stored_position,
    };

    // Update the position
    let position = if let Some(position) = tree.position() {
        StoredPosition::Position(position)
    } else {
        StoredPosition::Full
    };
    writer.set_position(position).await?;

    // Write all the new hashes
    let mut new_hashes = serializer.hashes_stream(tree);
    while let Some((position, height, hash)) = new_hashes.next().await {
        writer.add_hash(position, height, hash).await?;
    }

    // Write all the new commitments
    let mut new_commitments = serializer.commitments_stream(tree);
    while let Some((position, commitment)) = new_commitments.next().await {
        writer.add_commitment(position, commitment).await?;
    }

    // Delete all the forgotten points
    let mut forgotten_points = serializer.forgotten_stream(tree);
    while let Some((position, below_height, hash)) = forgotten_points.next().await {
        // Add the hash that was pruned, because previously it may have been skipped, but now it
        // needs to be represented
        writer.add_hash(position, below_height, hash).await?;

        // Calculate the range of positions to delete, based on the height
        let position = u64::from(position);
        let stride = 4u64.pow(below_height.into());
        let range = position.into()..(position + stride).min(4u64.pow(24) - 1).into();

        // Delete the range of positions
        writer.delete_range(below_height, range).await?;
    }

    Ok(())
}
