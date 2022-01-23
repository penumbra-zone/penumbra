use anyhow::Result;
use penumbra_crypto::merkle::NoteCommitmentTree;
use tendermint::abci::{ConsensusRequest, ConsensusResponse};
use tokio::sync::{mpsc, oneshot};

use crate::{stake::StakeChanges, State};

type Message = (ConsensusRequest, oneshot::Sender<ConsensusResponse>, tracing::Span);

struct Message = {
    req: ConsensusRequest,
    rsp_sender: 
}

pub struct Worker {
    // fields
    requests: mpsc::Receiver<Message>,
    state: State,


    // these have to be optional, since we only get them in EndBlock
    height: Option<u64>,
    epoch: Option<Epoch>,

    // structs for each group
    stake_changes: StakeChanges,
    note_changes: ShieldedPoolChanges,
}

impl Worker {
    pub async fn run(mut self) -> Result<()> {
        while let Some((req, rsp_sender)) = self.requests.recv().await {
            match req {
                ConsensusRequest::InitChain(init_chain) => todo!(),
                ConsensusRequest::BeginBlock(init_chain) => todo!(),
                ConsensusRequest::DeliverTx(init_chain) => todo!(),
                ConsensusRequest::EndBlock(init_chain) => todo!(),
                ConsensusRequest::Commit => todo!(),
            }
        }
        unreachable!()
    }
}
