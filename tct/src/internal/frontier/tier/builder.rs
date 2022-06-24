use super::*;

/// A builder for a frontier tier.
pub struct Builder<Item: Built + Focus>
where
    Item::Complete: Built,
{
    index: u64,
    global_position: u64,
    inner: Option<Inner<Item>>,
}

enum Inner<Item: Built + Focus>
where
    Item::Complete: Built,
{
    Frontier(<Nested<Item> as Built>::Builder),
    Complete(<<Nested<Item> as Focus>::Complete as Built>::Builder),
}

impl<Item: Focus + Built + Height> Built for Tier<Item>
where
    Item::Complete: Built,
{
    type Builder = Builder<Item>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder {
            index,
            global_position,
            inner: None,
        }
    }
}

impl<Item: Built + Focus> Build for Builder<Item>
where
    Item::Complete: Built,
{
    type Output = Tier<Item>;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, InvalidInstruction<Self>> {
        use {IResult::*, Instruction::*};

        if let Some(inner) = self.inner {
            // If we're already building something, pass the instruction along to the inside:
            match inner {
                Inner::Frontier(builder) => match builder.go(instruction) {
                    Err(InvalidInstruction { incomplete, unexpected }) => {
                        self.inner = Some(Inner::Frontier(incomplete));
                        Err(InvalidInstruction { incomplete: self, unexpected })
                    }
                    Ok(Incomplete(builder)) => {
                        self.inner = Some(Inner::Frontier(builder));
                        Ok(Incomplete(self))
                    }
                    Ok(Complete(frontier)) => Ok(Complete(Tier {
                        inner: super::Inner::Frontier(Box::new(frontier)),
                    })),
                },
                Inner::Complete(builder) => match builder.go(instruction) {
                    Err(InvalidInstruction { incomplete, unexpected }) => {
                        self.inner = Some(Inner::Complete(incomplete));
                        Err(InvalidInstruction { incomplete: self, unexpected })
                    }
                    Ok(Incomplete(builder)) => {
                        self.inner = Some(Inner::Complete(builder));
                        Ok(Incomplete(self))
                    }
                    Ok(Complete(complete)) => Ok(Complete(Tier {
                        inner: super::Inner::Complete(complete),
                    })),
                },
            }
        } else if let Leaf { here } = instruction {
            // If we're not yet building anything and we receive our first instruction as a `Leaf`,
            // then immediately return a completed hashed tier
            Ok(Complete(Tier {
                inner: super::Inner::Hash(Hash::new(here)),
            }))
        } else {
            // Otherwise, our instruction is to builder a witnessed tier, so set that up
            // and follow the instruction:

            // In which we do some math to determine whether or not the node is on the frontier...

            // The height of this tier
            let height = <Self::Output as Height>::Height::HEIGHT;
            // The number of positions each index increment corresponds to
            let stride = 4u64.pow(height.into());
            // The position of the zeroth child of this tier
            let start_position = stride * self.index;
            // The position after the last child of this tier
            let end_position_non_inclusive = (start_position + stride).min(4u64.pow(24) - 1);
            // Whether the node is on the frontier
            let frontier =
                (start_position..end_position_non_inclusive).contains(&self.global_position);

            self.inner = if frontier {
                Some(Inner::Frontier(<Nested<Item>>::build(
                    self.global_position,
                    self.index,
                )))
            } else {
                Some(Inner::Complete(<Nested<Item> as Focus>::Complete::build(
                    self.global_position,
                    self.index,
                )))
            };

            // Now that we've set up the inside, use the instruction to proceed
            self.go(instruction)
        }
    }

    fn is_started(&self) -> bool {
        self.inner.is_some()
    }

    fn index(&self) -> u64 {
        if let Some(inner) = &self.inner {
            match inner {
                Inner::Frontier(frontier) => frontier.index(),
                Inner::Complete(complete) => complete.index(),
            }
        } else {
            self.index
        }
    }

    fn height(&self) -> u8 {
        if let Some(inner) = &self.inner {
            match inner {
                Inner::Frontier(frontier) => frontier.height(),
                Inner::Complete(complete) => complete.height(),
            }
        } else {
            <Self::Output as Height>::Height::HEIGHT
        }
    }

    fn min_required(&self) -> usize {
        if let Some(inner) = &self.inner {
            match inner {
                Inner::Frontier(frontier) => frontier.min_required(),
                Inner::Complete(complete) => complete.min_required(),
            }
        } else {
            1
        }
    }
}
