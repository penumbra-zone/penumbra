//! Non-incremental deserialization for the [`Tree`](crate::Tree).

use std::{collections::VecDeque, io};

use ark_ed_on_bls12_377::Fq;
use decaf377::FieldExt;
use futures::StreamExt;

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

/// Builder for loading commitments to create a [`Tree`].
///
/// This does not check for internal consistency: inputs that are not derived from a serialization
/// of the tree can lead to internal invariant violations.
pub struct LoadCommitments {
    inner: frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>,
    index: HashedMap<Commitment, index::within::Tree>,
}

impl LoadCommitments {
    pub(crate) fn new(position: impl Into<StoredPosition>, forgotten: Forgotten) -> Self {
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

    /// Insert a commitment at a given position.
    pub fn insert(&mut self, position: Position, commitment: Commitment) {
        self.inner
            .uninitialized_out_of_order_insert_commitment(position.into(), commitment);
        self.index.insert(commitment, u64::from(position).into());
    }

    /// Start loading the hashes for the inside of the tree.
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

/// Builder for loading hashes to create a [`Tree`].
pub struct LoadHashes {
    inner: frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>,
    index: HashedMap<Commitment, index::within::Tree>,
}

impl LoadHashes {
    /// Insert a hash at a given position and height.
    pub fn insert(&mut self, position: Position, height: u8, hash: Hash) {
        self.inner.unchecked_set_hash(position.into(), height, hash);
    }

    /// Finish loading the tree.
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

pub fn succinct(mut reader: impl io::Read) -> io::Result<crate::Tree> {
    fn six_to_eight(bytes: [u8; 6]) -> [u8; 8] {
        let mut result = [0; 8];
        result[0..6].copy_from_slice(&bytes);
        result
    }

    // If the buffer is empty, return the empty tree; otherwise, read position bytes
    let mut position_bytes = [0; 6];
    let (first_position_byte, rest_position_bytes) = position_bytes.split_at_mut(1);
    match reader.read_exact(first_position_byte) {
        Ok(()) => (),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(crate::Tree::default()),
        Err(e) => return Err(e),
    }
    reader.read_exact(rest_position_bytes)?;

    let stored_position = u64::from_le_bytes(six_to_eight(position_bytes));

    // We represented a full tree as u64::MAX, so we need to convert back
    let global_position = if stored_position == u64::MAX {
        StoredPosition::Full
    } else {
        StoredPosition::Position((stored_position + 1).into())
    };

    // Find out how many commitments there are
    let mut witnessed_count_bytes = [0; 6];
    reader.read_exact(&mut witnessed_count_bytes)?;
    let witnessed_count = usize::from_le_bytes(six_to_eight(witnessed_count_bytes));

    // Load all the commitments with a default forgotten count
    let mut load_commitments = crate::Tree::load(global_position, Forgotten::default());

    // Keep track of all the positions of all the commitments
    let mut commitment_positions = VecDeque::with_capacity(witnessed_count);

    // Load all the commitments
    for _ in 0..witnessed_count {
        let mut position_bytes = [0; 6];
        reader.read_exact(&mut position_bytes)?;

        let position = Position::from(u64::from_le_bytes(six_to_eight(position_bytes)));

        let mut commitment_bytes = [0; 32];
        reader.read_exact(&mut commitment_bytes)?;

        let commitment = Commitment(
            Fq::from_bytes(commitment_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );

        // Insert the commitment into the tree
        load_commitments.insert(position, commitment);

        // Recall this position
        commitment_positions.push_back(position);
    }

    // Now switch to loading hashes
    let mut load_hashes = load_commitments.load_hashes();

    // We infer the position by tracking what it must be as we follow the fringe
    let mut position = Position::default();

    // The position of the tip of the frontier
    let frontier_tip = Option::from(global_position)
        .map(|p: Position| Position::from(u64::from(p) - 1))
        .unwrap_or(Position::MAX);

    while position <= frontier_tip {
        // Find the next position on the fringe
        if let Some(next_commitment_position) = commitment_positions.front() {
            if *next_commitment_position == position {
                // We're at the point of a commitment, so we skip it
                commitment_positions.pop_front();
                position = Position::from(u64::from(position) + 1);
                continue;
            }
        }

        let mut hash_bytes = [0; 32];
        let (first_hash_byte, rest_hash_bytes) = hash_bytes.split_at_mut(1);

        // Based on the first byte we read, determine if it's an abbreviated 1-hash, or some other
        // hash, and only read more if it's not an abbreviated 1-hash
        // reader.read_exact(first_hash_byte)?;
        // let hash = if first_hash_byte[0] == u8::MAX {
        //     Hash::one()
        // } else {
        //     reader.read_exact(rest_hash_bytes)?;
        //     hash_bytes.reverse(); // hash is stored big-endian but needs to be little-endian

        // Create the hash
        reader.read_exact(&mut hash_bytes)?;
        let hash = Hash::from_bytes(hash_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        // };

        // Determine the height by inferring what it must be based on the position of the next
        // commitment: it's one below the height of their lowest common ancestor
        let next_commitment_position = commitment_positions
            .front()
            .copied()
            .unwrap_or(frontier_tip);

        let height = {
            // Least-common-ancestor calculation: shift right `here` and `next` until they are
            // equal, going 2 bits at a time
            let mut lca: u8 = 0;
            let mut here = u64::from(position);
            let mut next = u64::from(next_commitment_position);
            while here != next {
                lca += 1;
                // Shift right by 2 because paths are made of 2-bit segments
                here >>= 2;
                next >>= 2;
            }
            lca.saturating_sub(1)
        };

        // Load the hash
        load_hashes.insert(position, height, hash);

        // Advance the position based on the stride at this height
        position = Position::from(u64::from(position) + 4u64.pow(height.into()));
    }

    Ok(load_hashes.finish())
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

    #[test]
    fn succinct_roundtrip() {
        use crate::Witness::*;

        let mut tree = Tree::new();
        for (i, witness) in (0..)
            .zip([Keep, Forget, Forget].into_iter().cycle())
            .take(100)
        {
            let mut serialized = Vec::new();
            crate::storage::serialize::succinct(&mut serialized, &tree).unwrap();
            let deserialized =
                crate::storage::deserialize::succinct(io::Cursor::new(&serialized)).unwrap();
            assert_eq!(tree, deserialized);

            tree.insert(witness, Commitment(i.into())).unwrap();
        }
    }
}
