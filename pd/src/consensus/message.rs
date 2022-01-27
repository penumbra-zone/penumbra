use tendermint::abci::{ConsensusRequest, ConsensusResponse};
use tokio::sync::oneshot;
use tracing::Span;

#[derive(Debug)]
pub struct Message {
    pub req: ConsensusRequest,
    pub rsp_sender: oneshot::Sender<ConsensusResponse>,
    pub span: Span,
}
