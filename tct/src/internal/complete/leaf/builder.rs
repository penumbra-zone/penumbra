use super::*;

use crate::storage::{
    deserialize::{IResult, Unexpected},
    Instruction,
};

/// A builder for a leaf.
pub struct Builder<Item: Built>(Item::Builder);

impl<Item: Built> Built for Leaf<Item> {
    type Builder = Builder<Item>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder(Item::build(global_position, index))
    }
}

impl<Item: Built> Build for Builder<Item> {
    type Output = Leaf<Item>;

    fn go(self, instruction: Instruction) -> Result<IResult<Self>, Unexpected> {
        self.0.go(instruction).map(|r| r.map(Builder, Leaf))
    }

    fn is_started(&self) -> bool {
        self.0.is_started()
    }

    fn index(&self) -> u64 {
        self.0.index()
    }

    fn height(&self) -> u8 {
        self.0.height()
    }

    fn min_required(&self) -> usize {
        self.0.min_required()
    }
}
