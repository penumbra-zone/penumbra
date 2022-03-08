use crate::{GetHash, Hash, Height};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    type Height = T::Height;
}

impl<T: crate::Complete> crate::Complete for Complete<T> {
    type Active = super::Active<<T as crate::Complete>::Active>;
}
