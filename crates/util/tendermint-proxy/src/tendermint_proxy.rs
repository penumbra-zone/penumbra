use crate::TendermintProxy;
use chrono::DateTime;
use penumbra_proto::{
    self as proto,
    util::tendermint_proxy::v1::{
        tendermint_proxy_service_server::TendermintProxyService, AbciQueryRequest,
        AbciQueryResponse, BroadcastTxAsyncRequest, BroadcastTxAsyncResponse,
        BroadcastTxSyncRequest, BroadcastTxSyncResponse, GetBlockByHeightRequest,
        GetBlockByHeightResponse, GetStatusRequest, GetStatusResponse, GetTxRequest, GetTxResponse,
    },
    DomainType, Message,
};
use penumbra_transaction::Transaction;
use std::ops::Deref;
use tap::TapFallible;
use tendermint::{abci::Code, block::Height};
use tendermint_rpc::{Client, HttpClient};
use tonic::Status;
use tracing::instrument;

#[tonic::async_trait]
impl TendermintProxyService for TendermintProxy {
    // Note: the conversions that take place in here could be moved to
    // from/try_from impls, but they're not used anywhere else, so it's
    // unimportant right now, and would require additional wrappers
    // since none of the structs are defined in our crates :(
    // TODO: move those to proto/src/protobuf.rs

    /// Fetches a transaction by hash.
    ///
    /// Returns a [`GetTxResponse`] information about the requested transaction.
    #[instrument(level = "info", skip_all)]
    async fn get_tx(
        &self,
        req: tonic::Request<GetTxRequest>,
    ) -> Result<tonic::Response<GetTxResponse>, Status> {
        // Create an HTTP client, connecting to tendermint.
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Parse the inbound transaction hash from the client request.
        let GetTxRequest { hash, prove } = req.into_inner();
        let hash = hash
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("invalid transaction hash: {e:#?}")))?;

        // Send the request to Tendermint.
        let rsp = client
            .tx(hash, prove)
            .await
            .map(GetTxResponse::from)
            .map_err(|e| Status::unavailable(format!("error getting tx: {e}")))?;

        // Before forwarding along the response, verify that the transaction can be
        // successfully decoded into our domain type.
        Transaction::decode(rsp.tx.as_ref())
            .map_err(|e| Status::unavailable(format!("error decoding tx: {e}")))?;

        Ok(tonic::Response::new(rsp))
    }

    /// Broadcasts a transaction asynchronously.
    #[instrument(
        level = "info",
        skip_all,
        fields(req_id = tracing::field::Empty),
    )]
    async fn broadcast_tx_async(
        &self,
        req: tonic::Request<BroadcastTxAsyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxAsyncResponse>, Status> {
        // Create an HTTP client, connecting to tendermint.
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Process the inbound request, recording the request ID in the tracing span.
        let BroadcastTxAsyncRequest { req_id, params } = req.into_inner();
        tracing::Span::current().record("req_id", req_id);

        // Broadcast the transaction parameters.
        client
            .broadcast_tx_async(params)
            .await
            .map(BroadcastTxAsyncResponse::from)
            .map(tonic::Response::new)
            .map_err(|e| Status::unavailable(format!("error broadcasting tx async: {e}")))
    }

    // Broadcasts a transaction synchronously.
    #[instrument(
        level = "info",
        skip_all,
        fields(req_id = tracing::field::Empty),
    )]
    async fn broadcast_tx_sync(
        &self,
        req: tonic::Request<BroadcastTxSyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxSyncResponse>, Status> {
        // Create an HTTP client, connecting to tendermint.
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Process the inbound request, recording the request ID in the tracing span.
        let BroadcastTxSyncRequest { req_id, params } = req.into_inner();
        tracing::Span::current().record("req_id", req_id);

        // Broadcast the transaction parameters.
        client
            .broadcast_tx_sync(params)
            .await
            .map(BroadcastTxSyncResponse::from)
            .map(tonic::Response::new)
            .map_err(|e| tonic::Status::unavailable(format!("error broadcasting tx sync: {e}")))
            .tap_ok(|res| tracing::debug!("{:?}", res))
    }

    // Queries the current status.
    #[instrument(level = "info", skip_all)]
    async fn get_status(
        &self,
        _req: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            tonic::Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Send the status request.
        client
            .status()
            .await
            .map(GetStatusResponse::from)
            .map(tonic::Response::new)
            .map_err(|e| tonic::Status::unavailable(format!("error querying status: {e}")))
    }

    #[instrument(level = "info", skip_all)]
    async fn abci_query(
        &self,
        req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).map_err(|e| {
            tonic::Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // TODO: how does path validation work on tendermint-rs@29
        let path = req.get_ref().path.clone();
        let data = &req.get_ref().data;
        let height: Height = req
            .get_ref()
            .height
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid height"))?;
        let prove = req.get_ref().prove;
        let res = client
            .abci_query(Some(path), data.clone(), Some(height), prove)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error querying abci: {e}")))?;

        match res.code {
            Code::Ok => Ok(tonic::Response::new(AbciQueryResponse {
                code: u32::from(res.code),
                log: res.log.to_string(),
                info: res.info,
                index: res.index,
                key: res.key,
                value: res.value,
                proof_ops: res.proof.map(|p| proto::tendermint::crypto::ProofOps {
                    ops: p
                        .ops
                        .into_iter()
                        .map(|op| proto::tendermint::crypto::ProofOp {
                            r#type: op.field_type,
                            key: op.key,
                            data: op.data,
                        })
                        .collect(),
                }),
                height: i64::try_from(res.height.value()).map_err(|_| {
                    tonic::Status::internal(
                        "height from tendermint overflowed i64, this should never happen",
                    )
                })?,
                codespace: res.codespace,
            })),
            tendermint::abci::Code::Err(e) => Err(tonic::Status::unavailable(format!(
                "error querying abci: {e}"
            ))),
        }
    }

    #[instrument(level = "info", skip_all)]
    async fn get_block_by_height(
        &self,
        req: tonic::Request<GetBlockByHeightRequest>,
    ) -> Result<tonic::Response<GetBlockByHeightResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).map_err(|e| {
            tonic::Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        let res = client
            .block(
                tendermint::block::Height::try_from(req.get_ref().height)
                    .expect("height should be less than 2^63"),
            )
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error querying abci: {e}")))?;

        // TODO: these conversions exist because the penumbra proto files define
        // their own proxy methods, since tendermint does not include them. This results
        // in duplicated proto types relative to the tendermint-proto ones.

        // The tendermint-rs `Timestamp` type is a newtype wrapper
        // around a `time::PrimitiveDateTime` however it's private so we
        // have to use string parsing to get to the prost type we want :(
        let header_time = DateTime::parse_from_rfc3339(&res.block.header.time.to_rfc3339())
            .expect("timestamp should roundtrip to string");
        Ok(tonic::Response::new(GetBlockByHeightResponse {
            block_id: Some(penumbra_proto::tendermint::types::BlockId {
                hash: res.block_id.hash.into(),
                part_set_header: Some(penumbra_proto::tendermint::types::PartSetHeader {
                    total: res.block_id.part_set_header.total,
                    hash: res.block_id.part_set_header.hash.into(),
                }),
            }),
            block: Some(proto::tendermint::types::Block {
                header: Some(proto::tendermint::types::Header {
                    version: Some(penumbra_proto::tendermint::version::Consensus {
                        block: res.block.header.version.block,
                        app: res.block.header.version.app,
                    }),
                    chain_id: res.block.header.chain_id.into(),
                    height: res.block.header.height.into(),
                    time: Some(pbjson_types::Timestamp {
                        seconds: header_time.timestamp(),
                        nanos: header_time.timestamp_nanos_opt().ok_or_else(|| tonic::Status::invalid_argument("missing header_time nanos"))? as i32,
                    }),
                    last_block_id: res.block.header.last_block_id.map(|id| {
                        penumbra_proto::tendermint::types::BlockId {
                            hash: id.hash.into(),
                            part_set_header: Some(
                                penumbra_proto::tendermint::types::PartSetHeader {
                                    total: id.part_set_header.total,
                                    hash: id.part_set_header.hash.into(),
                                },
                            ),
                        }
                    }),
                    last_commit_hash: res
                        .block
                        .header
                        .last_commit_hash
                        .map(Into::into)
                        .unwrap_or_default(),
                    data_hash: res
                        .block
                        .header
                        .data_hash
                        .map(Into::into)
                        .unwrap_or_default(),
                    validators_hash: res.block.header.validators_hash.into(),
                    next_validators_hash: res.block.header.next_validators_hash.into(),
                    consensus_hash: res.block.header.consensus_hash.into(),
                    app_hash: res.block.header.app_hash.into(),
                    last_results_hash: res
                        .block
                        .header
                        .last_results_hash
                        .map(Into::into)
                        .unwrap_or_default(),
                    evidence_hash: res
                        .block
                        .header
                        .evidence_hash
                        .map(Into::into)
                        .unwrap_or_default(),
                    proposer_address: res.block.header.proposer_address.into(),
                }),
                data: Some(proto::tendermint::types::Data {
                    txs: res.block.data,
                }),
                evidence: Some(proto::tendermint::types::EvidenceList {
                    evidence: res
                        .block
                        .evidence
                        .into_vec()
                        .iter()
                        .map(|e| Ok(proto::tendermint::types::Evidence {
                            sum: Some( match e {
                                tendermint::evidence::Evidence::DuplicateVote(e) => {
                                   let e2 = tendermint_proto::types::DuplicateVoteEvidence::from(e.deref().clone());
                                    proto::tendermint::types::evidence::Sum::DuplicateVoteEvidence(proto::tendermint::types::DuplicateVoteEvidence{
                                    vote_a: Some(proto::tendermint::types::Vote{
                                        r#type: match e.votes().0.vote_type {
                                            tendermint::vote::Type::Prevote => proto::tendermint::types::SignedMsgType::Prevote as i32,
                                            tendermint::vote::Type::Precommit => proto::tendermint::types::SignedMsgType::Precommit as i32,
                                        },
                                        height: e.votes().0.height.into(),
                                        round: e.votes().0.round.into(),
                                        block_id: Some(proto::tendermint::types::BlockId{
                                            hash: e.votes().0.block_id.expect("block id").hash.into(),
                                            part_set_header: Some(proto::tendermint::types::PartSetHeader{
                                                total: e.votes().0.block_id.expect("block id").part_set_header.total,
                                                hash: e.votes().0.block_id.expect("block id").part_set_header.hash.into(),
                                            }),
                                        }),
                                        timestamp: Some(pbjson_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&e.votes().0.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&e.votes().0.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos_opt().ok_or_else(|| tonic::Status::invalid_argument("missing timestamp nanos"))? as i32,
                                        }),
                                        validator_address: e.votes().0.validator_address.into(),
                                        validator_index: e.votes().0.validator_index.into(),
                                        signature: e.votes().0.signature.clone().expect("signed vote").into(),
                                    }),
                                    vote_b: Some(proto::tendermint::types::Vote{
                                        r#type: match e.votes().1.vote_type {
                                            tendermint::vote::Type::Prevote => proto::tendermint::types::SignedMsgType::Prevote as i32,
                                            tendermint::vote::Type::Precommit => proto::tendermint::types::SignedMsgType::Precommit as i32,
                                        },
                                        height: e.votes().1.height.into(),
                                        round: e.votes().1.round.into(),
                                        block_id: Some(proto::tendermint::types::BlockId{
                                            hash: e.votes().1.block_id.expect("block id").hash.into(),
                                            part_set_header: Some(proto::tendermint::types::PartSetHeader{
                                                total: e.votes().1.block_id.expect("block id").part_set_header.total,
                                                hash: e.votes().1.block_id.expect("block id").part_set_header.hash.into(),
                                            }),
                                        }),
                                        timestamp: Some(pbjson_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&e.votes().1.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&e.votes().1.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos_opt().ok_or_else(|| tonic::Status::invalid_argument("missing timestamp nanos"))?  as i32,
                                        }),
                                        validator_address: e.votes().1.validator_address.into(),
                                        validator_index: e.votes().1.validator_index.into(),
                                        signature: e.votes().1.signature.clone().expect("signed vote").into(),
                                    }),
                                    total_voting_power: e2.total_voting_power,
                                    validator_power: e2.validator_power,
                                    timestamp: e2.timestamp.map(|t| pbjson_types::Timestamp{seconds: t.seconds, nanos: t.nanos}),
                                })
                                },
                                tendermint::evidence::Evidence::LightClientAttack(e) => {
                                   let e2 = tendermint_proto::types::LightClientAttackEvidence::from(e.deref().clone());
                                   let e2_bytes = e2.encode_to_vec();
                                   let e3 = proto::tendermint::types::LightClientAttackEvidence::decode(e2_bytes.as_slice()).expect("can decode encoded data");
                                    proto::tendermint::types::evidence::Sum::LightClientAttackEvidence(
                                        e3
                                    )
                                }

                        }),
                        }))
                        .collect::<anyhow::Result<Vec<_>, tonic::Status>>()?,
                }),
                last_commit: Some(proto::tendermint::types::Commit {
                    height: res.block.last_commit.as_ref().expect("last_commit").height.into(),
                    round: res.block.last_commit.as_ref().expect("last_commit").round.into(),
                    block_id: Some(proto::tendermint::types::BlockId {
                        hash: res.block.last_commit.as_ref().expect("last_commit").block_id.hash.into(),
                        part_set_header: Some(proto::tendermint::types::PartSetHeader {
                            total: res.block.last_commit.as_ref().expect("last_commit").block_id.part_set_header.total,
                            hash: res.block.last_commit.as_ref().expect("last_commit").block_id.part_set_header.hash.into(),
                        }),
                    }),
                    signatures: match res.block.last_commit {
                        Some(commit) => commit
                            .signatures
                            .into_iter()
                            .map(|s| Ok({
                                match s {
                                    tendermint::block::CommitSig::BlockIdFlagAbsent => proto::tendermint::types::CommitSig {
                                        block_id_flag: proto::tendermint::types::BlockIdFlag::Absent as i32,
                                        // No validator address, or timestamp is recorded for this variant. Not sure if this is a bug in tendermint-rs or not.
                                        validator_address: vec![],
                                        timestamp: None,
                                        signature: vec![],
                                    },
                                    tendermint::block::CommitSig::BlockIdFlagCommit { validator_address, timestamp, signature } => proto::tendermint::types::CommitSig {
                                        block_id_flag: proto::tendermint::types::BlockIdFlag::Commit as i32,
                                        validator_address: validator_address.into(),
                                        timestamp: Some(pbjson_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos_opt().ok_or_else(|| tonic::Status::invalid_argument("missing timestamp nanos"))? as i32,
                                        }),
                                        signature: signature.expect("signature").into(),
                                    },
                                    tendermint::block::CommitSig::BlockIdFlagNil { validator_address, timestamp, signature } => proto::tendermint::types::CommitSig {
                                        block_id_flag: proto::tendermint::types::BlockIdFlag::Nil as i32,
                                        validator_address: validator_address.into(),
                                        timestamp: Some(pbjson_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos_opt().ok_or_else(|| tonic::Status::invalid_argument("missing timestamp nanos"))? as i32,
                                        }),
                                        signature: signature.expect("signature").into(),
                                    },
                                }
                            }))
                            .collect::<anyhow::Result<Vec<_>, tonic::Status>>()?,
                        None => vec![],
                    },
                }),
            }),
        }))
    }
}
