//! Types to distinguish between different kinds of indices, to prevent them from being confused for
//! each other internally.
//!
//! Methods that take `Into<u64>` as an index argument can be given types from the [`within`]
//! module, which are all `Into<u64>`. They can be constructed from types in this module, which are
//! all `From<u16>`.

use serde::{Deserialize, Serialize};

/// The index of an individual item in a block.
///
/// Create this using `From<u16>`.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Derivative, Serialize, Deserialize,
)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
#[derivative(Debug = "transparent")]
pub struct Commitment(u16);

impl Commitment {
    /// Increment the commitment.
    pub fn increment(&mut self) {
        self.0
            .checked_add(1)
            .expect("block index should never overflow");
    }

    /// The maximum representable commitment index.
    pub const MAX: Self = Self(u16::MAX);
}

impl From<u16> for Commitment {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl From<Commitment> for u16 {
    fn from(commitment: Commitment) -> Self {
        commitment.0
    }
}

/// The index of an individual block in an epoch.
///
/// Create this using `From<u16>`.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Derivative, Serialize, Deserialize,
)]
#[derivative(Debug = "transparent")]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
pub struct Block(u16);

impl From<u16> for Block {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl From<Block> for u16 {
    fn from(block: Block) -> Self {
        block.0
    }
}

impl Block {
    /// Increment the block.
    pub fn increment(&mut self) {
        self.0
            .checked_add(1)
            .expect("block index should never overflow");
    }

    /// The maximum representable block index.
    pub const MAX: Self = Self(u16::MAX);
}

/// The index of an individual epoch in a tree.
///
/// Create this using `From<u16>`.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Derivative, Serialize, Deserialize,
)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
#[derivative(Debug = "transparent")]
pub struct Epoch(u16);

impl From<u16> for Epoch {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl From<Epoch> for u16 {
    fn from(epoch: Epoch) -> Self {
        epoch.0
    }
}

impl Epoch {
    /// Increment the epoch.
    pub fn increment(&mut self) {
        self.0
            .checked_add(1)
            .expect("block index should never overflow");
    }

    /// The maximum epoch index representable.
    pub const MAX: Self = Self(u16::MAX);
}

/// Indices of individual items within larger structures.
pub mod within {
    use super::*;

    /// The index of an individual item within a block.
    #[derive(
        Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
    )]
    #[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
    pub struct Block {
        /// The index of the item within its block.
        pub commitment: super::Commitment,
    }

    impl Block {
        /// The maximum representable index within a block.
        pub const MAX: Self = Self {
            commitment: Commitment::MAX,
        };
    }

    impl From<Block> for u16 {
        fn from(
            Block {
                commitment: Commitment(item),
            }: Block,
        ) -> Self {
            item
        }
    }

    impl From<u16> for Block {
        fn from(position: u16) -> Self {
            Self {
                commitment: Commitment(position),
            }
        }
    }

    impl From<Block> for u64 {
        fn from(block: Block) -> Self {
            u16::from(block) as u64
        }
    }

    /// The index of an individual item within an epoch.
    #[derive(
        Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
    )]
    #[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
    pub struct Epoch {
        /// The index of the block within its epoch.
        pub block: super::Block,
        /// The index of the item within its block.
        pub commitment: super::Commitment,
    }

    impl Epoch {
        /// The maximum representable index within an epoch.
        pub const MAX: Self = Self {
            block: super::Block::MAX,
            commitment: Commitment::MAX,
        };
    }

    impl From<Epoch> for u32 {
        fn from(
            Epoch {
                block: super::Block(block),
                commitment: Commitment(item),
            }: Epoch,
        ) -> Self {
            ((block as u32) << 16) | item as u32
        }
    }

    impl From<u32> for Epoch {
        fn from(position: u32) -> Self {
            let block = (position >> 16) as u16;
            let commitment = position as u16;
            Self {
                block: super::Block(block),
                commitment: Commitment(commitment),
            }
        }
    }

    impl From<Epoch> for u64 {
        fn from(epoch: Epoch) -> Self {
            u32::from(epoch) as u64
        }
    }

    /// The index of an individual item within a tree.
    #[derive(
        Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
    )]
    #[cfg_attr(any(test, feature = "arbitrary"), derive(proptest_derive::Arbitrary))]
    pub struct Tree {
        /// The index of the epoch within its tree.
        pub epoch: super::Epoch,
        /// The index of the block within its epoch.
        pub block: super::Block,
        /// The index of the item within its block.
        pub commitment: super::Commitment,
    }

    impl Tree {
        /// The maximum representable index within a tree.
        pub const MAX: Self = Self {
            epoch: super::Epoch::MAX,
            block: super::Block::MAX,
            commitment: Commitment::MAX,
        };
    }

    impl From<Tree> for u64 {
        fn from(
            Tree {
                epoch: super::Epoch(epoch),
                block: super::Block(block),
                commitment: super::Commitment(item),
            }: Tree,
        ) -> Self {
            ((epoch as u64) << 32) | ((block as u64) << 16) | item as u64
        }
    }

    impl From<u64> for Tree {
        fn from(position: u64) -> Self {
            let epoch = (position >> 32) as u16;
            let block = (position >> 16) as u16;
            let commitment = position as u16;
            Self {
                epoch: super::Epoch(epoch),
                block: super::Block(block),
                commitment: super::Commitment(commitment),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn u64_convert_eternity_inverse(e in 0u16..u16::MAX, b in 0u16..u16::MAX, c in 0u16..u16::MAX) {
            let tree = within::Tree { epoch: e.into(), block: b.into(), commitment: c.into() };
            let position: u64 = tree.into();
            let back_again = position.into();
            assert_eq!(tree, back_again);
        }

        #[test]
        fn u32_convert_epoch_inverse(b in 0u16..u16::MAX, c in 0u16..u16::MAX) {
            let epoch = within::Epoch { block: b.into(), commitment: c.into() };
            let position: u32 = epoch.into();
            let back_again = position.into();
            assert_eq!(epoch, back_again);
        }

        #[test]
        fn u16_convert_block_inverse(c in 0u16..u16::MAX) {
            let block = within::Block { commitment: c.into() };
            let position: u16 = block.into();
            let back_again = position.into();
            assert_eq!(block, back_again);
        }
    }
}
