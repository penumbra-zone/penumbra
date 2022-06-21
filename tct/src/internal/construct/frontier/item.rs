use crate::prelude::*;

use crate::internal::frontier::Item;

pub struct Builder;

impl Constructed for Item {
    type Builder = Builder;
}

impl Construct for Builder {
    type Output = Item;

    fn build(global_position: u64, index: u64) -> Self {
        todo!()
    }

    fn go(self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        todo!()
    }

    fn index(&self) -> u64 {
        todo!()
    }

    fn height(&self) -> u8 {
        todo!()
    }

    fn min_required(&self) -> usize {
        todo!()
    }
}
