use anyhow::Context as _;
use sqlx::{query_as, Pool, Postgres};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

use penumbra_proto::{
    transaction,
    wallet::{
        wallet_server::Wallet, CompactBlock, CompactBlockRangeRequest, TransactionByNoteRequest,
    },
};

use crate::{
    dbschema::{PenumbraStateFragment, PenumbraTransaction},
    dbutils::db_connection,
};

/// The Penumbra wallet service.
pub struct WalletApp {
    db_pool: Pool<Postgres>,
}

impl WalletApp {
    pub async fn new() -> Result<WalletApp, anyhow::Error> {
        Ok(WalletApp {
            db_pool: db_connection().await.context("Could not open database")?,
        })
    }
}

#[tonic::async_trait]
impl Wallet for WalletApp {
    type CompactBlockRangeStream = ReceiverStream<Result<CompactBlock, Status>>;

    async fn compact_block_range(
        &self,
        request: tonic::Request<CompactBlockRangeRequest>,
    ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, Status> {
        let mut p = self
            .db_pool
            .acquire()
            .await
            .map_err(|_| tonic::Status::unavailable("server error"))?;
        let request = request.into_inner();
        let start_height = request.start_height;
        let end_height = request.end_height;

        if end_height < start_height {
            return Err(tonic::Status::failed_precondition(
                "end height must be greater than start height",
            ));
        }

        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            for block_height in start_height..=end_height {
                let rows = query_as::<_, PenumbraStateFragment>(
                    r#"
SELECT note_commitment, ephemeral_key, note_ciphertext FROM notes
WHERE transaction_id IN (select id from transactions where block_id IN
(SELECT id FROM blocks WHERE height = $1)
)
"#,
                )
                .bind(block_height)
                .fetch_all(&mut p)
                .await
                .expect("if no results will return empty state fragments");

                let block = CompactBlock {
                    height: block_height,
                    fragment: rows.into_iter().map(|x| x.into()).collect::<Vec<_>>(),
                };
                tracing::info!("sending block response: {:?}", block);
                tx.send(Ok(block.clone())).await.unwrap();
            }
        });

        Ok(tonic::Response::new(Self::CompactBlockRangeStream::new(rx)))
    }

    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<transaction::Transaction>, Status> {
        let mut p = self
            .db_pool
            .acquire()
            .await
            .map_err(|_| tonic::Status::unavailable("server error"))?;

        let note_commitment = request.into_inner().cm;
        let rows = query_as::<_, PenumbraTransaction>(
            r#"
SELECT transactions.transaction FROM transactions
JOIN notes ON transactions.id = (SELECT transaction_id FROM notes WHERE note_commitment=$1
)
"#,
        )
        .bind(note_commitment)
        .fetch_one(&mut p)
        .await
        .map_err(|_| tonic::Status::not_found("transaction not found"))?;

        let transaction = penumbra_crypto::Transaction::try_from(&rows.transaction[..])
            .map_err(|_| tonic::Status::data_loss("transaction not well formed"))?;
        let protobuf_transaction: transaction::Transaction = transaction.into();

        Ok(tonic::Response::new(protobuf_transaction))
    }
}
