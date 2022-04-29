/// An error when inserting into any builder.
///
/// Not all of these can be thrown by every builder. Unlike the very specific error types used in
/// the main crate, this is a blanket summary of every possible insertion error, for simplicity of
/// specification and testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertError {
    /// The eternity is full.
    Full,
    /// The current epoch is full.
    EpochFull,
    /// The current block is full.
    BlockFull,
}

impl From<penumbra_tct::InsertError> for InsertError {
    fn from(e: penumbra_tct::InsertError) -> Self {
        match e {
            penumbra_tct::InsertError::Full => InsertError::Full,
            penumbra_tct::InsertError::EpochFull => InsertError::EpochFull,
            penumbra_tct::InsertError::BlockFull => InsertError::BlockFull,
        }
    }
}

impl From<penumbra_tct::InsertBlockError> for InsertError {
    fn from(e: penumbra_tct::InsertBlockError) -> Self {
        match e {
            penumbra_tct::InsertBlockError::Full(_) => InsertError::Full,
            penumbra_tct::InsertBlockError::EpochFull(_) => InsertError::EpochFull,
        }
    }
}

impl From<penumbra_tct::InsertEpochError> for InsertError {
    fn from(e: penumbra_tct::InsertEpochError) -> Self {
        match e {
            penumbra_tct::InsertEpochError(_) => InsertError::Full,
        }
    }
}
