/// A clue that allows probabilistic message detection.
#[derive(Debug, Clone)]
pub struct Clue(pub [u8; 68]);

impl Clue {
    /// The bits of precision for this `Clue`.
    pub fn precision_bits(&self) -> u8 {
        self.0[64]
    }
}
