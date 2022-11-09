//! Non-incremental deserialization for the [`Tree`](crate::Tree).

use futures::{Stream, StreamExt};

use crate::prelude::*;

/// Deserialize a [`Tree`] from an asynchronous storage backend.
pub async fn from_async_reader<R: AsyncRead>(reader: &mut R) -> Result<Tree, R::Error> {
    let position = reader.position().await?;
    let forgotten = reader.forgotten().await?;
    let mut load_commitments = LoadCommitments::new(position, forgotten);
    let mut commitments = reader.commitments();
    while let Some((position, commitment)) = commitments.next().await.transpose()? {
        load_commitments.insert(position, commitment);
    }
    drop(commitments);
    let mut hashes = reader.hashes();
    let mut load_hashes = load_commitments.load_hashes();
    while let Some((position, height, hash)) = hashes.next().await.transpose()? {
        load_hashes.insert(position, height, hash);
    }
    Ok(load_hashes.finish())
}

/// Deserialize a [`Tree`] from a synchronous storage backend.
pub fn from_reader<R: Read>(reader: &mut R) -> Result<Tree, R::Error> {
    let position = reader.position()?;
    let forgotten = reader.forgotten()?;
    let mut load_commitments = LoadCommitments::new(position, forgotten);
    let mut commitments = reader.commitments();
    while let Some((position, commitment)) = commitments.next().transpose()? {
        load_commitments.insert(position, commitment);
    }
    drop(commitments);
    let mut load_hashes = load_commitments.load_hashes();
    let mut hashes = reader.hashes();
    while let Some((position, height, hash)) = hashes.next().transpose()? {
        load_hashes.insert(position, height, hash);
    }
    Ok(load_hashes.finish())
}

/// Load iterators of commitments and hashes into a tree.
pub fn load<Err>(
    position: impl Into<StoredPosition>,
    forgotten: Forgotten,
    commitments: impl IntoIterator<Item = Result<(Position, Commitment), Err>>,
    hashes: impl IntoIterator<Item = Result<(Position, u8, Hash), Err>>,
) -> Result<Tree, Err> {
    let mut commitments = commitments.into_iter();
    let mut hashes = hashes.into_iter();

    let mut load_commitments = LoadCommitments::new(position, forgotten);
    while let Some((position, commitment)) = commitments.next().transpose()? {
        load_commitments.insert(position, commitment);
    }
    let mut load_hashes = load_commitments.load_hashes();
    while let Some((position, height, hash)) = hashes.next().transpose()? {
        load_hashes.insert(position, height, hash);
    }
    Ok(load_hashes.finish())
}

/// Load streams of commitments and hashes into a tree.
pub async fn load_stream<Err>(
    position: impl Into<StoredPosition>,
    forgotten: Forgotten,
    mut commitments: impl Stream<Item = Result<(Position, Commitment), Err>> + Unpin,
    mut hashes: impl Stream<Item = Result<(Position, u8, Hash), Err>> + Unpin,
) -> Result<Tree, Err> {
    let mut load_commitments = LoadCommitments::new(position, forgotten);
    while let Some((position, commitment)) = commitments.next().await.transpose()? {
        load_commitments.insert(position, commitment);
    }
    let mut load_hashes = load_commitments.load_hashes();
    while let Some((position, height, hash)) = hashes.next().await.transpose()? {
        load_hashes.insert(position, height, hash);
    }
    Ok(load_hashes.finish())
}

struct LoadCommitments {
    inner: frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>,
    index: HashedMap<Commitment, index::within::Tree>,
}

impl LoadCommitments {
    pub fn new(position: impl Into<StoredPosition>, forgotten: Forgotten) -> Self {
        // Make an uninitialized tree with the correct position and forgotten version
        let position = match position.into() {
            StoredPosition::Position(position) => Some(position.into()),
            StoredPosition::Full => None,
        };
        Self {
            inner: OutOfOrder::uninitialized(position, forgotten),
            index: HashedMap::default(),
        }
    }

    pub fn insert(&mut self, position: Position, commitment: Commitment) {
        self.inner
            .uninitialized_out_of_order_insert_commitment(position.into(), commitment);
        self.index.insert(commitment, u64::from(position).into());
    }

    pub fn load_hashes(self) -> LoadHashes {
        LoadHashes {
            inner: self.inner,
            index: self.index,
        }
    }
}

impl Extend<(Position, Commitment)> for LoadCommitments {
    fn extend<T: IntoIterator<Item = (Position, Commitment)>>(&mut self, iter: T) {
        for (position, commitment) in iter {
            self.insert(position, commitment);
        }
    }
}

pub struct LoadHashes {
    inner: frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>,
    index: HashedMap<Commitment, index::within::Tree>,
}

impl LoadHashes {
    pub fn insert(&mut self, position: Position, height: u8, hash: Hash) {
        self.inner.unchecked_set_hash(position.into(), height, hash);
    }

    pub fn finish(mut self) -> Tree {
        self.inner.finish_initialize();
        Tree::unchecked_from_parts(self.index, self.inner)
    }
}

impl Extend<(Position, u8, Hash)> for LoadHashes {
    fn extend<T: IntoIterator<Item = (Position, u8, Hash)>>(&mut self, iter: T) {
        for (position, height, hash) in iter {
            self.insert(position, height, hash);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::{arbitrary::*, prelude::*};

    proptest::proptest! {
        #[test]
        fn uninitialized_produces_correct_position_and_forgotten(init_position in prop::option::of(any::<Position>()), init_forgotten in any::<Forgotten>()) {
            let tree: frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>> =
                OutOfOrder::uninitialized(init_position.map(Into::into), init_forgotten);
            assert_eq!(init_position, tree.position().map(Into::into));
            assert_eq!(init_forgotten, tree.forgotten().unwrap());
        }
    }
}
