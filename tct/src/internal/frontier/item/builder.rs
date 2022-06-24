use super::*;

use build::{Build, Built, IResult, Instruction, Unexpected};

/// A builder for the tip of the frontier.
pub struct Builder {
    index: u64,
    inner: Inner,
}

enum Inner {
    Init,
    Witnessed { hash: Option<Hash> },
}

impl Built for Item {
    type Builder = Builder;

    fn build(_global_position: u64, index: u64) -> Self::Builder {
        Builder {
            index,
            inner: Inner::Init,
        }
    }
}

impl Build for Builder {
    type Output = Item;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, Unexpected> {
        use {IResult::*, Inner::*, Instruction::*};

        match (&self.inner, instruction) {
            (Init, Leaf { here }) => Ok(Complete(Hash::new(here).into())),
            (Init, Node { here, .. }) => {
                self.inner = Witnessed {
                    hash: here.map(Hash::new),
                };
                Ok(Incomplete(self))
            }
            (Witnessed { hash: None }, Leaf { here }) => Ok(Complete(Commitment(here).into())),
            (Witnessed { hash: Some(hash) }, Leaf { here }) => Ok(Complete(Item {
                item: Insert::Keep((Commitment(here), *hash)),
            })),
            (Witnessed { .. }, Node { .. }) => Err(Unexpected::Node),
        }
    }

    fn is_started(&self) -> bool {
        !matches!(self.inner, Inner::Init)
    }

    fn index(&self) -> u64 {
        self.index
    }

    fn height(&self) -> u8 {
        0
    }

    fn min_required(&self) -> usize {
        1
    }
}
