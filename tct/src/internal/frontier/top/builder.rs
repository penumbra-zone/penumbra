use super::*;

pub struct Builder<T>(T);

impl<Item: Focus + Built> Built for Top<Item> {
    type Builder = Builder<<Item as Built>::Builder>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        todo!()
    }
}

impl<T: Build> Build for Builder<T>
where
    T::Output: Focus,
{
    type Output = Top<T::Output>;

    fn go(self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        todo!()
    }

    fn is_started(&self) -> bool {
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
