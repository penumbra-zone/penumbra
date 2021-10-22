use crate::Fq;

pub const NK_LEN_BYTES: usize = 32;

/// Allows deriving the nullifier associated with a note.
#[derive(Clone, Copy)]
pub struct NullifierKey(pub Fq);
