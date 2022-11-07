//! Incremental serialization for the [`Tree`](crate::Tree).

use archery::SharedPointerKind;
use decaf377::FieldExt;
use futures::{Stream, StreamExt};
use poseidon377::Fq;
use serde::de::Visitor;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::prelude::*;
use crate::storage::Write;
use crate::structure::{Kind, Place, Traversal, Traverse};
use crate::tree::Position;

use super::StoredPosition;

pub(crate) mod fq;

/// Options for serializing a tree.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = ""),
    Default(bound = "")
)]
pub(crate) struct Serializer<RefKind: SharedPointerKind> {
    /// The last position stored in storage, to allow for incremental serialization.
    last_stored_position: StoredPosition,
    /// The minimum forgotten version which should be reported for deletion.
    last_forgotten: Forgotten,
    /// The kind of reference used in trees serialized with this serializer.
    _ref_kind: std::marker::PhantomData<RefKind>,
}

/// Data about an internal hash at a particular point in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct InternalHash {
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
    /// Whether the children of the node should be deleted.
    pub delete_children: bool,
}

impl<RefKind: SharedPointerKind + 'static> Serializer<RefKind> {
    fn is_node_fresh(&self, node: &structure::Node<RefKind>) -> bool {
        match self.last_stored_position {
            StoredPosition::Full => false,
            StoredPosition::Position(last_stored_position) => {
                let node_position: u64 = node.position().into();
                let last_stored_position: u64 = last_stored_position.into();

                // If the node is ahead of the last stored position, we need to serialize it
                node_position >= last_stored_position
                    || (
                        // If the height is zero, we don't need to care because the frontier tip is
                        // always serialized
                        node.height() > 0
                        // The harder part: if the node is not ahead of the last stored position, we omitted
                        // serializing it if it was at that time on the frontier, but we can't skip that now
                        && self.was_node_on_previous_frontier(node)
                    )
            }
        }
    }

    fn was_node_on_previous_frontier(&self, node: &structure::Node<RefKind>) -> bool {
        if let StoredPosition::Position(last_stored_position) = self.last_stored_position {
            let last_stored_position: u64 = last_stored_position.into();

            if let Some(last_frontier_tip) = last_stored_position.checked_sub(1) {
                let height = node.height();
                let node_position: u64 = node.position().into();

                // This is true precisely when the node *was* on the frontier at the time
                // when the position was `last_stored_position`: because frontier nodes are
                // not serialized unless they are the leaf, we need to take care of these
                // also: Shift by height * 2 and compare to compare the leading prefixes of
                // the position of the hypothetical frontier tip node as of the last stored
                // position, but only *down to* the height, indicating whether the node we
                // are examining was on the frontier
                node_position >> (height * 2) == last_frontier_tip >> (height * 2)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn node_has_fresh_children(&self, node: &structure::Node<RefKind>) -> bool {
        self.is_node_fresh(node)
            || match self.last_stored_position {
                StoredPosition::Position(last_stored_position) => node
                    .range()
                    // Subtract one from the last-stored position to get the frontier tip as of the
                    // last serialization: if this is in range, some of the node's children might be
                    // worth investigating
                    .contains(&u64::from(last_stored_position).saturating_sub(1).into()),
                StoredPosition::Full => false,
            }
    }

    /// Serialize a tree's structure into a depth-first pre-order traversal of hashes within it.
    // pub fn hashes_stream(
    //     &self,
    //     tree: &crate::Tree<RefKind>,
    // ) -> impl Stream<Item = InternalHash> + Send + Unpin {
    //     fn hashes_inner<'a, 'tree: 'a, RefKind: SharedPointerKind>(
    //         options: Serializer<RefKind>,
    //         node: structure::Node<RefKind>,
    //     ) -> Pin<Box<dyn Stream<Item = InternalHash> + Send>> {
    //         Box::pin(stream! {
    //             let position = node.position();
    //             let height = node.height();
    //             let children = node.children();

    //             if let Some(hash) = node.cached_hash() {
    //                 // A node's hash is recalculable if it has children or if it has a witnessed commitment
    //                 let recalculable = children.len() > 0
    //                     || matches!(
    //                         node.kind(),
    //                         Kind::Leaf {
    //                             commitment: Some(_)
    //                         }
    //                     );

    //                 // A node's hash is essential if it is not recalculable
    //                 let essential = !recalculable;

    //                 // A node is complete if it's not on the frontier
    //                 let complete = node.place() == Place::Complete;

    //                 // A node is fresh if it couldn't have been serialized to storage yet
    //                 let fresh = options.is_node_fresh(&node);

    //                 // We always serialize the frontier leaf hash, even though it's not essential,
    //                 // because it's not going to change
    //                 let frontier_leaf = !complete && matches!(node.kind(), Kind::Leaf { .. });

    //                 // We only need to issue an instruction to delete the children if the node
    //                 // is both essential and also was previously on the frontier
    //                 let delete_children = essential && options.was_node_on_previous_frontier(&node);

    //                 // If a node is not default, fresh, and either essential (i.e. the frontier
    //                 // leaf) or complete, then we should emit a hash for it
    //                 if fresh && (essential || complete || frontier_leaf) {
    //                     yield InternalHash {
    //                         position,
    //                         height,
    //                         hash,
    //                         essential,
    //                         delete_children,
    //                     };
    //                 }
    //             }

    //             // Traverse the children in order, provided that the minimum position doesn't preclude this
    //             if options.node_has_fresh_children(&node) {
    //                 for child in children {
    //                     let mut stream = hashes_inner(options, child);
    //                     while let Some(point) = stream.next().await {
    //                         yield point;
    //                     }
    //                 }
    //             }
    //         })
    //     }

    //     hashes_inner(*self, tree.structure())
    // }

    pub fn hashes(
        self,
        tree: &crate::Tree<RefKind>,
    ) -> Traversal<impl FnMut(&mut Traverse<RefKind>) -> Option<InternalHash>, RefKind> {
        tree.structure()
            .traverse(move |traverse: &mut Traverse<_>| {
                let node = traverse.here();
                let position = node.position();
                let height = node.height();
                let children = node.children();

                let output = if let Some(hash) = node.cached_hash() {
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

                    // A node is fresh if it couldn't have been serialized to storage yet
                    let fresh = self.is_node_fresh(&node);

                    // We always serialize the frontier leaf hash, even though it's not essential,
                    // because it's not going to change
                    let frontier_leaf = !complete && matches!(node.kind(), Kind::Leaf { .. });

                    // We only need to issue an instruction to delete the children if the node
                    // is both essential and also was previously on the frontier
                    let delete_children = essential && self.was_node_on_previous_frontier(&node);

                    // If a node is not default, fresh, and either essential (i.e. the frontier
                    // leaf) or complete, then we should emit a hash for it
                    if fresh && (essential || complete || frontier_leaf) {
                        Some(InternalHash {
                            position,
                            height,
                            hash,
                            essential,
                            delete_children,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Traverse the children in order, provided that the minimum position doesn't preclude this
                if !(self.node_has_fresh_children(node) && traverse.down())
                    && !traverse.next_right()
                {
                    traverse.stop();
                }

                output
            })
    }

    /// Serialize a tree's structure into its commitments, in right-to-left order.
    // pub fn commitments_stream(
    //     &self,
    //     tree: &crate::Tree<RefKind>,
    // ) -> impl Stream<Item = (Position, Commitment)> + Send + Unpin {
    //     fn commitments_inner<RefKind: SharedPointerKind>(
    //         options: Serializer<RefKind>,
    //         node: structure::Node<RefKind>,
    //     ) -> Pin<Box<dyn Stream<Item = (Position, Commitment)> + Send>> {
    //         Box::pin(stream! {
    //             let position = node.position();
    //             let children = node.children();

    //             // If the minimum position is too high, then don't keep this node (but maybe some of
    //             // its children will be kept)
    //             if options.is_node_fresh(&node) {
    //                 // If we're at a witnessed commitment, yield it
    //                 if let Kind::Leaf {
    //                     commitment: Some(commitment),
    //                 } = node.kind()
    //                 {
    //                     yield (position, commitment);
    //                 }
    //             }

    //             // Traverse the children in order, provided that the minimum position doesn't preclude this
    //             if options.node_has_fresh_children(&node) {
    //                 for child in children {
    //                     let mut stream = commitments_inner(options, child);
    //                     while let Some(point) = stream.next().await {
    //                         yield point;
    //                     }
    //                 }
    //             }
    //         })
    //     }

    //     commitments_inner(*self, tree.structure())
    // }

    /// Serialize a tree's structure into an iterator of commitments within it, for use in
    /// synchronous contexts.
    pub fn commitments(
        self,
        tree: &crate::Tree<RefKind>,
    ) -> Traversal<impl FnMut(&mut Traverse<RefKind>) -> Option<(Position, Commitment)>, RefKind>
    {
        tree.structure()
            .traverse(move |traverse: &mut Traverse<_>| {
                let node = traverse.here();
                let position = node.position();

                // If the minimum position is too high, then don't keep this node (but maybe some of
                // its children will be kept)
                let here = if self.is_node_fresh(&node) {
                    // If we're at a witnessed commitment, yield it
                    if let Kind::Leaf {
                        commitment: Some(commitment),
                    } = node.kind()
                    {
                        Some((position, commitment))
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Traverse the children in order, provided that the minimum position doesn't preclude this
                if !(self.node_has_fresh_children(node) && traverse.down())
                    && !traverse.next_right()
                {
                    traverse.stop();
                }

                here
            })
    }

    /// Get a stream of forgotten locations, which can be deleted from incremental storage.
    // pub fn forgotten_stream(
    //     &self,
    //     tree: &crate::Tree<RefKind>,
    // ) -> impl Stream<Item = InternalHash> + Send + Unpin {
    //     fn forgotten_inner<RefKind: SharedPointerKind>(
    //         options: Serializer<RefKind>,
    //         node: structure::Node<RefKind>,
    //     ) -> Pin<Box<dyn Stream<Item = InternalHash> + Send>> {
    //         Box::pin(stream! {
    //             // Only report nodes (and their children) which are less than the last stored position
    //             // (because those greater will not have yet been serialized to storage) and greater
    //             // than or equal to the minimum forgotten version (because those lesser will already
    //             // have been deleted from storage)
    //             let before_last_stored_position = match options.last_stored_position {
    //                 StoredPosition::Full => true,
    //                 StoredPosition::Position(last_stored_position) =>
    //                     // We don't do anything at all if the node position is greater than or equal
    //                     // to the last stored position, because in that case, it, *as well as its
    //                     // children* have never been persisted into storage, so no deletions are
    //                     // necessary to deal with any things that have been forgotten within them
    //                     node.position() < last_stored_position,
    //             };

    //             if before_last_stored_position && node.forgotten() > options.last_forgotten {
    //                 let children = node.children();
    //                 if children.is_empty() {
    //                     // If there are no children, report the point
    //                     // A node with no children definitely has a precalculated hash, so this
    //                     // is not evaluating any extra hashes
    //                     let hash = node.hash().into();
    //                     yield InternalHash {
    //                         position: node.position(),
    //                         height: node.height(),
    //                         hash,
    //                         // All forgotten nodes are essential, because they have nothing
    //                         // beneath them to witness them
    //                         essential: true,
    //                         // All forgotten nodes should cause their children to be deleted
    //                         delete_children: true,
    //                     };
    //                 } else {
    //                     // If there are children, this node was not yet forgotten, but because the
    //                     // node's forgotten version is greater than the minimum forgotten specified
    //                     // in the options, we know there is some child which needs to be accounted for
    //                     for child in children {
    //                         let mut stream = forgotten_inner(options, child);
    //                         while let Some(point) = stream.next().await {
    //                             yield point;
    //                         }
    //                     }
    //                 }
    //             }
    //         })
    //     }

    //     forgotten_inner(*self, tree.structure())
    // }

    pub fn forgotten(
        self,
        tree: &crate::Tree<RefKind>,
    ) -> Traversal<impl FnMut(&mut Traverse<RefKind>) -> Option<InternalHash>, RefKind> {
        tree.structure()
            .traverse(move |traverse: &mut Traverse<_>| {
                let node = traverse.here();

                // Only report nodes (and their children) which are less than the last stored position
                // (because those greater will not have yet been serialized to storage) and greater
                // than or equal to the minimum forgotten version (because those lesser will already
                // have been deleted from storage)
                let before_last_stored_position = match self.last_stored_position {
                    StoredPosition::Full => true,
                    StoredPosition::Position(last_stored_position) =>
                    // We don't do anything at all if the node position is greater than or equal
                    // to the last stored position, because in that case, it, *as well as its
                    // children* have never been persisted into storage, so no deletions are
                    // necessary to deal with any things that have been forgotten within them
                    {
                        node.position() < last_stored_position
                    }
                };

                let output =
                    if before_last_stored_position && node.forgotten() > self.last_forgotten {
                        let children = node.children();
                        if children.is_empty() {
                            // If there are no children, report the point
                            // A node with no children definitely has a precalculated hash, so this
                            // is not evaluating any extra hashes
                            let hash = node.hash().into();
                            Some(InternalHash {
                                position: node.position(),
                                height: node.height(),
                                hash,
                                // All forgotten nodes are essential, because they have nothing
                                // beneath them to witness them
                                essential: true,
                                // All forgotten nodes should cause their children to be deleted
                                delete_children: true,
                            })
                        } else {
                            // If there are children, this node was not yet forgotten, but because the
                            // node's forgotten version is greater than the minimum forgotten specified
                            // in the options, we know there is some child which needs to be accounted for
                            if !traverse.down() && !traverse.next_right() {
                                traverse.stop();
                            }

                            None
                        }
                    } else {
                        // We're ahead of the forgotten-ness of this node, so proceed onwards
                        if !traverse.next_right() {
                            traverse.stop();
                        }

                        None
                    };

                output
            })
    }
}

/// Serialize the changes to a [`Tree`](crate::Tree) into a writer, deleting all forgotten nodes and
/// adding all new nodes.
pub async fn to_writer<W: Write, RefKind: SharedPointerKind + 'static>(
    writer: &mut W,
    tree: &crate::Tree<RefKind>,
) -> Result<(), W::Error> {
    // If the tree is empty, skip doing anything
    if tree.is_empty() {
        return Ok(());
    }

    // Grab the current position stored in storage
    let last_stored_position = writer.position().await?;

    // Grab the last forgotten version stored in storage
    let last_forgotten = writer.forgotten().await?;

    let serializer = Serializer {
        last_forgotten,
        last_stored_position,
        _ref_kind: PhantomData,
    };

    // Update the position
    let position = if let Some(position) = tree.position() {
        StoredPosition::Position(position)
    } else {
        StoredPosition::Full
    };
    if position != last_stored_position {
        writer.set_position(position).await?;
    }

    // Update the forgotten version
    let forgotten = tree.forgotten();
    if forgotten != last_forgotten {
        writer.set_forgotten(forgotten).await?;
    }

    // Write all the new commitments
    let mut new_commitments = serializer.commitments(tree);
    while let Some((position, commitment)) = new_commitments.next() {
        writer.add_commitment(position, commitment).await?;
    }

    // Add all the new hashes and delete all the forgotten points: this is a unified stream, because
    // we process forgotten points and new points identically
    let mut points = serializer.forgotten(tree).chain(serializer.hashes(tree));

    while let Some(InternalHash {
        position,
        height,
        hash,
        essential,
        delete_children,
    }) = points.next()
    {
        // If the hash's children need deletion, that means any remaining hashes or commitments
        // beneath it should be removed, because they are no longer present in the tree
        if delete_children {
            // Calculate the range of positions to delete, based on the height
            let position = u64::from(position);
            let stride = 4u64.pow(height.into());
            let range = position.into()..(position + stride).min(4u64.pow(24) - 1).into();

            // Delete the range of positions
            writer.delete_range(height, range).await?;
        }

        // Deleting children, then adding the hash allows the backend to do a sensibility check that
        // there are no children of essential hashes, if it chooses to.

        // Add the hash, if it wasn't already present (in the case of forgetting things, this serves
        // to ensure that omitted internal hashes are stored when necessary)
        if hash != Hash::one() {
            // Optimization: don't serialize `Hash::one()`, because it will be filled in automatically
            writer.add_hash(position, height, hash, essential).await?;
        }
    }

    Ok(())
}
