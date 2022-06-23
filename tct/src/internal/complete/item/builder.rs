use super::*;

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

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        use {IResult::*, Inner::*, Instruction::*};

        match (&self.inner, instruction) {
            (Init, Leaf { here }) => Ok(Complete(Hash::new(here).into())),
            (Init, Node { here, .. }) => {
                self.inner = AwaitingCommitment {
                    hash: here.map(Hash::new),
                };
                Ok(Incomplete(self))
            }
            (AwaitingCommitment { hash: None }, Leaf { here }) => {
                Ok(Complete(Commitment(here).into()))
            }
            (AwaitingCommitment { hash: Some(hash) }, Leaf { here }) => Ok(Complete(Item {
                hash,
                commitment: Commitment(here),
            })),
            (AwaitingCommitment { .. }, Node { .. }) => Err(HitBottom(self)),
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
