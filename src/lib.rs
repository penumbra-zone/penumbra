pub enum Sparse<T, H> {
    Pruned { hash: H },
    Kept { hash: H, value: T },
}

impl<T, H> Sparse<T, H> {
    pub fn hash(&self) -> &H {
        match self {
            Sparse::Pruned { hash } => hash,
            Sparse::Kept { hash, .. } => hash,
        }
    }
}

pub enum Tree<T, H, const ARITY: usize> {
    Leaf {
        value: T,
    },
    Node {
        children: Box<[Sparse<Tree<T, H, ARITY>, H>; ARITY]>,
    },
}

pub const NCT_ARITY: usize = 4;

pub struct Hash;

pub type Nct<T> = Sparse<Tree<T, Hash, NCT_ARITY>, Hash>;
