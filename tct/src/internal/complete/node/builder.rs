use std::collections::VecDeque;

use super::*;

/// A builder for a frontier node.
pub struct Builder<Child: Built> {
    index: u64,
    global_position: u64,
    // This is populated when we first get a `Node` instruction, to the size requested
    remaining: Option<Remaining<Child>>,
    // Then as we complete siblings, we push them onto this
    children: Three<Insert<Child>>,
}

struct Remaining<Child: Built> {
    hash: Option<Hash>,
    children: VecDeque<Child::Builder>,
}

impl<Child: Height + Built> Built for Node<Child> {
    type Builder = Builder<Child>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder {
            global_position,
            index,
            children: Three::new(),
            remaining: None,
        }
    }
}

impl<Child: Height + Built> Build for Builder<Child> {
    type Output = Node<Child>;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, InvalidInstruction<Self>> {
        use {IResult::*, Instruction::*};

        todo!();
    }

    fn is_started(&self) -> bool {
        self.remaining.is_some()
    }

    fn index(&self) -> u64 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first) = remaining.children.front() {
                first.index()
            } else {
                unreachable!("list of under-construction children is never empty")
            }
        } else {
            self.index
        }
    }

    fn height(&self) -> u8 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first) = remaining.children.front() {
                first.height()
            } else {
                unreachable!("list of under-construction children is never empty")
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
                .children
                .iter()
                .map(|sibling| sibling.min_required())
                .sum::<usize>()
        } else {
            1
        }
    }
}
