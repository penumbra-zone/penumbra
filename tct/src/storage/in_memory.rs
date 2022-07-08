//! An in-memory but not necessarily very fast storage backend, useful for testing.

use super::*;

/// An in-memory storage backend, useful for testing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct InMemory {
    position: u64,
    points: BTreeMap<u64, BTreeMap<u8, Fq>>,
}

/// An error which can occur when using the in-memory storage backend.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Error {
    /// A write was attempted over an existing point.
    #[error("refusing to overwrite existing point at position {position}, depth {depth}")]
    DuplicateWrite {
        /// The position of the existing point.
        position: u64,
        /// The depth of the existing point.
        depth: u8,
    },
}

#[async_trait]
impl Read for InMemory {
    type Error = Error;

    async fn position(&mut self) -> Result<u64, Self::Error> {
        Ok(self.position)
    }

    async fn read(&mut self, position: u64, depth: u8) -> Result<Option<Fq>, Self::Error> {
        Ok(self
            .points
            .get(&position)
            .and_then(|column| column.get(&depth).copied()))
    }

    fn points(&mut self) -> Pin<Box<dyn Stream<Item = Result<Point, Self::Error>> + '_>> {
        Box::pin(stream::iter(self.points.iter().flat_map(
            |(&position, column)| {
                column.iter().map(move |(&depth, &here)| {
                    Ok(Point {
                        position,
                        depth,
                        here,
                    })
                })
            },
        )))
    }
}

#[async_trait]
impl Write for InMemory {
    async fn write(&mut self, point: Point) -> Result<(), Self::Error> {
        let column = self.points.entry(point.position).or_default();
        // Only insert if nothing is already there
        match column.entry(point.depth) {
            Entry::Vacant(e) => e.insert(point.here),
            Entry::Occupied(_) => {
                return Err(Error::DuplicateWrite {
                    position: point.position,
                    depth: point.depth,
                })
            }
        };
        Ok(())
    }

    async fn delete_range(
        &mut self,
        minimum_depth: u8,
        positions: Range<u64>,
    ) -> Result<(), Self::Error> {
        // TODO: this could be faster if there was a way to iterate over a range of a `BTreeMap`
        // rather than traversing the entire thing each time
        for (position, column) in self.points.iter_mut() {
            if positions.contains(position) {
                column.retain(|&depth, _| depth < minimum_depth);
            }
        }
        Ok(())
    }

    async fn set_position(&mut self, position: u64) -> Result<(), Self::Error> {
        self.position = position;
        Ok(())
    }
}
