use decaf377::FieldExt;
use futures::{Stream, StreamExt};
use poseidon377::Fq;
use serde::de::Visitor;
use std::pin::Pin;

use crate::deserialize::Point;
use crate::structure::{Kind, Place};
use crate::tree::Position;

pub(crate) mod fq;

/// Serialize a tree's structure into a depth-first pre-order traversal of represented values within it.
pub fn to_points(
    keep_frontier: bool,
    keep_internal: bool,
    minimum_position: Position,
    tree: &crate::Tree,
) -> impl Stream<Item = Point> + '_ {
    to_points_inner(
        keep_frontier,
        keep_internal,
        minimum_position,
        tree.structure(),
    )
}

fn to_points_inner(
    keep_frontier: bool,
    keep_internal: bool,
    minimum_position: Position,
    node: crate::structure::Node,
) -> Pin<Box<dyn Stream<Item = Point> + '_>> {
    Box::pin(stream! {
        let position = node.position();
        let depth = 24 - node.height();
        let children = node.children();

        // If the minimum position is too high, then don't keep this node (but maybe some of its
        // children will be kept)
        if position >= minimum_position {
            if let Some(hash) = node.cached_hash() {
                let is_internal = !children.is_empty();
                let is_frontier = matches!(node.place(), Place::Frontier);

                // We must absolutely keep all hashes which are leaf nodes, because they are the
                // minimum necessary to reconstruct the tree
                let mut keep_hash = !is_internal;

                // Keeping internal nodes is optional, but saves a lot of computation
                if is_internal && keep_internal {
                    keep_hash = true;
                }

                // Keeping frontier nodes is optional, and less usually desired, but saves a small
                // amount of computation
                if is_frontier && keep_frontier {
                    keep_hash = true;
                }

                if keep_hash {
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
        if node.range().contains(&minimum_position) {
            for child in children {
                let mut stream = to_points_inner(keep_frontier, keep_internal, minimum_position, child);
                while let Some(point) = stream.next().await {
                    yield point;
                }
            }
        }
    })
}
