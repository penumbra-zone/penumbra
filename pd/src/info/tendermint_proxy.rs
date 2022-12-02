use std::pin::Pin;
use std::str::FromStr;

use async_stream::try_stream;
use chrono::DateTime;
use futures::StreamExt;
use futures::TryStreamExt;
use penumbra_chain::AppHashRead;
use penumbra_chain::StateReadExt as _;
use penumbra_component::stake::StateReadExt as _;
use penumbra_crypto::asset::{self, Asset};
use penumbra_proto::{
    self as proto, tendermint_proxy::service_server::Service as TendermintService,
};

use penumbra_storage::StateRead;
use proto::tendermint_proxy::AbciQueryRequest;
use proto::tendermint_proxy::AbciQueryResponse;
use proto::tendermint_proxy::GetBlockByHeightRequest;
use proto::tendermint_proxy::GetBlockByHeightResponse;
use proto::tendermint_proxy::GetLatestBlockRequest;
use proto::tendermint_proxy::GetLatestBlockResponse;
use proto::tendermint_proxy::GetLatestValidatorSetRequest;
use proto::tendermint_proxy::GetLatestValidatorSetResponse;
use proto::tendermint_proxy::GetNodeInfoRequest;
use proto::tendermint_proxy::GetNodeInfoResponse;
use proto::tendermint_proxy::GetSyncingRequest;
use proto::tendermint_proxy::GetSyncingResponse;
use proto::tendermint_proxy::GetValidatorSetByHeightRequest;
use proto::tendermint_proxy::GetValidatorSetByHeightResponse;
use tendermint::block::Height;
use tendermint::time as tm_time;
use tendermint_config::net;
use tendermint_rpc::abci::Path;
use tendermint_rpc::HttpClientUrl;
use tendermint_rpc::{Client, HttpClient};
use tonic::Status;
use tracing::instrument;

// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
//use tracing_futures::Instrument;

use super::Info;

// Note: the conversions that take place in here could be moved to
// from/try_from impls, but they're not used anywhere else, so it's
// unimportant right now.

#[tonic::async_trait]
impl TendermintService for Info {
    async fn abci_query(
        &self,
        req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let path = Path::from_str(&req.get_ref().path)
            .or_else(|_| Err(tonic::Status::invalid_argument("invalid abci path")))?;
        let data = req.get_ref().data;
        let height: Height = req
            .get_ref()
            .height
            .try_into()
            .or_else(|_| Err(tonic::Status::invalid_argument("invalid height")))?;
        let prove = req.get_ref().prove;
        let res = client
            .abci_query(Some(path), data, Some(height), prove)
            .await
            .or_else(|e| {
                Err(tonic::Status::unavailable(format!(
                    "error querying abci: {}",
                    e.to_string()
                )))
            })?;

        match res.code {
            tendermint_rpc::abci::Code::Ok => Ok(tonic::Response::new(AbciQueryResponse {
                code: u32::from(res.code),
                log: res.log.to_string(),
                info: res.info,
                index: res.index,
                key: res.key,
                value: res.value,
                proof_ops: res
                    .proof
                    .map(|p| penumbra_proto::tendermint_proxy::ProofOps {
                        ops: p
                            .ops
                            .into_iter()
                            .map(|op| penumbra_proto::tendermint_proxy::ProofOp {
                                r#type: op.field_type.into(),
                                key: op.key.into(),
                                data: op.data.into(),
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
            tendermint_rpc::abci::Code::Err(e) => Err(tonic::Status::unavailable(format!(
                "error querying abci: {}",
                e.to_string()
            ))),
        }
    }

    async fn get_node_info(
        &self,
        req: tonic::Request<GetNodeInfoRequest>,
    ) -> Result<tonic::Response<GetNodeInfoResponse>, Status> {
        todo!()
    }

    async fn get_syncing(
        &self,
        req: tonic::Request<GetSyncingRequest>,
    ) -> Result<tonic::Response<GetSyncingResponse>, Status> {
        todo!()
    }

    async fn get_latest_block(
        &self,
        req: tonic::Request<GetLatestBlockRequest>,
    ) -> Result<tonic::Response<GetLatestBlockResponse>, Status> {
        todo!()
    }

    async fn get_block_by_height(
        &self,
        req: tonic::Request<GetBlockByHeightRequest>,
    ) -> Result<tonic::Response<GetBlockByHeightResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let res = client
            .block(
                tendermint::block::Height::try_from(req.get_ref().height)
                    .expect("height should be less than 2^63"),
            )
            .await
            .or_else(|e| {
                Err(tonic::Status::unavailable(format!(
                    "error querying abci: {}",
                    e.to_string()
                )))
            })?;

        // The tendermint-rs `Timestamp` type is a newtype wrapper
        // around a `time::PrimitiveDateTime` however it's private so we
        // have to use string parsing to get to the prost type we want :(
        let time = DateTime::parse_from_rfc3339(&res.block.header.time.to_rfc3339())
            .expect("timestamp should roundtrip to string");
        Ok(tonic::Response::new(GetBlockByHeightResponse {
            block_id: Some(penumbra_proto::core::tendermint::types::BlockId {
                hash: res.block_id.hash.into(),
                part_set_header: Some(penumbra_proto::core::tendermint::types::PartSetHeader {
                    total: res.block_id.part_set_header.total.into(),
                    hash: res.block_id.part_set_header.hash.into(),
                }),
            }),
            sdk_block: Some(penumbra_proto::tendermint_proxy::Block {
                header: Some(penumbra_proto::tendermint_proxy::Header {
                    version: Some(penumbra_proto::core::tendermint::version::Consensus {
                        block: res.block.header.version.block,
                        app: res.block.header.version.app,
                    }),
                    chain_id: res.block.header.chain_id.into(),
                    height: res.block.header.height.into(),
                    time: Some(prost_types::Timestamp {
                        seconds: time.timestamp(),
                        nanos: time.timestamp_nanos() as i32,
                    }),
                    last_block_id: res.block.header.last_block_id.map(|id| {
                        penumbra_proto::core::tendermint::types::BlockId {
                            hash: id.hash.into(),
                            part_set_header: Some(
                                penumbra_proto::core::tendermint::types::PartSetHeader {
                                    total: id.part_set_header.total.into(),
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
                    proposer_address: res.block.header.proposer_address.to_string(),
                }),
                data: Some(proto::core::tendermint::types::Data {
                    txs: res.block.data,
                }),
                evidence: Some(proto::core::tendermint::types::EvidenceList {
                    evidence: res
                        .block
                        .evidence
                        .into_vec()
                        .iter()
                        .map(Into::into)
                        .collect(),
                }),
                last_commit: Some(proto::core::tendermint::types::Commit {
                    height: res.block.last_commit.height.into(),
                    round: res.block.last_commit.round.into(),
                    block_id: Some(proto::core::tendermint::types::BlockId {
                        hash: res.block.last_commit.block_id.hash.into(),
                        part_set_header: Some(proto::core::tendermint::types::PartSetHeader {
                            total: res.block.last_commit.block_id.part_set_header.total.into(),
                            hash: res.block.last_commit.block_id.part_set_header.hash.into(),
                        }),
                    }),
                    signatures: match res.block.last_commit {
                        Some(commit) => commit
                            .signatures
                            .into_iter()
                            .map(|s| {
                                Some(proto::core::tendermint::types::CommitSig {
                                    block_id_flag: s.block_id_flag.into(),
                                    validator_address: s.validator_address.to_string(),
                                    timestamp: Some(prost_types::Timestamp {
                                        seconds: s.timestamp.timestamp(),
                                        nanos: s.timestamp.timestamp_nanos() as i32,
                                    }),
                                    signature: s.signature.into(),
                                })
                            })
                            .collect(),
                        None => vec![],
                    },
                }),
            }),

            // Deprecated:
            block: None,
        }))
    }

    async fn get_latest_validator_set(
        &self,
        req: tonic::Request<GetLatestValidatorSetRequest>,
    ) -> Result<tonic::Response<GetLatestValidatorSetResponse>, Status> {
        todo!()
    }

    async fn get_validator_set_by_height(
        &self,
        req: tonic::Request<GetValidatorSetByHeightRequest>,
    ) -> Result<tonic::Response<GetValidatorSetByHeightResponse>, Status> {
        todo!()
    }
}
