use crate::{GetHash, Hash, Height};

pub struct Complete<T>(T);

impl<T> Complete<T> {
    pub fn new(item: T) -> Self {
        Self(item)
    }
}

impl<T: GetHash> GetHash for Complete<T> {
    fn hash(&self) -> Hash {
        self.0.hash()
    }
}

impl<T: Height> Height for Complete<T> {
    const HEIGHT: usize = T::HEIGHT;
}

impl<T: crate::Complete> crate::Complete for Complete<T> {
    type Active = super::Active<<T as crate::Complete>::Active>;
}
