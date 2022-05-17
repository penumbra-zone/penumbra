//! A specification of the behavior of [`Block`](crate::Block).

use crate::*;

/// A builder for a [`Block`]: a sequence of [`Commitment`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    /// The inner tiers of the builder.
    pub block: List<Insert<Commitment>>,
}

impl Builder {
    /// Insert a new [`Commitment`] into the [`block::Builder`](Builder), returning its [`Position`] if
    /// successful.
    ///
    /// See [`penumbra_tct::Block::insert`].
    pub fn insert(&mut self, witness: Witness, commitment: Commitment) -> Result<(), InsertError> {
        let insert = match witness {
            Witness::Keep => Insert::Keep(commitment),
            Witness::Forget => Insert::Hash(Hash::of(commitment)),
        };

        // Fail if block is full
        if self.block.len() >= TIER_CAPACITY {
            return Err(InsertError::BlockFull);
        }

        // Insert the item into the block
        self.block.push_back(insert);

        Ok(())
    }

    /// Get the root of this unfinalized block builder.
    ///
    /// This operation is expensive, because it constructs the entire block and hashes it, which is
    /// not done in the implementation, because the implementation is incremental.
    pub fn root(&self) -> tct::builder::block::Root {
        let tree = Tree::from_block(false, self.block.clone());
        tct::builder::block::Root(tree.root())
    }
}
