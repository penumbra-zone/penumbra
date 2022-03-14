//! Types to distinguish between different kinds of indices, to prevent them from being confused for
//! each other internally.
//!
//! Methods that take `Into<u64>` as an index argument can be given types from the [`within`]
//! module, which are all `Into<u64>`. They can be constructed from types in this module, which are
//! all `From<u16>`.

/// The index of an individual item in a block.
///
/// Create this using `From<u16>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Item(u16);

impl From<u16> for Item {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

/// The index of an individual block in an epoch.
///
/// Create this using `From<u16>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Block(u16);

impl From<u16> for Block {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

/// The index of an individual epoch in an eternity.
///
/// Create this using `From<u16>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Epoch(u16);

impl From<u16> for Epoch {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

/// Indices of individual items within larger structures.
pub mod within {
    use super::*;

    /// The index of an individual item within a block.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Block {
        /// The index of the item within its block.
        pub item: super::Item,
    }

    impl From<Block> for u64 {
        fn from(Block { item: Item(item) }: Block) -> Self {
            item as u64
        }
    }

    /// The index of an individual item within an epoch.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Epoch {
        /// The index of the block within its epoch.
        pub block: super::Block,
        /// The index of the item within its block.
        pub item: super::Item,
    }

    impl From<Epoch> for u64 {
        fn from(
            Epoch {
                block: super::Block(block),
                item: Item(item),
            }: Epoch,
        ) -> Self {
            ((block as u64) << 16) | item as u64
        }
    }

    /// The index of an individual item within an eternity.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Eternity {
        /// The index of the epoch within its eternity.
        pub epoch: super::Epoch,
        /// The index of the block within its epoch.
        pub block: super::Block,
        /// The index of the item within its block.
        pub item: super::Item,
    }

    impl From<Eternity> for u64 {
        fn from(
            Eternity {
                epoch: super::Epoch(epoch),
                block: super::Block(block),
                item: super::Item(item),
            }: Eternity,
        ) -> Self {
            ((epoch as u64) << 32) | ((block as u64) << 16) | item as u64
        }
    }
}
