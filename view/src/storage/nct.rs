use async_trait::async_trait;
use futures::Stream;
use std::{ops::Range, pin::Pin};

use penumbra_tct::{
    storage::{Read, StoredPosition, Write},
    structure::Hash,
    Commitment, Forgotten, Position,
};

pub struct TreeStore<'a, 'c: 'a>(pub &'a mut sqlx::Transaction<'c, sqlx::Sqlite>);

#[async_trait]
impl Read for TreeStore<'_, '_> {
    type Error = sqlx::Error;

    async fn position(&mut self) -> Result<StoredPosition, Self::Error> {
        todo!()
    }

    async fn forgotten(&mut self) -> Result<Forgotten, Self::Error> {
        todo!()
    }

    fn hashes(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, u8, Hash), Self::Error>> + Send + '_>> {
        todo!()
    }

    fn commitments(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<(Position, Commitment), Self::Error>> + Send + '_>> {
        todo!()
    }
}

#[async_trait]
impl Write for TreeStore<'_, '_> {
    async fn add_hash(
        &mut self,
        position: Position,
        height: u8,
        hash: Hash,
        essential: bool,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn add_commitment(
        &mut self,
        position: Position,
        commitment: Commitment,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn delete_range(
        &mut self,
        below_height: u8,
        positions: Range<Position>,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn set_position(&mut self, position: StoredPosition) -> Result<(), Self::Error> {
        todo!()
    }

    async fn set_forgotten(&mut self, forgotten: Forgotten) -> Result<(), Self::Error> {
        todo!()
    }
}
