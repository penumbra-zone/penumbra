//! Incremental serialization for the [`Tree`](crate::Tree).

use decaf377::FieldExt;
use futures::{Stream, StreamExt};
use poseidon377::Fq;
use serde::de::Visitor;
use std::pin::Pin;

use crate::prelude::*;
use crate::storage::{Instruction, Point, Size};
use crate::structure::{Kind, Place};
use crate::tree::Position;

pub(crate) mod fq;

/// Options for serializing a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Serializer {
    /// Should the internal hashes of complete nodes be preserved?
    keep_complete: bool,
    /// Should the internal hashes of frontier nodes be preserved?
    keep_frontier: bool,
    /// The minimum position of node which should be included in the serialization.
    minimum_position: Position,
    /// The minimum forgotten version which should be reported for deletion.
    minimum_forgotten: Forgotten,
}

impl Default for Serializer {
    fn default() -> Self {
        Self {
            keep_frontier: false,
            keep_complete: true,
            minimum_position: 0.into(),
            minimum_forgotten: Forgotten::default(),
        }
    }
}

impl Serializer {
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

        is_essential || (is_frontier && self.keep_frontier) || (is_complete && self.keep_complete)
    }

    fn should_keep_children(&self, node: &structure::Node) -> bool {
        node.range().contains(&self.minimum_position)
    }

    /// Create a new default serializer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum position to include in the serialization.
    pub fn position(&mut self, position: Position) -> &mut Self {
        self.minimum_position = position;
        self
    }

    /// Set the minimum forgotten version to include in the serialization of forgettable locations.
    pub fn forgotten(&mut self, forgotten: Forgotten) -> &mut Self {
        self.minimum_forgotten = forgotten;
        self
    }

    /// Set the serializer to omit the frontier hashes in the output (this is the default).
    ///
    /// If frontier hashes are kept, this slightly reduces the amount of computation upon
    /// deserialization, but imposes a constant space overhead on incremental serialization.
    pub fn omit_frontier(&mut self) -> &mut Self {
        self.keep_frontier = false;
        self
    }

    /// Set the serializer to keep the frontier hashes in the output.
    ///
    /// If frontier hashes are kept, this slightly reduces the amount of computation upon
    /// deserialization, but imposes a constant space overhead each time incremental serialization
    /// is performed.
    pub fn keep_frontier(&mut self) -> &mut Self {
        self.keep_frontier = true;
        self
    }

    /// Set the serializer to keep internal complete hashes in the output (this is the default).
    ///
    /// If complete internal hashes are kept, this significantly reduces the amount of computation
    /// upon deserialization, since if they are not cached, a number of hashes proportionate to the
    /// number of witnessed commitments need to be recomputed. However, this also imposes a linear
    /// space overhead on the total amount of serialized data.
    pub fn keep_complete(&mut self) -> &mut Self {
        self.keep_complete = true;
        self
    }

    /// Set the serializer to omit internal complete hashes in the output.
    ///
    /// If complete internal hashes are kept, this significantly reduces the amount of computation
    /// upon deserialization, since if they are not cached, a number of hashes proportionate to the
    /// number of witnessed commitments need to be recomputed. However, this also imposes a linear
    /// space overhead on the total amount of serialized data.
    pub fn omit_complete(&mut self) -> &mut Self {
        self.keep_complete = false;
        self
    }

    /// Serialize a tree's structure into a depth-first pre-order traversal of represented values within it.
    pub fn points_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = Point> + Unpin + 'tree {
        fn points_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = Point> + '_>> {
            Box::pin(stream! {
                let position = node.position();
                let depth = 24 - node.height();
                let children = node.children();

                // If the minimum position is too high, then don't keep this node (but maybe some of its
                // children will be kept)
                if position >= options.minimum_position {
                    if options.should_keep_hash(&node, children.len()) {
                        if let Some(hash) = node.cached_hash() {
                            yield Point {
                                position: position.into(),
                                depth,
                                // The hash as an `Fq`
                                here: hash.into(),
                            };
                        }
                    }

                    // If there is a witnessed commitment, always yield that
                    if let Kind::Leaf { commitment: Some(commitment) } = node.kind() {
                        yield Point {
                            position: position.into(),
                            depth: depth + 1,
                            // The commitment as an `Fq`
                            here: commitment.0,
                        };
                    }
                }

                // Traverse the children in order, provided that the minimum position doesn't preclude this
                if options.should_keep_children(&node) {
                    for child in children {
                        let mut stream = points_inner(options, child);
                        while let Some(point) = stream.next().await {
                            yield point;
                        }
                    }
                }
            })
        }

        points_inner(*self, tree.structure())
    }

    /// Serialize a tree's structure into an iterator of points within it, for use in synchronous contexts.
    pub fn points_iter<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Iterator<Item = Point> + 'tree {
        futures::executor::block_on_stream(self.points_stream(tree))
    }

    /// Serialize a tree's structure into a depth-first pre-order traversal of represented values within it.
    pub fn instructions_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = Instruction> + Unpin + 'tree {
        fn instructions_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = Instruction> + '_>> {
            Box::pin(stream! {
                let position = node.position();
                let children = node.children();
                let is_internal = !children.is_empty();
                let is_complete = matches!(node.place(), Place::Complete);

                // If the minimum position is too high, then don't keep this node (but maybe some of its
                // children will be kept)
                if position >= options.minimum_position {
                    if is_internal {
                        let here = if options.should_keep_hash(&node, children.len()) {
                            node.cached_hash().map(Into::into)
                        } else {
                            None
                        };

                        // Calculate the number of children of the node which are not default
                        let mut size = Size::One; // We are an internal node, so there is at least 1 child
                        for (child, i) in children.iter().zip([Size::One, Size::Two, Size::Three, Size::Four].into_iter()) {
                            if let Some(hash) = child.cached_hash() {
                                // We know that if one of the children is the default finalized hash,
                                // then the rest following it will be, so we will choose not to emit any
                                // children with this hash, saving space
                                if is_complete && hash == Hash::one() {
                                    break;
                                }
                            }
                            size = i;
                        }

                        yield Instruction::Node {
                            here,
                            size,
                        }
                    } else {
                        yield Instruction::Leaf {
                            // This never forces any work, because leaf nodes always have computed hashes
                            here: node.hash().into(),
                        }
                    }

                    // If there is a witnessed commitment, always yield that
                    if let Kind::Leaf { commitment: Some(commitment) } = node.kind() {
                        yield Instruction::Leaf { here: commitment.0 };
                    }
                }

                // Traverse the children in order, provided that the minimum position doesn't preclude this
                if options.should_keep_children(&node) {
                    for child in children {
                        // If the child is a finalized empty hash, don't emit it (or following children)
                        // at all, saving space (this behavior matches with the reported size of the
                        // node, yielded above)
                         if let Some(hash) = child.cached_hash() {
                            if is_complete && hash == Hash::one() {
                                break;
                            }
                        }

                        let mut stream = instructions_inner(options, child);
                        while let Some(point) = stream.next().await {
                            yield point;
                        }
                    }
                }
            })
        }

        instructions_inner(*self, tree.structure())
    }

    /// Serialize a tree's structure into an iterator of instructions to form it, for use in synchronous contexts.
    pub fn instructions_iter<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Iterator<Item = Instruction> + 'tree {
        futures::executor::block_on_stream(self.instructions_stream(tree))
    }

    /// Get a stream of forgotten locations, which can be deleted from incremental storage.
    pub fn forgotten_stream<'tree>(
        &self,
        tree: &'tree crate::Tree,
    ) -> impl Stream<Item = Point> + Unpin + 'tree {
        fn forgotten_inner(
            options: Serializer,
            node: structure::Node,
        ) -> Pin<Box<dyn Stream<Item = Point> + '_>> {
            Box::pin(stream! {
                // Only report nodes (and their children) which are less than the minimum position
                // (because those greater will not have yet been serialized to storage) and greater
                // than or equal to the minimum forgotten version (because those lesser will already
                // have been deleted from storage)
                if node.position() < options.minimum_position && node.forgotten() >= options.minimum_forgotten {
                    let children = node.children();
                    if children.is_empty() {
                        // If there are no children, report the point
                        yield Point {
                            depth: 24 - node.height(),
                            // A node with no children definitely has a precalculated hash, so this
                            // is not evaluating any extra hashes
                            here: node.hash().into(),
                            position: node.position().into(),
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
    ) -> impl Iterator<Item = Point> + 'tree {
        futures::executor::block_on_stream(self.forgotten_stream(tree))
    }
}
