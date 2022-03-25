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

impl From<crate::eternity::InsertError> for InsertError {
    fn from(e: crate::eternity::InsertError) -> Self {
        match e {
            crate::eternity::InsertError::Full => InsertError::EternityFull,
            crate::eternity::InsertError::EpochFull => InsertError::EpochFull,
            crate::eternity::InsertError::EpochForgotten => InsertError::EpochForgotten,
            crate::eternity::InsertError::BlockFull => InsertError::BlockFull,
            crate::eternity::InsertError::BlockForgotten => InsertError::BlockForgotten,
        }
    }
}

impl From<crate::eternity::InsertBlockError> for InsertError {
    fn from(e: crate::eternity::InsertBlockError) -> Self {
        match e {
            crate::eternity::InsertBlockError::Full(_) => InsertError::EternityFull,
            crate::eternity::InsertBlockError::EpochFull(_) => InsertError::EpochFull,
            crate::eternity::InsertBlockError::EpochForgotten(_) => InsertError::EpochForgotten,
        }
    }
}

impl From<crate::eternity::InsertBlockRootError> for InsertError {
    fn from(e: crate::eternity::InsertBlockRootError) -> Self {
        match e {
            crate::eternity::InsertBlockRootError::Full => InsertError::EternityFull,
            crate::eternity::InsertBlockRootError::EpochFull => InsertError::EpochFull,
            crate::eternity::InsertBlockRootError::EpochForgotten => InsertError::EpochForgotten,
        }
    }
}

impl From<crate::eternity::InsertEpochError> for InsertError {
    fn from(e: crate::eternity::InsertEpochError) -> Self {
        match e {
            crate::eternity::InsertEpochError(_) => InsertError::EternityFull,
        }
    }
}

impl From<crate::eternity::InsertEpochRootError> for InsertError {
    fn from(e: crate::eternity::InsertEpochRootError) -> Self {
        match e {
            crate::eternity::InsertEpochRootError => InsertError::EternityFull,
        }
    }
}

impl From<crate::epoch::InsertError> for InsertError {
    fn from(e: crate::epoch::InsertError) -> Self {
        match e {
            crate::epoch::InsertError::Full => InsertError::EpochFull,
            crate::epoch::InsertError::BlockFull => InsertError::BlockFull,
            crate::epoch::InsertError::BlockForgotten => InsertError::BlockForgotten,
        }
    }
}

impl From<crate::epoch::InsertBlockError> for InsertError {
    fn from(e: crate::epoch::InsertBlockError) -> Self {
        match e {
            crate::epoch::InsertBlockError(_) => InsertError::EpochFull,
        }
    }
}

impl From<crate::epoch::InsertBlockRootError> for InsertError {
    fn from(e: crate::epoch::InsertBlockRootError) -> Self {
        match e {
            crate::epoch::InsertBlockRootError => InsertError::EpochFull,
        }
    }
}

impl From<crate::block::InsertError> for InsertError {
    fn from(e: crate::block::InsertError) -> Self {
        match e {
            crate::block::InsertError => InsertError::BlockFull,
        }
    }
}
