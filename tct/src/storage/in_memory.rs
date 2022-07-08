//! An in-memory but not necessarily very fast storage backend, useful for testing.

use super::*;

/// An in-memory storage backend, useful for testing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct InMemory {
    position: StoredPosition,
    hashes: BTreeMap<Position, BTreeMap<u8, Hash>>,
    commitments: BTreeMap<Position, Commitment>,
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
}

#[async_trait]
impl Read for InMemory {
    type Error = Error;

    async fn position(&mut self) -> Result<StoredPosition, Self::Error> {
        Ok(self.position)
    }

    async fn get_hash(
        &mut self,
        position: Position,
        height: u8,
    ) -> Result<Option<Hash>, Self::Error> {
        Ok(self
            .hashes
            .get(&position)
            .and_then(|column| column.get(&height).copied()))
    }

    async fn get_commitment(
        &mut self,
        position: Position,
    ) -> Result<Option<Commitment>, Self::Error> {
        Ok(self.commitments.get(&position).copied())
    }

    fn hashes(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, u8, Hash), Self::Error>> + '_>> {
        Box::pin(stream::iter(self.hashes.iter().flat_map(
            |(&position, column)| {
                column
                    .iter()
                    .map(move |(&height, &hash)| Ok((position, height, hash)))
            },
        )))
    }

    fn commitments(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, Commitment), Self::Error>> + '_>> {
        Box::pin(stream::iter(
            self.commitments
                .iter()
                .map(|(&position, &commitment)| Ok((position, commitment))),
        ))
    }
}

#[async_trait]
impl Write for InMemory {
    async fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
    ) -> Result<(), Self::Error> {
        let column = self.hashes.entry(position).or_default();
        // Only insert if nothing is already there
        match column.entry(height) {
            Entry::Vacant(e) => {
                println!(
                    "INSERTING INTO STORAGE: position: {position:?}, height: {height}, hash: {hash:?}"
                );
                e.insert(hash);
            }
            Entry::Occupied(_) => { /* do nothing */ }
        };
        Ok(())
    }

    async fn add_commitment(
        &mut self,
        position: Position,
        commitment: Commitment,
    ) -> Result<(), Self::Error> {
        // Only insert if nothing is already there
        match self.commitments.entry(position) {
            Entry::Vacant(e) => e.insert(commitment),
            Entry::Occupied(_) => return Err(Error::DuplicateWriteCommitment { position }),
        };
        Ok(())
    }

    async fn delete_range(
        &mut self,
        below_height: u8,
        range: Range<Position>,
    ) -> Result<(), Self::Error> {
        // TODO: this could be faster if there was a way to iterate over a range of a `BTreeMap`
        // rather than traversing the entire thing each time

        // Remove all the inner hashes below and in range
        for (position, column) in self.hashes.iter_mut() {
            if range.contains(position) {
                column.retain(|&height, _| height >= below_height);
            }
        }

        // Remove all the commitments within the range
        self.commitments
            .retain(|position, _| !range.contains(position));

        Ok(())
    }

    async fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error> {
        self.position = position;
        Ok(())
    }
}
