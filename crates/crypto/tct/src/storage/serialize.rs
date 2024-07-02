//! Incremental serialization for the [`Tree`](crate::Tree).

use poseidon377::Fq;
use serde::de::Visitor;

use crate::prelude::*;

pub(crate) mod fq;

/// Options for serializing a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct Serializer {
    /// The last position stored in storage, to allow for incremental serialization.
    last_position: StoredPosition,
    /// The minimum forgotten version which should be reported for deletion.
    last_forgotten: Forgotten,
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

impl Serializer {
    fn is_node_fresh(&self, node: &structure::Node) -> bool {
        match self.last_position {
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

    fn was_node_on_previous_frontier(&self, node: &structure::Node) -> bool {
        if let StoredPosition::Position(last_stored_position) = self.last_position {
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

    fn node_has_fresh_children(&self, node: &structure::Node) -> bool {
        self.is_node_fresh(node)
            || match self.last_position {
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
    pub fn hashes(
        self,
        tree: &crate::Tree,
    ) -> impl Iterator<Item = InternalHash> + Send + Sync + '_ {
        let mut stack = vec![vec![tree.structure()]];

        std::iter::from_fn(move || {
            while let Some(level) = stack.last_mut() {
                if let Some(node) = level.pop() {
                    let position = node.position();
                    let height = node.height();
                    let mut children = node.children();
                    let has_children = !children.is_empty();

                    // Traverse the children in order, provided that the minimum position doesn't preclude this
                    if self.node_has_fresh_children(&node) {
                        children.reverse();
                        stack.push(children);
                    }

                    if let Some(hash) = node.cached_hash() {
                        // A node's hash is recalculable if it has children or if it has a witnessed commitment
                        let recalculable = has_children
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
                        let delete_children =
                            essential && self.was_node_on_previous_frontier(&node);

                        // If a node is not default, fresh, and either essential (i.e. the frontier
                        // leaf) or complete, then we should emit a hash for it
                        if fresh && (essential || complete || frontier_leaf) {
                            return Some(InternalHash {
                                position,
                                height,
                                hash,
                                essential,
                                delete_children,
                            });
                        }
                    }
                } else {
                    stack.pop();
                }
            }

            None
        })
    }

    /// Serialize a tree's structure into its commitments, in right-to-left order.
    pub fn commitments(
        self,
        tree: &crate::Tree,
    ) -> impl Iterator<Item = (Position, StateCommitment)> + Send + Sync + '_ {
        let mut stack = vec![vec![tree.structure()]];

        std::iter::from_fn(move || {
            while let Some(level) = stack.last_mut() {
                if let Some(node) = level.pop() {
                    let position = node.position();
                    let mut children = node.children();

                    // Traverse the children in order, provided that the minimum position doesn't preclude this
                    if self.node_has_fresh_children(&node) {
                        children.reverse();
                        stack.push(children);
                    }

                    // If the minimum position is too high, then don't keep this node (but maybe some of
                    // its children will be kept)
                    if self.is_node_fresh(&node) {
                        // If we're at a witnessed commitment, yield it
                        if let Kind::Leaf {
                            commitment: Some(commitment),
                        } = node.kind()
                        {
                            return Some((position, commitment));
                        }
                    }
                } else {
                    stack.pop();
                }
            }

            None
        })
    }

    /// Get a stream of forgotten locations, which can be deleted from incremental storage.
    pub fn forgotten(
        self,
        tree: &crate::Tree,
    ) -> impl Iterator<Item = InternalHash> + Send + Sync + '_ {
        let mut stack = vec![vec![tree.structure()]];

        std::iter::from_fn(move || {
            while let Some(level) = stack.last_mut() {
                if let Some(node) = level.pop() {
                    // Only report nodes (and their children) which are less than the last stored position
                    // (because those greater will not have yet been serialized to storage) and greater
                    // than or equal to the minimum forgotten version (because those lesser will already
                    // have been deleted from storage)
                    let before_last_stored_position = match self.last_position {
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

                    if before_last_stored_position && node.forgotten() > self.last_forgotten {
                        let mut children = node.children();
                        if children.is_empty() {
                            // If there are no children, report the point
                            // A node with no children definitely has a precalculated hash, so this
                            // is not evaluating any extra hashes
                            let hash = node.hash();
                            return Some(InternalHash {
                                position: node.position(),
                                height: node.height(),
                                hash,
                                // All forgotten nodes are essential, because they have nothing
                                // beneath them to witness them
                                essential: true,
                                // All forgotten nodes should cause their children to be deleted
                                delete_children: true,
                            });
                        } else {
                            // If there are children, this node was not yet forgotten, but because the
                            // node's forgotten version is greater than the minimum forgotten specified
                            // in the options, we know there is some child which needs to be accounted for
                            children.reverse();
                            stack.push(children);
                        }
                    }
                } else {
                    stack.pop();
                }
            }

            None
        })
    }
}

/// Serialize the changes to a [`Tree`](crate::Tree) into an asynchronous writer, deleting all
/// forgotten nodes and adding all new nodes.
pub async fn to_async_writer<W: AsyncWrite>(
    writer: &mut W,
    tree: &crate::Tree,
) -> Result<(), W::Error> {
    // Grab the current position stored in storage
    let last_position = writer.position().await?;

    // Grab the last forgotten version stored in storage
    let last_forgotten = writer.forgotten().await?;

    for update in updates(last_position, last_forgotten, tree) {
        match update {
            Update::SetPosition(position) => writer.set_position(position).await?,
            Update::SetForgotten(forgotten) => writer.set_forgotten(forgotten).await?,
            Update::StoreHash(StoreHash {
                position,
                height,
                hash,
                essential,
            }) => {
                writer.add_hash(position, height, hash, essential).await?;
            }
            Update::StoreCommitment(StoreCommitment {
                position,
                commitment,
            }) => {
                writer.add_commitment(position, commitment).await?;
            }
            Update::DeleteRange(DeleteRange {
                below_height,
                positions,
            }) => {
                writer.delete_range(below_height, positions).await?;
            }
        }
    }

    Ok(())
}

/// Serialize the changes to a [`Tree`](crate::Tree) into a synchronous writer, deleting all
/// forgotten nodes and adding all new nodes.
pub fn to_writer<W: Write>(writer: &mut W, tree: &crate::Tree) -> Result<(), W::Error> {
    // Grab the current position stored in storage
    let last_position = writer.position()?;

    // Grab the last forgotten version stored in storage
    let last_forgotten = writer.forgotten()?;

    for update in updates(last_position, last_forgotten, tree) {
        match update {
            Update::SetPosition(position) => writer.set_position(position)?,
            Update::SetForgotten(forgotten) => writer.set_forgotten(forgotten)?,
            Update::StoreHash(StoreHash {
                position,
                height,
                hash,
                essential,
            }) => {
                writer.add_hash(position, height, hash, essential)?;
            }
            Update::StoreCommitment(StoreCommitment {
                position,
                commitment,
            }) => {
                writer.add_commitment(position, commitment)?;
            }
            Update::DeleteRange(DeleteRange {
                below_height,
                positions,
            }) => {
                writer.delete_range(below_height, positions)?;
            }
        }
    }

    Ok(())
}

/// Create an iterator of all the updates to the tree since the specified last position and last
/// forgotten version.
pub fn updates(
    last_position: impl Into<StoredPosition>,
    last_forgotten: Forgotten,
    tree: &crate::Tree,
) -> impl Iterator<Item = storage::Update> + Send + Sync + '_ {
    if tree.is_empty() {
        None
    } else {
        let last_position = last_position.into();

        let serializer = Serializer {
            last_forgotten,
            last_position,
        };

        let position_updates = Some(if let Some(position) = tree.position() {
            StoredPosition::Position(position)
        } else {
            StoredPosition::Full
        })
        .into_iter()
        .filter(move |&position| position != last_position)
        .map(storage::Update::SetPosition);

        let forgotten_updates = Some(tree.forgotten())
            .into_iter()
            .filter(move |&forgotten| forgotten != last_forgotten)
            .map(storage::Update::SetForgotten);

        let commitment_updates = serializer.commitments(tree).map(|(position, commitment)| {
            storage::Update::StoreCommitment(storage::StoreCommitment {
                position,
                commitment,
            })
        });

        let hash_and_deletion_updates = serializer
            .forgotten(tree)
            .chain(serializer.hashes(tree))
            .flat_map(
                move |InternalHash {
                          position,
                          height,
                          hash,
                          essential,
                          delete_children,
                      }| {
                    let deletion_update = if delete_children {
                        // Calculate the range of positions to delete, based on the height
                        let position = u64::from(position);
                        let stride = 4u64.pow(height.into());
                        let positions =
                            position.into()..(position + stride).min(4u64.pow(24) - 1).into();

                        // Delete the range of positions
                        Some(storage::Update::DeleteRange(storage::DeleteRange {
                            below_height: height,
                            positions,
                        }))
                    } else {
                        None
                    };

                    let hash_update = if hash != Hash::one() {
                        // Optimization: don't serialize `Hash::one()`, because it will be filled in automatically
                        Some(storage::Update::StoreHash(storage::StoreHash {
                            position,
                            height,
                            hash,
                            essential,
                        }))
                    } else {
                        None
                    };

                    // Deleting children, then adding the hash allows the backend to do a sensibility check that
                    // there are no children of essential hashes, if it chooses to.

                    deletion_update.into_iter().chain(hash_update)
                },
            );

        Some(
            position_updates
                .chain(forgotten_updates)
                .chain(commitment_updates)
                .chain(hash_and_deletion_updates),
        )
    }
    .into_iter()
    .flatten()
}
