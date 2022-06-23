use std::collections::VecDeque;

use super::*;

/// A builder for a frontier node.
pub struct Builder<Child: Built + Focus>
where
    Child::Complete: Built,
{
    global_position: u64,
    index: u64,
    // This is populated when we first get a `Node` instruction, to the size requested
    remaining: Option<Remaining<Child>>,
    // Then as we complete siblings, we push them onto this
    siblings: Three<Insert<Child::Complete>>,
}

struct Remaining<Child: Built + Focus>
where
    Child::Complete: Built,
{
    hash: Option<Hash>,
    siblings: VecDeque<<Child::Complete as Built>::Builder>,
    focus: Option<<Child as Built>::Builder>,
}

impl<Child: Focus + Built> Built for Node<Child>
where
    Child::Complete: Built,
{
    type Builder = Builder<Child>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder {
            global_position,
            index,
            siblings: Three::new(),
            remaining: None,
        }
    }
}

impl<Child: Focus + Built> Build for Builder<Child>
where
    Child::Complete: Built,
{
    type Output = Node<Child>;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        use {IResult::*, Instruction::*};

        // We have started construction of the children...
        if let Some(ref mut remaining) = self.remaining {
            // If there's a sibling builder remaining, we should direct the instruction to that builder
            if let Some(front_sibling) = remaining.siblings.pop_front() {
                // If the instruction is a `Leaf` instruction and we haven't started construction of
                // the front sibling, we want to skip the construction of the front sibling and just
                // abbreviate it with a hash -- but if we've already started construction, then the
                // instruction should just be forwarded as usual
                let mut skipped = false;
                if !front_sibling.is_started() {
                    if let Leaf { here } = instruction {
                        self.siblings.push_mut(Insert::Hash(Hash::new(here)));
                        skipped = true;
                    }
                };

                // If we didn't skip constructing this sibling, then we should forward the
                // instruction to the front sibling builder
                if !skipped {
                    match front_sibling.go(instruction) {
                        Err(HitBottom(incomplete)) => {
                            // We bounced off the bottom, so restore this sibling builder and error
                            remaining.siblings.push_front(incomplete);
                            return Err(HitBottom(self));
                        }
                        Ok(Incomplete(incomplete)) => {
                            // We haven't finished with this sibling builder, so push it back onto
                            // the front, so we'll pop it off again next instruction
                            remaining.siblings.push_front(incomplete);
                        }
                        Ok(Complete(complete)) => {
                            // We finished building the sibling, so push it into list of siblings
                            self.siblings.push_mut(Insert::Keep(complete));
                        }
                    }
                }

                // No matter what, we're never complete after finishing a sibling, because we still
                // have to construct the focus
                Ok(Incomplete(self))
            } else {
                // If there are no remaining siblings to construct, then construct the focus by
                // forwarding the instruction to it
                match remaining
                    .focus
                    .take()
                    .expect("focus builder is present")
                    .go(instruction)
                {
                    Err(HitBottom(incomplete)) => {
                        // We bounced off the bottom, so restore the focus builder and error
                        remaining.focus = Some(incomplete);
                        Err(HitBottom(self))
                    }
                    Ok(Incomplete(incomplete)) => {
                        // We haven't finished building the focus, so restore it so we'll pop it off
                        // and keep building it next time
                        remaining.focus = Some(incomplete);
                        Ok(Incomplete(self))
                    }
                    // The completion of the builder: map everything into a node
                    Ok(Complete(focus)) => Ok(Complete(self::Node {
                        hash: remaining.hash.map(Into::into).unwrap_or_default(),
                        forgotten: [Forgotten::default(); 4],
                        siblings: self.siblings,
                        focus,
                    })),
                }
            }
        // We have not started construction of the children (this is the first instruction)...
        } else {
            self.remaining = Some(match instruction {
                Leaf { .. } => {
                    unreachable!("leaf instruction is never given as first instruction to a node")
                }
                Node { here, size } => {
                    let hash = here.map(Hash::new);
                    let size: usize = size.into();

                    // Pre-allocate builders for each of the siblings and the focus, giving each the
                    // correct index, so that when we continue with construction, we just need to
                    // pop off the next builder to work with it:
                    let mut index = self.index * 4; // multiply by 4 because we're going one level down

                    let mut siblings = VecDeque::with_capacity(size - 1);
                    while index < self.index + size as u64 {
                        siblings.push_back(Child::Complete::build(self.global_position, index));
                        index += 1;
                    }

                    let focus = Some(Child::build(self.global_position, index));

                    Remaining {
                        hash,
                        siblings,
                        focus,
                    }
                }
            });

            // We're never complete at this point, because at very least we need to build the focus
            Ok(Incomplete(self))
        }
    }

    fn is_started(&self) -> bool {
        self.remaining.is_some()
    }

    fn index(&self) -> u64 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first_sibling) = remaining.siblings.front() {
                first_sibling.index()
            } else {
                remaining
                    .focus
                    .as_ref()
                    .expect("focus builder is present")
                    .index()
            }
        } else {
            self.index
        }
    }

    fn height(&self) -> u8 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first_sibling) = remaining.siblings.front() {
                first_sibling.height()
            } else {
                remaining
                    .focus
                    .as_ref()
                    .expect("focus builder is present")
                    .height()
            }
        } else {
            <Self::Output as Height>::Height::HEIGHT
        }
    }

    fn min_required(&self) -> usize {
        if let Some(ref remaining) = self.remaining {
            // In the case when we have already started construction, we can just sum up the
            // remaining required things for each in-progress builder:
            remaining
                .siblings
                .iter()
                .map(|sibling| sibling.min_required())
                .sum::<usize>()
                + remaining
                    .focus
                    .as_ref()
                    .expect("focus builder is present")
                    .min_required()
        } else {
            // Because we are a frontier node and we haven't yet started construction, the minimum
            // number of instructions we need is the amount necessary to construct the frontier,
            // assuming that nothing is witnessed: we calculate that here by iterating through the
            // global position, adding a number between 1 and 4 to our count depending on the value
            // at that height; we then add 1, because the tip of the frontier must be represented,
            // at least by a hash.
            let mut position = self.global_position;
            let mut min_required = 1; // start at 1 to include the tip
            for _ in 0..self.height() {
                min_required += (position & 0b11) + 1; // 00 => 1, 01 => 2, 10 => 3, 11 => 4
                position >>= 2;
            }
            min_required as usize
        }
    }
}
