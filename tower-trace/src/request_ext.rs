use sha2::{Digest, Sha256};
use tendermint::abci::{ConsensusRequest, InfoRequest, MempoolRequest, SnapshotRequest};
use tendermint::v0_34::abci::request::{
    BeginBlock, CheckTx, DeliverTx, EndBlock, InitChain, Query, Request,
};
use tracing::error_span;

pub trait RequestExt {
    /// Create a [`tracing::Span`] for this request, including the request name
    /// and some relevant context (but not including the entire request data).
    fn create_span(&self) -> tracing::Span;
}

impl RequestExt for ConsensusRequest {
    fn create_span(&self) -> tracing::Span {
        // Create a parent "abci" span. All of these spans are at error level, so they're always recorded.
        let p = error_span!("abci");
        match self {
            ConsensusRequest::BeginBlock(BeginBlock { header, .. }) => {
                error_span!(parent: &p, "BeginBlock", height = ?header.height.value())
            }
            ConsensusRequest::DeliverTx(DeliverTx { tx }) => {
                error_span!(parent: &p, "DeliverTx", txid = ?hex::encode(Sha256::digest(tx.as_ref())))
            }
            ConsensusRequest::EndBlock(EndBlock { height }) => {
                error_span!(parent: &p, "EndBlock", ?height)
            }
            ConsensusRequest::Commit => error_span!(parent: &p, "Commit"),
            ConsensusRequest::InitChain(InitChain { chain_id, .. }) => {
                error_span!(parent: &p, "InitChain", ?chain_id)
            }
            ConsensusRequest::PrepareProposal(_) => {
                unimplemented!("unimplemented in Tendermint v0.34.x")
            }
            ConsensusRequest::ProcessProposal(_) => {
                unimplemented!("unimplemented in Tendermint v0.34.x")
            }
        }
    }
}

impl RequestExt for MempoolRequest {
    fn create_span(&self) -> tracing::Span {
        // Create a parent "abci" span. All of these spans are at error level, so they're always recorded.
        let p = error_span!("abci");
        match self {
            MempoolRequest::CheckTx(CheckTx { kind, tx }) => {
                error_span!(parent: &p, "CheckTx", ?kind, txid = ?hex::encode(Sha256::digest(tx.as_ref())))
            }
        }
    }
}

impl RequestExt for InfoRequest {
    fn create_span(&self) -> tracing::Span {
        // Create a parent "abci" span. All of these spans are at error level, so they're always recorded.
        let p = error_span!("abci");
        match self {
            InfoRequest::Info(_) => error_span!(parent: &p, "Info"),
            InfoRequest::Query(Query {
                path,
                height,
                prove,
                ..
            }) => {
                error_span!(parent: &p, "Query", ?path, ?height, prove)
            }
            InfoRequest::Echo(_) => error_span!(parent: &p, "Echo"),
            InfoRequest::SetOption(_) => todo!("not implemented"),
        }
    }
}

impl RequestExt for SnapshotRequest {
    fn create_span(&self) -> tracing::Span {
        // Create a parent "abci" span. All of these spans are at error level, so they're always recorded.
        let p = error_span!("abci");
        match self {
            SnapshotRequest::ListSnapshots => error_span!(parent: &p, "ListSnapshots"),
            SnapshotRequest::OfferSnapshot(_) => error_span!(parent: &p, "OfferSnapshot"),
            SnapshotRequest::LoadSnapshotChunk(_) => error_span!(parent: &p, "LoadSnapshotChunk"),
            SnapshotRequest::ApplySnapshotChunk(_) => error_span!(parent: &p, "ApplySnapshotChunk"),
        }
    }
}

impl RequestExt for Request {
    fn create_span(&self) -> tracing::Span {
        // Create a parent "abci" span. All of these spans are at error level, so they're always recorded.
        let p = error_span!("abci");
        match self {
            Request::Info(_) => error_span!(parent: &p, "Info"),
            Request::Query(Query {
                path,
                height,
                prove,
                ..
            }) => {
                error_span!(parent: &p, "Query", ?path, ?height, prove)
            }
            Request::CheckTx(CheckTx { kind, tx }) => {
                error_span!(parent: &p, "CheckTx", ?kind, txid = ?hex::encode(Sha256::digest(tx.as_ref())))
            }
            Request::BeginBlock(BeginBlock { hash, header, .. }) => {
                error_span!(parent: &p, "BeginBlock", height = ?header.height, hash = ?hex::encode(hash.as_ref()))
            }
            Request::DeliverTx(DeliverTx { tx }) => {
                error_span!(parent: &p, "DeliverTx", txid = ?hex::encode(Sha256::digest(tx.as_ref())))
            }
            Request::EndBlock(EndBlock { height }) => error_span!(parent: &p, "EndBlock", ?height),
            Request::Commit => error_span!(parent: &p, "Commit"),
            Request::InitChain(InitChain { chain_id, .. }) => {
                error_span!(parent: &p, "InitChain", ?chain_id)
            }
            Request::Flush => error_span!(parent: &p, "Flush"),
            Request::Echo(_) => error_span!(parent: &p, "Echo"),
            Request::ListSnapshots => error_span!(parent: &p, "ListSnapshots"),
            Request::OfferSnapshot(_) => error_span!(parent: &p, "OfferSnapshot"),
            Request::LoadSnapshotChunk(_) => error_span!(parent: &p, "LoadSnapshotChunk"),
            Request::ApplySnapshotChunk(_) => error_span!(parent: &p, "ApplySnapshotChunk"),
            Request::SetOption(_) => todo!("not implemented"),
        }
    }
}
