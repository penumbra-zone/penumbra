use anyhow::Result;
use tendermint::block;
use tokio::sync::watch;
use tracing::instrument;

mod reader;
mod writer;

pub use reader::Reader;
pub use writer::Writer;

#[instrument]
// TODO: killme
pub async fn new(uri: &str) -> Result<(Reader, Writer)> {
    Err(anyhow::anyhow!("deprecated"))
}
