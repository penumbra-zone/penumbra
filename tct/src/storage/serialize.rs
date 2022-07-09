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
    /// The last position stored in storage, to allow for incremental serialization.
    last_stored_position: StoredPosition,
    /// The minimum forgotten version which should be reported for deletion.
    last_forgotten: Forgotten,
}

/// Data about an internal hash at a particular point in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InternalHash {
    /// The position of the hash.
    pub position: Position,
    /// The height of the hash.
    pub height: u8,
    /// The hash.
    pub hash: Hash,
    /// Whether the hash is essential to be serialized.
    ///
    /// If this is `false`, that means this hash could be omitted and deserialization would be
    /// correct, but slower.
    pub essential: bool,
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

    /// Serialize a tree's structure into a depth-first pre-order traversal of hashes within it.
    pub fn hashes_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = InternalHash> + Unpin + 'tree {
        fn hashes_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = InternalHash> + '_>> {
            Box::pin(stream! {
                let position = node.position();
                let height = node.height();
                let children = node.children();

                if let Some(hash) = node.cached_hash() {
                    // A node's hash is recalculable if it has children or if it has a witnessed commitment
                    let recalculable = children.len() > 0
                        || matches!(
                            node.kind(),
                            Kind::Leaf {
                                commitment: Some(_)
                            }
                        );

                    // A node's hash is essential if it is not recalculable
                    let essential = !recalculable;

                    // A node is complete if it's not on the frontier
                    let complete = node.place() == Place::Complete;

                    // Optimization: don't write any complete hashes that are equal to
                    // `Hash::one()`, because they will be filled in automatically
                    let default = hash == Hash::one() && complete;

                    // A node is fresh if it couldn't have been serialized to storage yet
                    let fresh = options.is_node_fresh(&node);

                    // If a node is not default, fresh, and either essential (i.e. the frontier
                    // leaf) or complete, then we should emit a hash for it
                    if !default && fresh && (essential || complete) {
                        yield InternalHash {
                            position,
                            height,
                            hash,
                            essential,
                        };
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
    ) -> impl Iterator<Item = InternalHash> + 'tree {
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
    ) -> impl Stream<Item = InternalHash> + Unpin + 'tree {
        fn forgotten_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = InternalHash> + '_>> {
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
                            yield InternalHash {
                                position: node.position(),
                                height: node.height(),
                                hash,
                                // All forgotten nodes are essential, because they have nothing
                                // beneath them to witness them
                                essential: true,
                            };
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
    ) -> impl Iterator<Item = InternalHash> + 'tree {
        futures::executor::block_on_stream(self.forgotten_stream(tree))
    }
}

/// Serialize the changes to a [`Tree`](crate::Tree) into a writer, deleting all forgotten nodes and
/// adding all new nodes.
pub async fn to_writer<W: Write>(
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

    // Write all the new commitments
    let mut new_commitments = serializer.commitments_stream(tree);
    while let Some((position, commitment)) = new_commitments.next().await {
        writer.add_commitment(position, commitment).await?;
    }

    // Add all the new hashes and delete all the forgotten points: this is a unified stream, because
    // we process forgotten points and new points identically
    let mut points = serializer
        .hashes_stream(tree)
        .chain(serializer.forgotten_stream(tree));

    while let Some(InternalHash {
        position,
        height,
        hash,
        essential,
    }) = points.next().await
    {
        // Add the hash, if it wasn't already present (in the case of forgetting things, this serves
        // to ensure that omitted internal hashes are stored when necessary)
        writer.add_hash(position, height, hash).await?;

        // If the hash is essential, that means any remaining hashes or commitments beneath it
        // should be removed, because they are no longer present in the tree
        if essential {
            // Calculate the range of positions to delete, based on the height
            let position = u64::from(position);
            let stride = 4u64.pow(height.into());
            let range = position.into()..(position + stride).min(4u64.pow(24) - 1).into();

            // Delete the range of positions
            writer.delete_range(height, range).await?;
        }
    }

    Ok(())
}
