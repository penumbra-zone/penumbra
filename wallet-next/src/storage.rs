use penumbra_chain::params::ChainParams;
use penumbra_crypto::merkle::NoteCommitmentTree;
use penumbra_proto::{crypto::FullViewingKey, Message, Protobuf};
use sqlx::{query, Pool, Sqlite};

pub struct Storage {
    pub(super) pool: Pool<Sqlite>,
}

impl Storage {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn migrate(self: &Storage) -> anyhow::Result<()> {
        sqlx::migrate!().run(&self.pool).await.map_err(Into::into)
    }

    /// The last block height we've scanned to, if any.
    pub async fn last_sync_height(&self) -> anyhow::Result<Option<u64>> {
        let result = sqlx::query!(
            r#"
            SELECT height
            FROM sync_height
            ORDER BY height DESC
            LIMIT 1
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result[0].height.map(|h| h as u64))
    }
    pub async fn chain_params(&self) -> anyhow::Result<ChainParams> {
        let result = query!(
            r#"
            SELECT bytes
            FROM chain_params
            LIMIT 1
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        ChainParams::decode(result[0].bytes.as_ref().unwrap().as_slice())
    }
    pub async fn full_viewing_key(&self) -> anyhow::Result<FullViewingKey> {
        let result = query!(
            r#"
            SELECT bytes
            FROM full_viewing_key
            LIMIT 1
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(FullViewingKey::decode(
            result[0].bytes.as_ref().unwrap().as_slice(),
        )?)
    }
    pub async fn note_commitment_tree(&self) -> anyhow::Result<NoteCommitmentTree> {
        let result = query!(
            r#"
            SELECT bytes
            FROM note_commitment_tree
            LIMIT 1
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        //let nct_data = bincode::serialize(&result)?;

        Ok(bincode::deserialize(
            result[0].bytes.as_ref().unwrap().as_slice(),
        )?)
    }
}
