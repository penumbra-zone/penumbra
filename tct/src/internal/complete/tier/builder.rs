use super::*;

pub struct Builder<Item: Built + Height>(<Nested<Item> as Built>::Builder);

impl<Item: GetHash + Height + Built> Built for Tier<Item> {
    type Builder = Builder<Item>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder(<Nested<Item> as Built>::build(global_position, index))
    }
}

impl<Item: GetHash + Height + Built> Build for Builder<Item> {
    type Output = Tier<Item>;

    fn go(self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        use IResult::*;

        self.0
            .go(instruction)
            .map(|r| match r {
                Complete(inner) => Complete(Tier { inner }),
                Incomplete(builder) => Incomplete(Builder(builder)),
            })
            .map_err(|HitBottom(builder)| HitBottom(Builder(builder)))
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
