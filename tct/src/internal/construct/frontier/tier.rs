use crate::prelude::*;

use crate::internal::frontier::Tier;

pub struct Builder<T>(T);

impl<Item: Focus + Constructed> Constructed for Tier<Item> {
    type Builder = Builder<<Item as Constructed>::Builder>;
}

impl<T: Construct> Construct for Builder<T>
where
    T::Output: Focus,
{
    type Output = Tier<T::Output>;

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
