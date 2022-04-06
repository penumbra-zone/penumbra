use anyhow::Result;
use bytes::Bytes;
use tokio::sync::oneshot;
use tracing::Span;

#[derive(Debug)]
pub struct Message {
    pub tx_bytes: Bytes,
    pub rsp_sender: oneshot::Sender<Result<()>>,
    pub span: Span,
}
