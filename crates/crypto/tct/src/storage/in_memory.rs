//! An in-memory storage backend, useful for testing.

use super::*;

/// An in-memory storage backend, useful for testing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct InMemory {
    sparse: bool,
    position: StoredPosition,
    forgotten: Forgotten,
    hashes: BTreeMap<Position, BTreeMap<u8, Hash>>,
    commitments: BTreeMap<Position, StateCommitment>,
}

impl InMemory {
    /// Create a new in-memory storage backend.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new in-memory storage backend that only stores essential hashes.
    pub fn new_sparse() -> Self {
        let mut new = Self::new();
        new.sparse = true;
        new
    }

    /// Get the position of the stored tree.
    pub fn position(&self) -> StoredPosition {
        self.position
    }

    /// Get the forgotten version of the stored tree.
    pub fn forgotten(&self) -> Forgotten {
        self.forgotten
    }

    /// Get an iterator of all the hashes stored.
    pub fn hashes(&self) -> impl Iterator<Item = (Position, u8, Hash)> + '_ {
        self.hashes.iter().flat_map(|(position, hashes)| {
            hashes
                .iter()
                .map(move |(height, hash)| (*position, *height, *hash))
        })
    }

    /// Get an iterator of all the commitments stored.
    pub fn commitments(&self) -> impl Iterator<Item = (Position, StateCommitment)> + '_ {
        self.commitments
            .iter()
            .map(|(position, commitment)| (*position, *commitment))
    }
}

/// An error which can occur when using the in-memory storage backend.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Error {
    /// A write was attempted over an existing commitment.
    #[error("refusing to overwrite existing commitment at position {position:?}")]
    DuplicateWriteCommitment {
        /// The position of the existing commitment.
        position: Position,
    },
    /// An unnecessary write was performed.
    #[error("repeated write of hash at position {position:?}, height {height}")]
    RepeatedWriteHash {
        /// The position of the hash.
        position: Position,
        /// The height of the hash.
        height: u8,
    },
    /// A hash was overwritten with a different hash.
    #[error("hash overwritten with different hash at position {position:?}, height {height}")]
    OverwrittenHash {
        /// The position of the hash.
        position: Position,
        /// The height of the hash.
        height: u8,
    },
    /// A hash was marked essential, but it still had children.
    #[error("recalculable hash marked essential at position {position:?}, height {height}")]
    EssentialHashHasChildren {
        /// The position of the hash.
        position: Position,
        /// The height of the hash.
        height: u8,
    },
    /// The position was set, but it did not increase.
    #[error("set position did not increase from {previous:?} to {new:?}")]
    PositionDidNotIncrease {
        /// The previous position.
        previous: StoredPosition,
        /// The new position.
        new: StoredPosition,
    },
    /// The forgotten version was set, but it did not increase.
    #[error("set forgotten version did not increase from {previous:?} to {new:?}")]
    ForgottenDidNotIncrease {
        /// The previous forgotten version.
        previous: Forgotten,
        /// The new forgotten version.
        new: Forgotten,
    },
}

impl Read for InMemory {
    type Error = Error;

    type HashesIter<'a> =
        Box<dyn Iterator<Item = Result<(Position, u8, Hash), Self::Error>> + Send + 'a>;
    type CommitmentsIter<'a> =
        Box<dyn Iterator<Item = Result<(Position, StateCommitment), Self::Error>> + Send + 'a>;

    fn position(&mut self) -> Result<StoredPosition, Self::Error> {
        Ok(self.position)
    }

    fn forgotten(&mut self) -> Result<Forgotten, Self::Error> {
        Ok(self.forgotten)
    }

    fn hash(&mut self, position: Position, height: u8) -> Result<Option<Hash>, Self::Error> {
        Ok(self
            .hashes
            .get(&position)
            .and_then(|h| h.get(&height))
            .cloned())
    }

    fn hashes(&mut self) -> Self::HashesIter<'_> {
        Box::new(InMemory::hashes(self).map(Ok))
    }

    fn commitment(&mut self, position: Position) -> Result<Option<StateCommitment>, Self::Error> {
        Ok(self.commitments.get(&position).cloned())
    }

    fn commitments(&mut self) -> Self::CommitmentsIter<'_> {
        Box::new(InMemory::commitments(self).map(Ok))
    }
}

impl Write for InMemory {
    fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
        essential: bool,
    ) -> Result<(), Self::Error> {
        if !essential && self.sparse {
            // If running in sparse mode, non-essential hashes are not persisted
            return Ok(());
        }

        let column = self.hashes.entry(position).or_default();
        // Only insert if nothing is already there
        match column.entry(height) {
            Entry::Vacant(e) => {
                e.insert(hash);
            }
            Entry::Occupied(e) => {
                if !essential {
                    return Err(Error::RepeatedWriteHash { position, height });
                }
                if *e.into_mut() != hash {
                    return Err(Error::OverwrittenHash { position, height });
                }
            }
        };
        Ok(())
    }

    fn add_commitment(
        &mut self,
        position: Position,
        commitment: StateCommitment,
    ) -> Result<(), Self::Error> {
        // Only insert if nothing is already there
        match self.commitments.entry(position) {
            Entry::Vacant(e) => e.insert(commitment),
            Entry::Occupied(_) => return Err(Error::DuplicateWriteCommitment { position }),
        };
        Ok(())
    }

    fn delete_range(
        &mut self,
        below_height: u8,
        range: Range<Position>,
    ) -> Result<(), Self::Error> {
        // Remove all the inner hashes below and in range
        let empty_columns: Vec<Position> = self
            .hashes
            .range_mut(range.clone())
            .filter_map(|(&position, column)| {
                *column = column.split_off(&below_height);
                if column.is_empty() {
                    Some(position)
                } else {
                    None
                }
            })
            .collect();

        // Remove all the now-empty columns
        for position in empty_columns {
            self.hashes.remove(&position);
        }

        // Find the positions of the commitments within the range
        let commitments_to_delete: Vec<Position> = self
            .commitments
            .range(range)
            .map(|(&position, _)| position)
            .collect();

        // Remove all the commitments within the range
        for position in commitments_to_delete {
            self.commitments.remove(&position);
        }

        Ok(())
    }

    fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error> {
        if self.position >= position {
            return Err(Error::PositionDidNotIncrease {
                previous: self.position,
                new: position,
            });
        }
        self.position = position;
        Ok(())
    }

    fn set_forgotten(&mut self, forgotten: Forgotten) -> Result<(), Self::Error> {
        if self.forgotten >= forgotten {
            return Err(Error::ForgottenDidNotIncrease {
                previous: self.forgotten,
                new: forgotten,
            });
        }
        self.forgotten = forgotten;
        Ok(())
    }
}
