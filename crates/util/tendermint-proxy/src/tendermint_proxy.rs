use chrono::DateTime;
use penumbra_proto::{self as proto};

use penumbra_transaction::Transaction;
use proto::client::v1alpha1::tendermint_proxy_service_server::TendermintProxyService;
use proto::client::v1alpha1::AbciQueryRequest;
use proto::client::v1alpha1::AbciQueryResponse;
use proto::client::v1alpha1::BroadcastTxAsyncRequest;
use proto::client::v1alpha1::BroadcastTxAsyncResponse;
use proto::client::v1alpha1::BroadcastTxSyncRequest;
use proto::client::v1alpha1::BroadcastTxSyncResponse;
use proto::client::v1alpha1::GetBlockByHeightRequest;
use proto::client::v1alpha1::GetBlockByHeightResponse;
use proto::client::v1alpha1::GetStatusRequest;
use proto::client::v1alpha1::GetStatusResponse;
use proto::client::v1alpha1::GetTxRequest;
use proto::client::v1alpha1::GetTxResponse;
use proto::DomainType;
use tendermint::abci::Code;
use tendermint::block::Height;
use tendermint_rpc::{Client, HttpClient};
use tonic::Status;

// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
//use tracing_futures::Instrument;

// Note: the conversions that take place in here could be moved to
// from/try_from impls, but they're not used anywhere else, so it's
// unimportant right now, and would require additional wrappers
// since none of the structs are defined in our crates :(
// TODO: move those to proto/src/protobuf.rs

#[tonic::async_trait]
impl TendermintProxyService for TendermintProxy {
    async fn get_tx(
        &self,
        req: tonic::Request<GetTxRequest>,
    ) -> Result<tonic::Response<GetTxResponse>, Status> {
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let req = req.into_inner();
        let hash = req.hash;
        let prove = req.prove;
        let rsp = client
            .tx(
                hash.try_into().map_err(|e| {
                    tonic::Status::invalid_argument(format!("invalid transaction hash: {e:#?}"))
                })?,
                prove,
            )
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error getting tx: {e}")))?;

        let tx = Transaction::decode(rsp.tx.as_ref())
            .map_err(|e| tonic::Status::unavailable(format!("error decoding tx: {e}")))?;

        Ok(tonic::Response::new(GetTxResponse {
            tx: tx.into(),
            tx_result: Some(proto::client::v1alpha1::TxResult {
                log: rsp.tx_result.log.to_string(),
                // TODO: validation here, fix mismatch between i64 <> u64
                gas_wanted: rsp.tx_result.gas_wanted as u64,
                gas_used: rsp.tx_result.gas_used as u64,
                tags: rsp
                    .tx_result
                    .events
                    .iter()
                    .flat_map(|e| {
                        let a = &e.attributes;
                        a.iter().map(move |a| {
                            proto::client::v1alpha1::Tag {
                                key: a.key.to_string().as_bytes().to_vec(),
                                value: a.value.to_string().as_bytes().to_vec(),
                                // TODO: not sure where this index value comes from
                                index: false,
                            }
                        })
                    })
                    .collect(),
            }),
            height: rsp.height.value(),
            index: rsp.index as u64,
            hash: rsp.hash.as_bytes().to_vec(),
        }))
    }

    async fn broadcast_tx_async(
        &self,
        req: tonic::Request<BroadcastTxAsyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxAsyncResponse>, Status> {
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let params = req.into_inner().params;

        let res = client
            .broadcast_tx_async(params)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error broadcasting tx async: {e}")))?;

        Ok(tonic::Response::new(BroadcastTxAsyncResponse {
            code: u32::from(res.code) as u64,
            data: res.data.to_vec(),
            log: res.log.to_string(),
            hash: res.hash.as_bytes().to_vec(),
        }))
    }

    async fn broadcast_tx_sync(
        &self,
        req: tonic::Request<BroadcastTxSyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxSyncResponse>, Status> {
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let res = client
            .broadcast_tx_sync(req.into_inner().params)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error broadcasting tx sync: {e}")))?;

        tracing::debug!("{:?}", res);
        Ok(tonic::Response::new(BroadcastTxSyncResponse {
            code: u32::from(res.code) as u64,
            data: res.data.to_vec(),
            log: res.log.to_string(),
            hash: res.hash.as_bytes().to_vec(),
        }))
    }

    async fn get_status(
        &self,
        _req: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

        let res = client
            .status()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error querying status: {e}")))?;

        // The tendermint-rs `Timestamp` type is a newtype wrapper
        // around a `time::PrimitiveDateTime` however it's private so we
        // have to use string parsing to get to the prost type we want :(
        let latest_block_time =
            DateTime::parse_from_rfc3339(&res.sync_info.latest_block_time.to_rfc3339())
                .expect("timestamp should roundtrip to string");
        Ok(tonic::Response::new(GetStatusResponse {
            node_info: Some(penumbra_proto::tendermint::p2p::DefaultNodeInfo {
                protocol_version: Some(penumbra_proto::tendermint::p2p::ProtocolVersion {
                    p2p: res.node_info.protocol_version.p2p,
                    block: res.node_info.protocol_version.block,
                    app: res.node_info.protocol_version.app,
                }),
                default_node_id: res.node_info.id.to_string(),
                listen_addr: res.node_info.listen_addr.to_string(),
                network: res.node_info.network.to_string(),
                version: res.node_info.version.to_string(),
                channels: res.node_info.channels.to_string().as_bytes().to_vec(),
                moniker: res.node_info.moniker.to_string(),
                other: Some(penumbra_proto::tendermint::p2p::DefaultNodeInfoOther {
                    tx_index: match res.node_info.other.tx_index {
                        tendermint::node::info::TxIndexStatus::On => "on".to_string(),
                        tendermint::node::info::TxIndexStatus::Off => "off".to_string(),
                    },
                    rpc_address: res.node_info.other.rpc_address.to_string(),
                }),
            }),
            sync_info: Some(penumbra_proto::client::v1alpha1::SyncInfo {
                latest_block_hash: res
                    .sync_info
                    .latest_block_hash
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                latest_app_hash: res
                    .sync_info
                    .latest_app_hash
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                latest_block_height: res.sync_info.latest_block_height.value(),
                latest_block_time: Some(pbjson_types::Timestamp {
                    seconds: latest_block_time.timestamp(),
                    nanos: latest_block_time.timestamp_nanos() as i32,
                }),
                // These don't exist in tendermint-rpc right now.
                // earliest_app_hash: res.sync_info.earliest_app_hash.to_string().as_bytes().to_vec(),
                // earliest_block_hash: res.sync_info.earliest_block_hash.to_string().as_bytes().to_vec(),
                // earliest_block_height: res.sync_info.earliest_block_height.value(),
                // earliest_block_time: Some(pbjson_types::Timestamp{
                //     seconds: earliest_block_time.timestamp(),
                //     nanos: earliest_block_time.timestamp_nanos() as i32,
                // }),
                catching_up: res.sync_info.catching_up,
            }),
            validator_info: Some(penumbra_proto::tendermint::types::Validator {
                address: res.validator_info.address.to_string().as_bytes().to_vec(),
                pub_key: Some(penumbra_proto::tendermint::crypto::PublicKey {
                    sum: Some(
                        penumbra_proto::tendermint::crypto::public_key::Sum::Ed25519(
                            res.validator_info.pub_key.to_bytes().to_vec(),
                        ),
                    ),
                }),
                voting_power: res.validator_info.power.into(),
                proposer_priority: res.validator_info.proposer_priority.into(),
            }),
        }))
    }

    async fn abci_query(
        &self,
        req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).unwrap();

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
            .map_err(|e| tonic::Status::unavailable(format!("error querying abci: {e}")))?;

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
                        nanos: header_time.timestamp_nanos() as i32,
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
                        .map(|e| proto::tendermint::types::Evidence {
                            sum: Some( match e {
                                tendermint::evidence::Evidence::DuplicateVote(e) => {
                                   let e2 = tendermint_proto::types::DuplicateVoteEvidence::from(e.clone());
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
                                            nanos: DateTime::parse_from_rfc3339(&e.votes().0.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
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
                                            nanos: DateTime::parse_from_rfc3339(&e.votes().1.timestamp.expect("timestamp").to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
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
                                // This variant is currently unimplemented in tendermint-rs, so we can't supply
                                // conversions for it.
                                tendermint::evidence::Evidence::LightClientAttackEvidence => proto::tendermint::types::evidence::Sum::LightClientAttackEvidence(proto::tendermint::types::LightClientAttackEvidence{
                                    conflicting_block: None,
                                    common_height: -1,
                                    byzantine_validators: vec![],
                                    timestamp: None,
                                    total_voting_power: -1,
                                }),
                        }),
                        })
                        .collect(),
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
                            .map(|s| {
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
                                            nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339()).expect("timestamp should roundtrip to string").timestamp_nanos() as i32,
                                        }),
                                        signature: signature.expect("signature").into(),
                                    },
                                    tendermint::block::CommitSig::BlockIdFlagNil { validator_address, timestamp, signature } => proto::tendermint::types::CommitSig {
                                        block_id_flag: proto::tendermint::types::BlockIdFlag::Nil as i32,
                                        validator_address: validator_address.into(),
                                        timestamp: Some(pbjson_types::Timestamp{
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
        }))
    }
}

/// Implements service traits for Tonic gRPC services.
///
/// The fields of this struct are the configuration and data
/// necessary to the gRPC services.
#[derive(Clone, Debug)]
pub struct TendermintProxy {
    /// Address of upstream Tendermint server to proxy requests to.
    tendermint_url: url::Url,
}

impl TendermintProxy {
    pub fn new(tendermint_url: url::Url) -> Self {
        Self { tendermint_url }
    }
}
