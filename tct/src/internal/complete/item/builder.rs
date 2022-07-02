use super::*;

use crate::storage::{
    deserialize::{IResult, Unexpected},
    Instruction,
};

/// A builder for a complete item.
pub struct Builder {
    index: u64,
    inner: Inner,
}

enum Inner {
    Init,
    AwaitingCommitment { hash: Option<Hash> },
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
            (Init, Leaf { .. }) => Err(Unexpected::Leaf),
            (Init, Node { here, .. }) => {
                self.inner = AwaitingCommitment {
                    hash: here.map(Hash::new),
                };
                Ok(Incomplete(self))
            }
            (AwaitingCommitment { hash: None }, Leaf { here }) => {
                let commitment = Commitment(here);
                Ok(Complete(Item {
                    hash: Hash::of(commitment),
                    commitment,
                }))
            }
            (AwaitingCommitment { hash: Some(hash) }, Leaf { here }) => Ok(Complete(Item {
                hash: *hash,
                commitment: Commitment(here),
            })),
            (AwaitingCommitment { .. }, Node { .. }) => Err(Unexpected::Node),
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
