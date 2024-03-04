use std::ops::Range;

use anyhow::Context as _;
use genawaiter::{rc::gen, yield_};
use r2d2_sqlite::rusqlite::Transaction;

use core::fmt::Debug;
use penumbra_tct::{
    storage::{Read, StoredPosition, Write},
    structure::Hash,
    Forgotten, Position, StateCommitment,
};

#[derive(Debug)]
pub struct TreeStore<'a, 'c: 'a>(pub &'a mut Transaction<'c>);

impl Read for TreeStore<'_, '_> {
    type Error = anyhow::Error;

    type HashesIter<'a> = Box<dyn Iterator<Item = Result<(Position, u8, Hash), Self::Error>> + 'a>
    where
        Self: 'a;

    type CommitmentsIter<'a> = Box<dyn Iterator<Item = Result<(Position, StateCommitment), Self::Error>>
        + 'a>
    where
        Self: 'a;

    fn position(&mut self) -> Result<StoredPosition, Self::Error> {
        let mut stmt = self
            .0
            .prepare_cached("SELECT position FROM sct_position LIMIT 1")
            .context("failed to prepare position query")?;
        let position = stmt
            .query_row::<Option<u64>, _, _>([], |row| row.get("position"))
            .context("failed to query position")?
            .map(Position::from)
            .into();
        Ok(position)
    }

    fn forgotten(&mut self) -> Result<Forgotten, Self::Error> {
        let mut stmt = self
            .0
            .prepare_cached("SELECT forgotten FROM sct_forgotten LIMIT 1")
            .context("failed to prepare forgotten query")?;
        let forgotten = stmt
            .query_row::<u64, _, _>([], |row| row.get("forgotten"))
            .context("failed to query forgotten")?
            .into();
        Ok(forgotten)
    }

    fn hash(&mut self, position: Position, height: u8) -> Result<Option<Hash>, Self::Error> {
        let position = u64::from(position) as i64;

        let mut stmt = self
            .0
            .prepare_cached(
                "SELECT hash FROM sct_hashes WHERE position = ?1 AND height = ?2 LIMIT 1",
            )
            .context("failed to prepare hash query")?;
        let bytes = stmt
            .query_row::<Option<Vec<u8>>, _, _>((&position, &height), |row| row.get("hash"))
            .context("failed to query hash")?;

        bytes
            .map(|bytes| {
                <[u8; 32]>::try_from(bytes)
                    .map_err(|_| anyhow::anyhow!("hash was of incorrect length"))
                    .and_then(|array| {
                        if let Ok(hash) = Hash::from_bytes(array) {
                            Ok(hash)
                        } else {
                            Err(anyhow::anyhow!("Failed to create Hash from bytes"))
                        }
                    })
            })
            .transpose()
    }

    fn hashes(&mut self) -> Self::HashesIter<'_> {
        // The iterator has to *own* the stmt because the rows borrow from it, so we use the
        // `genawaiter` crate to shove the entire preparation of the iterator into an (implicit)
        // async block, which handles the desuguaring to properly own the stmt for us.
        Box::new(
            gen!({
                let mut stmt = match self
                    .0
                    .prepare_cached("SELECT position, height, hash FROM sct_hashes")
                    .context("failed to prepare hashes query")
                {
                    Ok(stmt) => stmt,
                    // If an error happens while preparing the statement, shove it inside the first returned
                    // item of the iterator, because we can't return an outer error:
                    Err(e) => {
                        yield_!(Err(e));
                        return;
                    }
                };

                let rows = match stmt
                    .query_and_then([], |row| {
                        let position: i64 = row.get("position")?;
                        let height: u8 = row.get("height")?;
                        let hash: Vec<u8> = row.get("hash")?;
                        let hash = <[u8; 32]>::try_from(hash)
                            .map_err(|_| anyhow::anyhow!("hash was of incorrect length"))
                            .and_then(|array| {
                                Hash::from_bytes(array).map_err(|e| {
                                    // Explicitly convert any error to anyhow::Error
                                    anyhow::Error::msg(format!("Error converting hash: {}", e))
                                })
                            })?;
                        anyhow::Ok((Position::from(position as u64), height, hash))
                    })
                    .context("couldn't query database")
                {
                    Ok(rows) => rows,
                    // If an error happens while querying the database, shove it inside the first
                    // returned item of the iterator, because we can't return an outer error:
                    Err(e) => {
                        yield_!(Err(e));
                        return;
                    }
                };

                // Actually iterate through the rows:
                for row in rows {
                    yield_!(row);
                }
            })
            .into_iter(),
        )
    }

    fn commitment(&mut self, position: Position) -> Result<Option<StateCommitment>, Self::Error> {
        let position = u64::from(position) as i64;

        let mut stmt = self
            .0
            .prepare_cached("SELECT commitment FROM sct_commitments WHERE position = ?1 LIMIT 1")
            .context("failed to prepare commitment query")?;

        let bytes = stmt
            .query_row::<Option<Vec<u8>>, _, _>((&position,), |row| row.get("commitment"))
            .context("failed to query commitment")?;

        bytes
            .map(|bytes| {
                <[u8; 32]>::try_from(bytes)
                    .map_err(|_| anyhow::anyhow!("commitment was of incorrect length"))
                    .and_then(|array| StateCommitment::try_from(array).map_err(Into::into))
            })
            .transpose()
    }

    fn commitments(&mut self) -> Self::CommitmentsIter<'_> {
        // The iterator has to *own* the stmt because the rows borrow from it, so we use the
        // `genawaiter` crate to shove the entire preparation of the iterator into an (implicit)
        // async block, which handles the desuguaring to properly own the stmt for us.
        Box::new(
            gen!({
                let mut stmt = match self
                    .0
                    .prepare_cached("SELECT position, commitment FROM sct_commitments")
                    .context("failed to prepare commitments query")
                {
                    Ok(stmt) => stmt,
                    // If an error happens while preparing the statement, shove it inside the first returned
                    // item of the iterator, because we can't return an outer error:
                    Err(e) => {
                        yield_!(Err(e));
                        return;
                    }
                };

                let rows = match stmt
                    .query_and_then([], |row| {
                        let position: i64 = row.get("position")?;
                        let commitment: Vec<u8> = row.get("commitment")?;
                        let commitment = <[u8; 32]>::try_from(commitment)
                            .map_err(|_| anyhow::anyhow!("commitment was of incorrect length"))
                            .and_then(|array| {
                                StateCommitment::try_from(array).map_err(Into::into)
                            })?;
                        anyhow::Ok((Position::from(position as u64), commitment))
                    })
                    .context("couldn't query database")
                {
                    Ok(rows) => rows,
                    // If an error happens while querying the database, shove it inside the first
                    // returned item of the iterator, because we can't return an outer error:
                    Err(e) => {
                        yield_!(Err(e));
                        return;
                    }
                };

                // Actually iterate through the rows:
                for row in rows {
                    yield_!(row);
                }
            })
            .into_iter(),
        )
    }
}

impl Write for TreeStore<'_, '_> {
    fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error> {
        let position = Option::from(position).map(|p: Position| u64::from(p) as i64);

        self.0
            .prepare_cached("UPDATE sct_position SET position = ?1")
            .context("failed to prepare position update")?
            .execute([&position])?;

        Ok(())
    }

    fn set_forgotten(&mut self, forgotten: Forgotten) -> Result<(), Self::Error> {
        let forgotten = u64::from(forgotten) as i64;

        self.0
            .prepare_cached("UPDATE sct_forgotten SET forgotten = ?1")
            .context("failed to prepare forgotten update")?
            .execute([&forgotten])?;

        Ok(())
    }

    fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
        _essential: bool,
    ) -> Result<(), Self::Error> {
        let position = u64::from(position) as i64;
        let hash = hash.to_bytes().to_vec();

        self.0.prepare_cached(
            "INSERT INTO sct_hashes (position, height, hash) VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING"
        ).context("failed to prepare hash insert")?
            .execute((&position, &height, &hash))
            .context("failed to insert hash")?;

        Ok(())
    }

    fn add_commitment(
        &mut self,
        position: Position,
        commitment: StateCommitment,
    ) -> Result<(), Self::Error> {
        let position = u64::from(position) as i64;
        let commitment = <[u8; 32]>::from(commitment).to_vec();

        self.0.prepare_cached(
            "INSERT INTO sct_commitments (position, commitment) VALUES (?1, ?2) ON CONFLICT DO NOTHING"
        ).context("failed to prepare commitment insert")?
            .execute((&position, &commitment))
            .context("failed to insert commitment")?;

        Ok(())
    }

    fn delete_range(
        &mut self,
        below_height: u8,
        positions: Range<Position>,
    ) -> Result<(), Self::Error> {
        let start = u64::from(positions.start) as i64;
        let end = u64::from(positions.end) as i64;

        self.0
            .prepare_cached(
                "DELETE FROM sct_hashes WHERE position >= ?1 AND position < ?2 AND height < ?3",
            )
            .context("failed to prepare hash delete")?
            .execute((&start, &end, &below_height))
            .context("failed to delete hashes")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use penumbra_tct::{StateCommitment, Witness};

    #[test]
    fn tree_store_spot_check() {
        // Set up the database:
        let mut db = r2d2_sqlite::rusqlite::Connection::open_in_memory().unwrap();
        let mut tx = db.transaction().unwrap();
        tx.execute_batch(include_str!("schema.sql")).unwrap();

        // Now we're exclusively going to talk to the db through the TreeStore:
        let mut store = TreeStore(&mut tx);

        // Check that the currently stored tree is the empty tree:
        let deserialized = penumbra_tct::Tree::from_reader(&mut store).unwrap();
        assert_eq!(deserialized, penumbra_tct::Tree::new());

        // Make some kind of tree:
        let mut tree = penumbra_tct::Tree::new();
        tree.insert(Witness::Keep, StateCommitment::try_from([0; 32]).unwrap())
            .unwrap();
        tree.end_block().unwrap();
        tree.insert(Witness::Forget, StateCommitment::try_from([1; 32]).unwrap())
            .unwrap();
        tree.end_epoch().unwrap();
        tree.insert(Witness::Keep, StateCommitment::try_from([2; 32]).unwrap())
            .unwrap();

        // Write the tree to the database:
        tree.to_writer(&mut store).unwrap();

        // Read the tree back from the database:
        let deserialized = penumbra_tct::Tree::from_reader(&mut store).unwrap();

        assert_eq!(tree, deserialized);
    }
}
