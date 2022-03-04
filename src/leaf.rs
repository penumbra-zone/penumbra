use crate::{Active, Commitment, Complete, GetHash, Hash, Height};

pub struct Leaf<const BASE_HEIGHT: usize> {
    hash: Hash,
    commitment: Commitment,
    witnessed: bool,
}

impl<const BASE_HEIGHT: usize> GetHash for Leaf<BASE_HEIGHT> {
    fn hash(&self) -> Hash {
        self.hash
    }
}

impl<const BASE_HEIGHT: usize> Height for Leaf<BASE_HEIGHT> {
    const HEIGHT: usize = BASE_HEIGHT;
}

impl<const BASE_HEIGHT: usize> Active for Leaf<BASE_HEIGHT> {
    type Item = Commitment;
    type Complete = Leaf<BASE_HEIGHT>;

    #[inline]
    fn singleton(item: Self::Item) -> Self {
        let hash = Hash::leaf(Self::HEIGHT, &item);
        Self {
            hash,
            commitment: item,
            witnessed: false,
        }
    }

    #[inline]
    fn witness(&mut self) {
        self.witnessed = true;
    }

    #[inline]
    fn insert(self, item: Self::Item) -> Result<Self, (Self::Item, Self::Complete)> {
        Err((item, self))
    }
}

impl<const BASE_HEIGHT: usize> Complete for Leaf<BASE_HEIGHT> {
    type Active = Leaf<BASE_HEIGHT>;

    fn witnessed(&self) -> bool {
        self.witnessed
    }
}
