use std::str::FromStr;

use chrono::DateTime;
use penumbra_proto::{
    self as proto, tendermint_proxy::service_server::Service as TendermintService,
    tendermint_proxy::service_server::ServiceServer as TendermintServiceServer,
};

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
use tendermint_rpc::abci::Path;
use tendermint_rpc::{Client, HttpClient};
use tonic::Status;

// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
//use tracing_futures::Instrument;

use super::Info;

#[tonic::async_trait]
trait TendermintServiceExt {}

// Note: the conversions that take place in here could be moved to
// from/try_from impls, but they're not used anywhere else, so it's
// unimportant right now, and would require additional wrappers
// since none of the structs are defined in our crates :(
// TODO: move those to proto/src/protobuf.rs

#[tonic::async_trait]
impl<T: TendermintService> TendermintServiceExt for T {
}

#[tonic::async_trait]
impl TendermintService for Info {
    async fn abci_query(
        &self,
        req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let path = Path::from_str(&req.get_ref().path).map_err(|_| tonic::Status::invalid_argument("invalid abci path"))?;
        let data = &req.get_ref().data;
        let height: Height = req
            .get_ref()
            .height
            .try_into().map_err(|_| tonic::Status::invalid_argument("invalid height"))?;
        let prove = req.get_ref().prove;
        let res = client
            .abci_query(Some(path), data.clone(), Some(height), prove)
            .await.map_err(|e| tonic::Status::unavailable(format!(
                    "error querying abci: {}",
                    e
                )))?;

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
            tendermint_rpc::abci::Code::Err(e) => Err(tonic::Status::unavailable(format!(
                "error querying abci: {}",
                e
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
            .await.map_err(|e| tonic::Status::unavailable(format!(
                    "error querying abci: {}",
                    e
                )))?;

        // The tendermint-rs `Timestamp` type is a newtype wrapper
        // around a `time::PrimitiveDateTime` however it's private so we
        // have to use string parsing to get to the prost type we want :(
        let header_time = DateTime::parse_from_rfc3339(&res.block.header.time.to_rfc3339())
            .expect("timestamp should roundtrip to string");
        Ok(tonic::Response::new(GetBlockByHeightResponse {
            block_id: Some(penumbra_proto::core::tendermint::types::BlockId {
                hash: res.block_id.hash.into(),
                part_set_header: Some(penumbra_proto::core::tendermint::types::PartSetHeader {
                    total: res.block_id.part_set_header.total,
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
                        seconds: header_time.timestamp(),
                        nanos: header_time.timestamp_nanos() as i32,
                    }),
                    last_block_id: res.block.header.last_block_id.map(|id| {
                        penumbra_proto::core::tendermint::types::BlockId {
                            hash: id.hash.into(),
                            part_set_header: Some(
                                penumbra_proto::core::tendermint::types::PartSetHeader {
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
                        .map(|e| proto::core::tendermint::types::Evidence {
                            sum: Some( match e {
                                tendermint::evidence::Evidence::DuplicateVote(e) => {
                                   let e2 = tendermint_proto::types::DuplicateVoteEvidence::from(e.clone()); 
                                    proto::core::tendermint::types::evidence::Sum::DuplicateVoteEvidence(proto::core::tendermint::types::DuplicateVoteEvidence{
                                    vote_a: Some(proto::core::tendermint::types::Vote{
                                        r#type: match e.votes().0.vote_type {
                                            tendermint::vote::Type::Prevote => proto::core::tendermint::types::SignedMsgType::Prevote as i32,
                                            tendermint::vote::Type::Precommit => proto::core::tendermint::types::SignedMsgType::Precommit as i32,
                                        },
                                        height: e.votes().0.height.into(),
                                        round: e.votes().0.round.into(),
                                        block_id: Some(proto::core::tendermint::types::BlockId{
                                            hash: e.votes().0.block_id.expect("block id").hash.into(),
                                            part_set_header: Some(proto::core::tendermint::types::PartSetHeader{
                                                total: e.votes().0.block_id.expect("block id").part_set_header.total,
                                                hash: e.votes().0.block_id.expect("block id").part_set_header.hash.into(),
                                            }),
                                        }),
                                        timestamp: Some(prost_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&e.votes().0.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&e.votes().0.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
                                        }),
                                        validator_address: e.votes().0.validator_address.into(),
                                        validator_index: e.votes().0.validator_index.into(),
                                        signature: e.votes().0.signature.clone().expect("signed vote").into(),
                                    }),
                                    vote_b: Some(proto::core::tendermint::types::Vote{
                                        r#type: match e.votes().1.vote_type {
                                            tendermint::vote::Type::Prevote => proto::core::tendermint::types::SignedMsgType::Prevote as i32,
                                            tendermint::vote::Type::Precommit => proto::core::tendermint::types::SignedMsgType::Precommit as i32,
                                        },
                                        height: e.votes().1.height.into(),
                                        round: e.votes().1.round.into(),
                                        block_id: Some(proto::core::tendermint::types::BlockId{
                                            hash: e.votes().1.block_id.expect("block id").hash.into(),
                                            part_set_header: Some(proto::core::tendermint::types::PartSetHeader{
                                                total: e.votes().1.block_id.expect("block id").part_set_header.total,
                                                hash: e.votes().1.block_id.expect("block id").part_set_header.hash.into(),
                                            }),
                                        }),
                                        timestamp: Some(prost_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&e.votes().1.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&e.votes().1.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
                                        }),
                                        validator_address: e.votes().1.validator_address.into(),
                                        validator_index: e.votes().1.validator_index.into(),
                                        signature: e.votes().1.signature.clone().expect("signed vote").into(),
                                    }),
                                    total_voting_power: e2.total_voting_power,
                                    validator_power: e2.validator_power,
                                    timestamp: e2.timestamp.map(|t| prost_types::Timestamp{seconds: t.seconds, nanos: t.nanos}),
                                })
                            },
                                // This variant is currently unimplemented in tendermint-rs, so we can't supply
                                // conversions for it.
                                tendermint::evidence::Evidence::LightClientAttackEvidence => proto::core::tendermint::types::evidence::Sum::LightClientAttackEvidence(proto::core::tendermint::types::LightClientAttackEvidence{
                                    conflicting_block: None,
                                    common_height: -1,
                                    byzantine_validators: vec![],
                                    timestamp: None,
                                    total_voting_power: -1,
                                }),
                                // This variant is described as not-implemented in tendermint-rs,
                                // and doesn't exist in the prost-generated protobuf types
                                tendermint::evidence::Evidence::ConflictingHeaders(_) => panic!("unimplemented"),
                        }),
                        })
                        .collect(),
                }),
                last_commit: Some(proto::core::tendermint::types::Commit {
                    height: res.block.last_commit.as_ref().expect("last_commit").height.into(),
                    round: res.block.last_commit.as_ref().expect("last_commit").round.into(),
                    block_id: Some(proto::core::tendermint::types::BlockId {
                        hash: res.block.last_commit.as_ref().expect("last_commit").block_id.hash.into(),
                        part_set_header: Some(proto::core::tendermint::types::PartSetHeader {
                            total: res.block.last_commit.as_ref().expect("last_commit").block_id.part_set_header.total,
                            hash: res.block.last_commit.as_ref().expect("last_commit").block_id.part_set_header.hash.into(),
                        }),
                    }),
                    signatures: match res.block.last_commit {
                        Some(commit) => commit
                            .signatures
                            .into_iter()
                            .map(|s| {
                                match s {
                                    tendermint::block::CommitSig::BlockIdFlagAbsent => proto::core::tendermint::types::CommitSig {
                                        block_id_flag: proto::core::tendermint::types::BlockIdFlag::Absent as i32,
                                        // No validator address, or timestamp is recorded for this variant. Not sure if this is a bug in tendermint-rs or not.
                                        validator_address: vec![],
                                        timestamp: None,
                                        signature: vec![],
                                    },
                                    tendermint::block::CommitSig::BlockIdFlagCommit { validator_address, timestamp, signature } => proto::core::tendermint::types::CommitSig {
                                        block_id_flag: proto::core::tendermint::types::BlockIdFlag::Commit as i32,
                                        validator_address: validator_address.into(),
                                        timestamp: Some(prost_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
                                        }),
                                        signature: signature.expect("signature").into(),
                                    },
                                    tendermint::block::CommitSig::BlockIdFlagNil { validator_address, timestamp, signature } => proto::core::tendermint::types::CommitSig {
                                        block_id_flag: proto::core::tendermint::types::BlockIdFlag::Nil as i32,
                                        validator_address: validator_address.into(),
                                        timestamp: Some(prost_types::Timestamp{
                                            seconds: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp(),
                                            nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
                                        }),
                                        signature: signature.expect("signature").into(),
                                    },
                                }
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
