#![allow(clippy::unusual_byte_groupings)]

//! Non-incremental deserialization for the [`Tree`](crate::Tree).

use archery::SharedPointerKind;
use futures::StreamExt;

use crate::prelude::*;
use crate::storage::Read;

use super::StoredPosition;

/// Deserialize a [`Tree`] from a storage backend.
pub async fn from_reader<R: Read, RefKind: SharedPointerKind>(
    reader: &mut R,
) -> Result<Tree<RefKind>, R::Error> {
    // Make an uninitialized tree with the correct position and forgotten version
    let position = match reader.position().await? {
        StoredPosition::Position(position) => Some(position.into()),
        StoredPosition::Full => None,
    };
    let forgotten = reader.forgotten().await?;
    let mut inner: frontier::Top<
        frontier::Tier<frontier::Tier<frontier::Item, RefKind>, RefKind>,
        RefKind,
    > = OutOfOrder::uninitialized(position, forgotten);

    // Make an index to track the commitments (we'll assemble this into the final tree)
    let mut index = HashedMap::default();

    // Insert all the commitments into the tree, simultaneously building the index
    let mut commitments = reader.commitments();
    while let Some((position, commitment)) = commitments.next().await.transpose()? {
        inner.uninitialized_out_of_order_insert_commitment(position.into(), commitment);
        index.insert_mut(commitment, u64::from(position).into());
    }

    drop(commitments); // explicit drop to satisfy borrow checker

    // Set all the hashes in the tree
    let mut hashes = reader.hashes();
    while let Some((position, height, hash)) = hashes.next().await.transpose()? {
        inner.unchecked_set_hash(position.into(), height, hash);
    }

    // Finalize the tree by recomputing all missing hashes
    inner.finish_initialize();

    Ok(Tree::unchecked_from_parts(index, inner))
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
