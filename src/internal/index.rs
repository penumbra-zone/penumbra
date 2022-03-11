#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Item(u16);

impl From<u16> for Item {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Block(u16);

impl From<u16> for Block {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Epoch(u16);

impl From<u16> for Epoch {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

pub mod within {
    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Block {
        pub item: super::Item,
    }

    impl From<Block> for u64 {
        fn from(Block { item: Item(item) }: Block) -> Self {
            item as u64
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Epoch {
        pub block: super::Block,
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

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Eternity {
        pub epoch: super::Epoch,
        pub block: super::Block,
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
