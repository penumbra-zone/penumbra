use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::{query, query_as, Pool, Postgres};
use tracing::instrument;

use penumbra_crypto::merkle::{NoteCommitmentTree, TreeExt};
use penumbra_proto::wallet::{CompactBlock, StateFragment};

use crate::{
    db::{self, schema},
    pending_block::NoteData,
    PendingBlock,
};

#[derive(Debug, Clone)]
pub struct State {
    pool: Pool<Postgres>,
}

impl State {
    /// Connect to the database with the given `uri`.
    #[instrument]
    pub async fn connect(uri: &str) -> Result<Self> {
        tracing::info!("connecting to postgres");
        let pool = PgPoolOptions::new().max_connections(4).connect(uri).await?;
        tracing::info!("building tables");
        db::init_tables(&pool).await?;
        Ok(State { pool })
    }

    pub async fn commit_block(&self, block: PendingBlock) -> Result<()> {
        let mut dbtx = self.pool.begin().await?;

        let nct_anchor = block.note_commitment_tree.root2();
        // TODO: work out what other stuff to put in apphashes
        let app_hash = nct_anchor.to_bytes();
        let height = block.height.expect("height must be set");

        let nct_bytes = bincode::serialize(&block.note_commitment_tree)?;

        query(
            r#"
INSERT INTO blobs (id, data) VALUES ('nct', $1)
ON CONFLICT (id) DO UPDATE SET data = $1
"#,
        )
        .bind(&nct_bytes[..])
        .execute(&mut dbtx)
        .await?;

        query("INSERT INTO blocks (height, nct_anchor, app_hash) VALUES ($1, $2, $3)")
            .bind(height)
            .bind(&nct_anchor.to_bytes()[..])
            .bind(&app_hash[..])
            .execute(&mut dbtx)
            .await?;

        // TODO: this could be batched / use prepared statements
        for (
            note_commitment,
            NoteData {
                ephemeral_key,
                encrypted_note,
                transaction_id,
            },
        ) in block.notes.into_iter()
        {
            query(
                r#"
INSERT INTO notes (
    note_commitment, 
    ephemeral_key, 
    encrypted_note, 
    transaction_id,
    height
) VALUES ($1, $2, $3, $4, $5)
"#,
            )
            .bind(&<[u8; 32]>::from(note_commitment)[..])
            .bind(&ephemeral_key.0[..])
            .bind(&encrypted_note[..])
            .bind(&transaction_id[..])
            .bind(height)
            .execute(&mut dbtx)
            .await?;
        }

        for nullifier in block.spent_nullifiers.into_iter() {
            query("INSERT INTO nullifiers VALUES ($1)")
                .bind(&<[u8; 32]>::from(nullifier)[..])
                .execute(&mut dbtx)
                .await?;
        }

        dbtx.commit().await.map_err(Into::into)
    }

    /// Retrieve the current note commitment tree.
    pub async fn note_commitment_tree(&self) -> Result<NoteCommitmentTree> {
        let mut conn = self.pool.acquire().await?;
        let note_commitment_tree = if let Some(schema::BlobsRow { data, .. }) =
            query_as::<_, schema::BlobsRow>("SELECT id, data FROM blobs WHERE id = 'nct';")
                .fetch_optional(&mut conn)
                .await?
        {
            bincode::deserialize(&data).context("Could not parse saved note commitment tree")?
        } else {
            NoteCommitmentTree::new(0)
        };

        Ok(note_commitment_tree)
    }

    /// Retrieve the latest block info, if any.
    pub async fn latest_block_info(&self) -> Result<Option<schema::BlocksRow>> {
        let mut conn = self.pool.acquire().await?;
        let latest =
            query_as::<_, schema::BlocksRow>("SELECT * FROM blocks ORDER BY height DESC LIMIT 1")
                .fetch_optional(&mut conn)
                .await?;

        Ok(latest)
    }

    /// Retrieve the latest block height.
    pub async fn height(&self) -> Result<i64> {
        Ok(self
            .latest_block_info()
            .await?
            .map(|row| row.height)
            .unwrap_or(0))
    }

    /// Retrieve the latest apphash.
    pub async fn app_hash(&self) -> Result<Vec<u8>> {
        Ok(self
            .latest_block_info()
            .await?
            .map(|row| row.app_hash)
            .unwrap_or(vec![0; 32]))
    }

    /// Retrieve the [`CompactBlock`] for the given height.
    ///
    /// If the block does not exist, the resulting `CompactBlock` will be empty.
    pub async fn compact_block(&self, height: i64) -> Result<CompactBlock> {
        let mut conn = self.pool.acquire().await?;

        Ok(CompactBlock {
            height: height as u32,
            fragments: query_as::<_, (Vec<u8>, Vec<u8>, Vec<u8>)>(
                r#"
SELECT (
    note_commitment, 
    ephemeral_key, 
    encrypted_note
) FROM notes WHERE height = $1
"#,
            )
            .bind(height)
            .fetch_all(&mut conn)
            .await?
            .into_iter()
            .map(
                |(note_commitment, ephemeral_key, encrypted_note)| StateFragment {
                    note_commitment: note_commitment.into(),
                    ephemeral_key: ephemeral_key.into(),
                    encrypted_note: encrypted_note.into(),
                },
            )
            .collect(),
        })
    }
}
