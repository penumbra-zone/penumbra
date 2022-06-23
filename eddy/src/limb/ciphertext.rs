// an Elgamal ciphertext (c1, c2).
#[derive(Clone, Copy)]
pub struct Ciphertext {
    pub(crate) c1: decaf377::Element,
    pub(crate) c2: decaf377::Element,
}
