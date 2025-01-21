//! Facilities to help interoperate with [`tendermint`] and [`tendermint_rpc`] types.
//
//  NOTE: this submodule is tightly focused on helping `penumbra-tendermint-proxy` function.
//  this is not an exhaustive pass at providing compatibility between all of the types in either
//  library. accordingly, it is grouped by conversions needed for each RPC endpoint.

use crate::util::tendermint_proxy::v1 as penumbra_sdk_pb;
use anyhow::anyhow;

// === get_tx ===

impl From<tendermint_rpc::endpoint::tx::Response> for penumbra_sdk_pb::GetTxResponse {
    fn from(
        tendermint_rpc::endpoint::tx::Response {
            hash,
            height,
            index,
            tx_result,
            tx,
            proof: _,
        }: tendermint_rpc::endpoint::tx::Response,
    ) -> Self {
        Self {
            height: height.value(),
            index: index as u64,
            hash: hash.as_bytes().to_vec(),
            tx_result: Some(tx_result.into()),
            tx,
        }
    }
}

impl From<tendermint::abci::types::ExecTxResult> for penumbra_sdk_pb::TxResult {
    fn from(
        tendermint::abci::types::ExecTxResult {
            log,
            gas_wanted,
            gas_used,
            events,
            code: _,
            data: _,
            info: _,
            codespace: _,
        }: tendermint::abci::types::ExecTxResult,
    ) -> Self {
        use tendermint::abci::Event;
        Self {
            log: log.to_string(),
            // TODO: validation here, fix mismatch between i64 <> u64
            gas_wanted: gas_wanted as u64,
            gas_used: gas_used as u64,
            tags: events
                .into_iter()
                .flat_map(|Event { attributes, .. }: Event| {
                    attributes.into_iter().map(penumbra_sdk_pb::Tag::from)
                })
                .collect(),
        }
    }
}

impl From<tendermint::abci::EventAttribute> for penumbra_sdk_pb::Tag {
    fn from(event_attr: tendermint::abci::EventAttribute) -> Self {
        Self {
            key: event_attr.key_bytes().into(),
            value: event_attr.value_bytes().into(),
            // TODO(kate): this was set to false previously, but it should probably use the
            // index field from the tendermint object. for now, carry out a refactor and avoid
            // changing behavior while doing so.
            index: false,
        }
    }
}

// impl From<tendermint::abci::event::v0_37::EventAttribute> for penumbra_sdk_pb::Tag {
//     fn from(
//         tendermint::abci::event::v0_37::EventAttribute {
//             key,
//             value,
//             index: _,
//         }: tendermint::abci::EventAttribute,
//     ) -> Self {
//         Self {
//             key: key.into_bytes(),
//             value: value.into_bytes(),
//             // TODO(kate): this was set to false previously, but it should probably use the
//             // index field from the tendermint object. for now, carry out a refactor and avoid
//             // changing behavior while doing so.
//             index: false,
//         }
//     }
// }

// === broadcast_tx_async ===

impl From<tendermint_rpc::endpoint::broadcast::tx_async::Response>
    for penumbra_sdk_pb::BroadcastTxAsyncResponse
{
    fn from(
        tendermint_rpc::endpoint::broadcast::tx_async::Response {
            code,
            data,
            log,
            hash,
            ..
        }: tendermint_rpc::endpoint::broadcast::tx_async::Response,
    ) -> Self {
        Self {
            code: u32::from(code) as u64,
            data: data.to_vec(),
            log,
            hash: hash.as_bytes().to_vec(),
        }
    }
}

// === broadcast_tx_sync ===

impl From<tendermint_rpc::endpoint::broadcast::tx_sync::Response>
    for penumbra_sdk_pb::BroadcastTxSyncResponse
{
    fn from(
        tendermint_rpc::endpoint::broadcast::tx_sync::Response {
            code,
            data,
            log,
            hash,
            ..
        }: tendermint_rpc::endpoint::broadcast::tx_sync::Response,
    ) -> Self {
        Self {
            code: u32::from(code) as u64,
            data: data.to_vec(),
            log,
            hash: hash.as_bytes().to_vec(),
        }
    }
}

// === get_status ===

impl From<tendermint_rpc::endpoint::status::Response> for penumbra_sdk_pb::GetStatusResponse {
    fn from(
        tendermint_rpc::endpoint::status::Response {
            node_info,
            sync_info,
            validator_info,
        }: tendermint_rpc::endpoint::status::Response,
    ) -> Self {
        Self {
            node_info: Some(node_info.into()),
            sync_info: Some(sync_info.into()),
            validator_info: Some(validator_info.into()),
        }
    }
}

impl From<tendermint::node::Info> for crate::tendermint::p2p::DefaultNodeInfo {
    fn from(
        tendermint::node::Info {
            protocol_version,
            id,
            listen_addr,
            network,
            version,
            channels,
            moniker,
            other,
        }: tendermint::node::Info,
    ) -> Self {
        Self {
            protocol_version: Some(protocol_version.into()),
            default_node_id: id.to_string(),
            listen_addr: listen_addr.to_string(),
            network: network.to_string(),
            version: version.to_string(),
            channels: channels.to_string().as_bytes().to_vec(),
            moniker: moniker.to_string(),
            other: Some(crate::tendermint::p2p::DefaultNodeInfoOther {
                tx_index: match other.tx_index {
                    tendermint::node::info::TxIndexStatus::On => "on".to_string(),
                    tendermint::node::info::TxIndexStatus::Off => "off".to_string(),
                },
                rpc_address: other.rpc_address.to_string(),
            }),
        }
    }
}

impl From<tendermint_rpc::endpoint::status::SyncInfo> for penumbra_sdk_pb::SyncInfo {
    fn from(
        tendermint_rpc::endpoint::status::SyncInfo {
            latest_block_hash,
            latest_app_hash,
            latest_block_height,
            latest_block_time,
            catching_up,
            earliest_block_hash: _,
            earliest_app_hash: _,
            earliest_block_height: _,
            earliest_block_time: _,
        }: tendermint_rpc::endpoint::status::SyncInfo,
    ) -> Self {
        // The tendermint-rs `Timestamp` type is a newtype wrapper
        // around a `time::PrimitiveDateTime` however it's private so we
        // have to use string parsing to get to the prost type we want :(
        let latest_block_time =
            chrono::DateTime::parse_from_rfc3339(latest_block_time.to_rfc3339().as_str())
                .expect("timestamp should roundtrip to string");

        Self {
            latest_block_hash: latest_block_hash.to_string().as_bytes().to_vec(),
            latest_app_hash: latest_app_hash.to_string().as_bytes().to_vec(),
            latest_block_height: latest_block_height.value(),
            latest_block_time: Some(pbjson_types::Timestamp {
                seconds: latest_block_time.timestamp(),
                nanos: latest_block_time.timestamp_subsec_nanos() as i32,
            }),
            catching_up,
            // These don't exist in tendermint-rpc right now.
            // earliest_app_hash: res.sync_info.earliest_app_hash.to_string().as_bytes().to_vec(),
            // earliest_block_hash: res.sync_info.earliest_block_hash.to_string().as_bytes().to_vec(),
            // earliest_block_height: res.sync_info.earliest_block_height.value(),
            // earliest_block_time: Some(pbjson_types::Timestamp{
            //     seconds: earliest_block_time.timestamp(),
            //     nanos: earliest_block_time.timestamp_nanos() as i32,
            // }),
        }
    }
}

impl From<tendermint::validator::Info> for crate::tendermint::types::Validator {
    fn from(
        tendermint::validator::Info {
            address,
            pub_key,
            power,
            proposer_priority,
            name: _,
        }: tendermint::validator::Info,
    ) -> Self {
        use crate::tendermint::crypto::{public_key::Sum::Ed25519, PublicKey};
        Self {
            address: address.to_string().as_bytes().to_vec(),
            pub_key: Some(PublicKey {
                sum: Some(Ed25519(pub_key.to_bytes().to_vec())),
            }),
            voting_power: power.into(),
            proposer_priority: proposer_priority.into(),
        }
    }
}

impl From<tendermint::node::info::ProtocolVersionInfo> for crate::tendermint::p2p::ProtocolVersion {
    fn from(
        tendermint::node::info::ProtocolVersionInfo {
                p2p,
                block,
                app
            }: tendermint::node::info::ProtocolVersionInfo,
    ) -> Self {
        Self { p2p, block, app }
    }
}

// === abci_query ===

#[derive(Debug, thiserror::Error)]
#[error("height '{height}' from tendermint overflowed i64, this should never happen")]
pub struct HeightOverflowError {
    height: u64,
    #[source]
    source: <i64 as TryFrom<u64>>::Error,
}

impl TryFrom<tendermint_rpc::endpoint::abci_query::AbciQuery>
    for penumbra_sdk_pb::AbciQueryResponse
{
    type Error = HeightOverflowError;
    fn try_from(
        tendermint_rpc::endpoint::abci_query::AbciQuery {
            code,
            log,
            info,
            index,
            key,
            value,
            proof,
            height,
            codespace,
        }: tendermint_rpc::endpoint::abci_query::AbciQuery,
    ) -> Result<Self, Self::Error> {
        let proof_ops = proof.map(crate::tendermint::crypto::ProofOps::from);
        let height = i64::try_from(height.value()).map_err(|source| HeightOverflowError {
            height: height.value(),
            source,
        })?;
        Ok(Self {
            code: u32::from(code),
            log,
            info,
            index,
            key,
            value,
            proof_ops,
            height,
            codespace,
        })
    }
}

impl From<tendermint::merkle::proof::ProofOps> for crate::tendermint::crypto::ProofOps {
    fn from(
        tendermint::merkle::proof::ProofOps { ops }: tendermint::merkle::proof::ProofOps,
    ) -> Self {
        Self {
            ops: ops
                .into_iter()
                .map(crate::tendermint::crypto::ProofOp::from)
                .collect(),
        }
    }
}

impl From<tendermint::merkle::proof::ProofOp> for crate::tendermint::crypto::ProofOp {
    fn from(
        tendermint::merkle::proof::ProofOp {
            field_type,
            key,
            data,
        }: tendermint::merkle::proof::ProofOp,
    ) -> Self {
        Self {
            r#type: field_type,
            key,
            data,
        }
    }
}

// === get_block_by_height ===

impl TryFrom<tendermint_rpc::endpoint::block::Response>
    for penumbra_sdk_pb::GetBlockByHeightResponse
{
    type Error = anyhow::Error;
    fn try_from(
        tendermint_rpc::endpoint::block::Response {
                block,
                block_id,
            }: tendermint_rpc::endpoint::block::Response,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            block: block.try_into().map(Some)?,
            block_id: Some(block_id.into()),
        })
    }
}
impl TryFrom<tendermint::Block> for crate::tendermint::types::Block {
    type Error = anyhow::Error;
    fn try_from(
        tendermint::Block {
            header,
            data,
            evidence,
            last_commit,
            ..
        }: tendermint::Block,
    ) -> Result<Self, Self::Error> {
        Ok(crate::tendermint::types::Block {
            header: header.try_into().map(Some)?,
            data: Some(crate::tendermint::types::Data { txs: data }),
            evidence: evidence.try_into().map(Some)?,
            last_commit: last_commit
                .map(crate::tendermint::types::Commit::try_from)
                .transpose()?,
        })
    }
}

impl TryFrom<crate::tendermint::types::PartSetHeader> for tendermint::block::parts::Header {
    type Error = anyhow::Error;
    fn try_from(
        crate::tendermint::types::PartSetHeader { total, hash }: crate::tendermint::types::PartSetHeader,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(total, hash.try_into()?)?)
    }
}

impl TryFrom<crate::tendermint::types::Header> for tendermint::block::Header {
    type Error = anyhow::Error;
    fn try_from(
        crate::tendermint::types::Header {
            version,
            chain_id,
            height,
            time,
            last_block_id,
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        }: crate::tendermint::types::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            version: tendermint::block::header::Version {
                block: version.clone().ok_or(anyhow!("version"))?.block,
                app: version.ok_or(anyhow!("version"))?.app,
            },
            chain_id: tendermint::chain::Id::try_from(chain_id)?,
            height: tendermint::block::Height::try_from(height)?,
            time: tendermint::Time::from_unix_timestamp(
                time.clone().ok_or(anyhow!("time"))?.seconds,
                time.clone()
                    .ok_or(anyhow!("missing time"))?
                    .nanos
                    .try_into()?,
            )?,
            last_block_id: match last_block_id {
                Some(last_block_id) => Some(tendermint::block::Id {
                    hash: tendermint::Hash::try_from(last_block_id.hash)?,
                    part_set_header: tendermint::block::parts::Header::try_from(
                        last_block_id
                            .part_set_header
                            .ok_or(anyhow::anyhow!("bad part set header"))?,
                    )?,
                }),
                None => None,
            },
            last_commit_hash: Some(last_commit_hash.try_into()?),
            data_hash: Some(data_hash.try_into()?),
            validators_hash: validators_hash.try_into()?,
            next_validators_hash: next_validators_hash.try_into()?,
            consensus_hash: consensus_hash.try_into()?,
            app_hash: app_hash.try_into()?,
            last_results_hash: Some(last_results_hash.try_into()?),
            evidence_hash: Some(evidence_hash.try_into()?),
            proposer_address: proposer_address.try_into()?,
        })
    }
}

impl TryFrom<tendermint::block::Header> for crate::tendermint::types::Header {
    // TODO(kate): ideally this would not return a tonic status object, but we'll use this for
    // now to avoid invasively refactoring this code.
    type Error = tonic::Status;
    fn try_from(
        tendermint::block::Header {
            version,
            chain_id,
            height,
            time,
            last_block_id,
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        }: tendermint::block::Header,
    ) -> Result<Self, Self::Error> {
        // The tendermint-rs `Timestamp` type is a newtype wrapper
        // around a `time::PrimitiveDateTime` however it's private so we
        // have to use string parsing to get to the prost type we want :(
        let header_time = chrono::DateTime::parse_from_rfc3339(time.to_rfc3339().as_str())
            .or_else(|_| {
                Err(tonic::Status::invalid_argument(
                    "timestamp should roundtrip to string",
                ))
            })?;
        Ok(Self {
            version: Some(crate::tendermint::version::Consensus {
                block: version.block,
                app: version.app,
            }),
            chain_id: chain_id.into(),
            height: height.into(),
            time: Some(pbjson_types::Timestamp {
                seconds: header_time.timestamp(),
                nanos: header_time.timestamp_subsec_nanos() as i32,
            }),
            last_block_id: last_block_id.map(|id| crate::tendermint::types::BlockId {
                hash: id.hash.into(),
                part_set_header: Some(crate::tendermint::types::PartSetHeader {
                    total: id.part_set_header.total,
                    hash: id.part_set_header.hash.into(),
                }),
            }),
            last_commit_hash: last_commit_hash.map(Into::into).unwrap_or_default(),
            data_hash: data_hash.map(Into::into).unwrap_or_default(),
            validators_hash: validators_hash.into(),
            next_validators_hash: next_validators_hash.into(),
            consensus_hash: consensus_hash.into(),
            app_hash: app_hash.into(),
            last_results_hash: last_results_hash.map(Into::into).unwrap_or_default(),
            evidence_hash: evidence_hash.map(Into::into).unwrap_or_default(),
            proposer_address: proposer_address.into(),
        })
    }
}

impl TryFrom<tendermint::evidence::List> for crate::tendermint::types::EvidenceList {
    type Error = anyhow::Error;
    fn try_from(list: tendermint::evidence::List) -> Result<Self, Self::Error> {
        list.into_vec()
            .into_iter()
            .map(crate::tendermint::types::Evidence::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(|evidence| Self { evidence })
    }
}

// TODO(kate): this should be decomposed further at a later point, i am refraining from doing
// so right now.
impl TryFrom<tendermint::evidence::Evidence> for crate::tendermint::types::Evidence {
    type Error = anyhow::Error;
    fn try_from(evidence: tendermint::evidence::Evidence) -> Result<Self, Self::Error> {
        use {chrono::DateTime, std::ops::Deref};
        Ok(Self {
            sum: Some(match evidence {
                tendermint::evidence::Evidence::DuplicateVote(e) => {
                    let e2 =
                        tendermint_proto::types::DuplicateVoteEvidence::from(e.deref().clone());
                    crate::tendermint::types::evidence::Sum::DuplicateVoteEvidence(
                        crate::tendermint::types::DuplicateVoteEvidence {
                            vote_a: Some(crate::tendermint::types::Vote {
                                r#type: match e.votes().0.vote_type {
                                    tendermint::vote::Type::Prevote => {
                                        crate::tendermint::types::SignedMsgType::Prevote as i32
                                    }
                                    tendermint::vote::Type::Precommit => {
                                        crate::tendermint::types::SignedMsgType::Precommit as i32
                                    }
                                },
                                height: e.votes().0.height.into(),
                                round: e.votes().0.round.into(),
                                block_id: Some(crate::tendermint::types::BlockId {
                                    hash: e
                                        .votes()
                                        .0
                                        .block_id
                                        .ok_or(anyhow!("block id"))?
                                        .hash
                                        .into(),
                                    part_set_header: Some(
                                        crate::tendermint::types::PartSetHeader {
                                            total: e
                                                .votes()
                                                .0
                                                .block_id
                                                .ok_or(anyhow!("block id"))?
                                                .part_set_header
                                                .total,
                                            hash: e
                                                .votes()
                                                .0
                                                .block_id
                                                .ok_or(anyhow!("block id"))?
                                                .part_set_header
                                                .hash
                                                .into(),
                                        },
                                    ),
                                }),
                                timestamp: Some(pbjson_types::Timestamp {
                                    seconds: DateTime::parse_from_rfc3339(
                                        &e.votes()
                                            .0
                                            .timestamp
                                            .ok_or(tonic::Status::invalid_argument(
                                                "bad timestamp",
                                            ))?
                                            .to_rfc3339(),
                                    )
                                    .or_else(|_| {
                                        Err(tonic::Status::invalid_argument("bad timestamp"))
                                    })?
                                    .timestamp(),
                                    nanos: DateTime::parse_from_rfc3339(
                                        &e.votes().0.timestamp.expect("timestamp").to_rfc3339(),
                                    )
                                    .expect("timestamp should roundtrip to string")
                                    .timestamp_subsec_nanos()
                                    .try_into()
                                    .expect("good round trip timestamps"),
                                }),
                                validator_address: e.votes().0.validator_address.into(),
                                validator_index: e.votes().0.validator_index.into(),
                                signature: e
                                    .votes()
                                    .0
                                    .signature
                                    .clone()
                                    .expect("signed vote")
                                    .into(),
                            }),
                            vote_b: Some(crate::tendermint::types::Vote {
                                r#type: match e.votes().1.vote_type {
                                    tendermint::vote::Type::Prevote => {
                                        crate::tendermint::types::SignedMsgType::Prevote as i32
                                    }
                                    tendermint::vote::Type::Precommit => {
                                        crate::tendermint::types::SignedMsgType::Precommit as i32
                                    }
                                },
                                height: e.votes().1.height.into(),
                                round: e.votes().1.round.into(),
                                block_id: Some(crate::tendermint::types::BlockId {
                                    hash: e.votes().1.block_id.expect("block id").hash.into(),
                                    part_set_header: Some(
                                        crate::tendermint::types::PartSetHeader {
                                            total: e
                                                .votes()
                                                .1
                                                .block_id
                                                .expect("block id")
                                                .part_set_header
                                                .total,
                                            hash: e
                                                .votes()
                                                .1
                                                .block_id
                                                .expect("block id")
                                                .part_set_header
                                                .hash
                                                .into(),
                                        },
                                    ),
                                }),
                                timestamp: Some(pbjson_types::Timestamp {
                                    seconds: DateTime::parse_from_rfc3339(
                                        &e.votes().1.timestamp.expect("timestamp").to_rfc3339(),
                                    )
                                    .expect("timestamp should roundtrip to string")
                                    .timestamp(),
                                    nanos: DateTime::parse_from_rfc3339(
                                        &e.votes().1.timestamp.expect("timestamp").to_rfc3339(),
                                    )
                                    .expect("timestamp should roundtrip to string")
                                    .timestamp_subsec_nanos()
                                    .try_into()
                                    .expect("good round trip timestamps"),
                                }),
                                validator_address: e.votes().1.validator_address.into(),
                                validator_index: e.votes().1.validator_index.into(),
                                signature: e
                                    .votes()
                                    .1
                                    .signature
                                    .clone()
                                    .expect("signed vote")
                                    .into(),
                            }),
                            total_voting_power: e2.total_voting_power,
                            validator_power: e2.validator_power,
                            timestamp: e2.timestamp.map(|t| pbjson_types::Timestamp {
                                seconds: t.seconds,
                                nanos: t.nanos,
                            }),
                        },
                    )
                }
                tendermint::evidence::Evidence::LightClientAttack(e) => {
                    use crate::Message;
                    let e2 =
                        tendermint_proto::types::LightClientAttackEvidence::from(e.deref().clone());
                    let e2_bytes = e2.encode_to_vec();
                    let e3 = crate::tendermint::types::LightClientAttackEvidence::decode(
                        e2_bytes.as_slice(),
                    )
                    .expect("can decode encoded data");
                    crate::tendermint::types::evidence::Sum::LightClientAttackEvidence(e3)
                }
            }),
        })
    }
}

impl TryFrom<tendermint::block::Commit> for crate::tendermint::types::Commit {
    // TODO(kate): ideally this would not return a tonic status object, but we'll use this for
    // now to avoid invasively refactoring this code.
    type Error = tonic::Status;
    fn try_from(
        tendermint::block::Commit {
            height,
            round,
            block_id,
            signatures,
        }: tendermint::block::Commit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            height: height.into(),
            round: round.into(),
            block_id: Some(block_id.into()),
            signatures: signatures
                .into_iter()
                .map(crate::tendermint::types::CommitSig::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<tendermint::block::CommitSig> for crate::tendermint::types::CommitSig {
    // TODO(kate): ideally this would not return a tonic status object, but we'll use this for
    // now to avoid invasively refactoring this code.
    type Error = tonic::Status;
    fn try_from(signature: tendermint::block::CommitSig) -> Result<Self, Self::Error> {
        use chrono::DateTime;
        Ok({
            match signature {
                tendermint::block::CommitSig::BlockIdFlagAbsent => {
                    crate::tendermint::types::CommitSig {
                        block_id_flag: crate::tendermint::types::BlockIdFlag::Absent as i32,
                        // No validator address, or timestamp is recorded for this variant. Not sure if this is a bug in tendermint-rs or not.
                        validator_address: vec![],
                        timestamp: None,
                        signature: vec![],
                    }
                }
                tendermint::block::CommitSig::BlockIdFlagCommit {
                    validator_address,
                    timestamp,
                    signature,
                } => crate::tendermint::types::CommitSig {
                    block_id_flag: crate::tendermint::types::BlockIdFlag::Commit as i32,
                    validator_address: validator_address.into(),
                    timestamp: Some(pbjson_types::Timestamp {
                        seconds: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339())
                            .expect("timestamp should roundtrip to string")
                            .timestamp(),
                        nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339())
                            .expect("timestamp should roundtrip to string")
                            .timestamp_subsec_nanos()
                            .try_into()
                            .expect("good round trip timestamps"),
                    }),
                    signature: signature.map(Into::into).unwrap_or_default(),
                },
                tendermint::block::CommitSig::BlockIdFlagNil {
                    validator_address,
                    timestamp,
                    signature,
                } => crate::tendermint::types::CommitSig {
                    block_id_flag: crate::tendermint::types::BlockIdFlag::Nil as i32,
                    validator_address: validator_address.into(),
                    timestamp: Some(pbjson_types::Timestamp {
                        seconds: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339())
                            .expect("timestamp should roundtrip to string")
                            .timestamp(),
                        nanos: DateTime::parse_from_rfc3339(&timestamp.to_rfc3339())
                            .expect("timestamp should roundtrip to string")
                            .timestamp_subsec_nanos()
                            .try_into()
                            .expect("good round trip timestamps"),
                    }),
                    signature: signature.expect("signature").into(),
                },
            }
        })
    }
}

impl From<tendermint::block::Id> for crate::tendermint::types::BlockId {
    fn from(
        tendermint::block::Id {
            hash,
            part_set_header,
        }: tendermint::block::Id,
    ) -> Self {
        Self {
            hash: hash.into(),
            part_set_header: Some(part_set_header.into()),
        }
    }
}

impl From<tendermint::block::parts::Header> for crate::tendermint::types::PartSetHeader {
    fn from(
        tendermint::block::parts::Header { total, hash, .. }: tendermint::block::parts::Header,
    ) -> Self {
        Self {
            total,
            hash: hash.into(),
        }
    }
}
