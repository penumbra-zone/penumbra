/// An error when inserting into any builder.
///
/// Not all of these can be thrown by every builder. Unlike the very specific error types used in
/// the main crate, this is a blanket summary of every possible insertion error, for simplicity of
/// specification and testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertError {
    /// The eternity is full.
    EternityFull,
    /// The current epoch is full.
    EpochFull,
    /// The epoch was forgotten.
    EpochForgotten,
    /// The current block is full.
    BlockFull,
    /// The current block was forgotten.
    BlockForgotten,
}

impl From<penumbra_tct::error::InsertError> for InsertError {
    fn from(e: penumbra_tct::error::InsertError) -> Self {
        match e {
            penumbra_tct::error::InsertError::Full => InsertError::EternityFull,
            penumbra_tct::error::InsertError::EpochFull => InsertError::EpochFull,
            penumbra_tct::error::InsertError::EpochForgotten => InsertError::EpochForgotten,
            penumbra_tct::error::InsertError::BlockFull => InsertError::BlockFull,
            penumbra_tct::error::InsertError::BlockForgotten => InsertError::BlockForgotten,
        }
    }
}

impl From<penumbra_tct::error::InsertBlockError> for InsertError {
    fn from(e: penumbra_tct::error::InsertBlockError) -> Self {
        match e {
            penumbra_tct::error::InsertBlockError::Full(_) => InsertError::EternityFull,
            penumbra_tct::error::InsertBlockError::EpochFull(_) => InsertError::EpochFull,
            penumbra_tct::error::InsertBlockError::EpochForgotten(_) => InsertError::EpochForgotten,
        }
    }
}

impl From<penumbra_tct::error::InsertBlockRootError> for InsertError {
    fn from(e: penumbra_tct::error::InsertBlockRootError) -> Self {
        match e {
            penumbra_tct::error::InsertBlockRootError::Full => InsertError::EternityFull,
            penumbra_tct::error::InsertBlockRootError::EpochFull => InsertError::EpochFull,
            penumbra_tct::error::InsertBlockRootError::EpochForgotten => {
                InsertError::EpochForgotten
            }
        }
    }
}

impl From<penumbra_tct::error::InsertEpochError> for InsertError {
    fn from(e: penumbra_tct::error::InsertEpochError) -> Self {
        match e {
            penumbra_tct::error::InsertEpochError(_) => InsertError::EternityFull,
        }
    }
}

impl From<penumbra_tct::error::InsertEpochRootError> for InsertError {
    fn from(e: penumbra_tct::error::InsertEpochRootError) -> Self {
        match e {
            penumbra_tct::error::InsertEpochRootError => InsertError::EternityFull,
        }
    }
}

impl From<penumbra_tct::epoch::InsertError> for InsertError {
    fn from(e: penumbra_tct::epoch::InsertError) -> Self {
        match e {
            penumbra_tct::epoch::InsertError::Full => InsertError::EpochFull,
            penumbra_tct::epoch::InsertError::BlockFull => InsertError::BlockFull,
            penumbra_tct::epoch::InsertError::BlockForgotten => InsertError::BlockForgotten,
        }
    }
}

impl From<penumbra_tct::epoch::InsertBlockError> for InsertError {
    fn from(e: penumbra_tct::epoch::InsertBlockError) -> Self {
        match e {
            penumbra_tct::epoch::InsertBlockError(_) => InsertError::EpochFull,
        }
    }
}

impl From<penumbra_tct::epoch::InsertBlockRootError> for InsertError {
    fn from(e: penumbra_tct::epoch::InsertBlockRootError) -> Self {
        match e {
            penumbra_tct::epoch::InsertBlockRootError => InsertError::EpochFull,
        }
    }
}

impl From<penumbra_tct::block::InsertError> for InsertError {
    fn from(e: penumbra_tct::block::InsertError) -> Self {
        match e {
            penumbra_tct::block::InsertError => InsertError::BlockFull,
        }
    }
}
