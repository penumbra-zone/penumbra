use crate::Name;
use std::convert::{From, TryFrom};

/// A marker type that captures the relationships between a domain type (`Self`) and a protobuf type (`Self::Proto`).
pub trait DomainType
where
    Self: Clone + Sized + TryFrom<Self::Proto>,
    Self::Proto: prost::Name + prost::Message + Default + From<Self> + Send + Sync + 'static,
    anyhow::Error: From<<Self as TryFrom<Self::Proto>>::Error>,
{
    type Proto;

    /// Encode this domain type to a byte vector, via proto type `P`.
    fn encode_to_vec(&self) -> Vec<u8> {
        use prost::Message;
        self.to_proto().encode_to_vec()
    }

    /// Convert this domain type to the associated proto type.
    ///
    /// This uses the `From` impl internally, so it works exactly
    /// like `.into()`, but does not require type inference.
    fn to_proto(&self) -> Self::Proto {
        Self::Proto::from(self.clone())
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> anyhow::Result<Self> {
        <Self::Proto as prost::Message>::decode(buf)?
            .try_into()
            .map_err(Into::into)
    }
}

// Implementations on foreign types.
//
// This should only be done here in cases where the domain type lives in a crate
// that shouldn't depend on the Penumbra proto framework.

use crate::penumbra::core::component::ibc::v1::IbcRelay;
use crate::penumbra::crypto::decaf377_rdsa::v1::{
    BindingSignature, SpendAuthSignature, SpendVerificationKey,
};

use decaf377_rdsa::{Binding, Signature, SpendAuth, VerificationKey};

impl DomainType for Signature<SpendAuth> {
    type Proto = SpendAuthSignature;
}
impl DomainType for Signature<Binding> {
    type Proto = BindingSignature;
}
impl DomainType for VerificationKey<SpendAuth> {
    type Proto = SpendVerificationKey;
}

impl From<Signature<SpendAuth>> for SpendAuthSignature {
    fn from(sig: Signature<SpendAuth>) -> Self {
        Self {
            inner: sig.to_bytes().to_vec(),
        }
    }
}

impl From<Signature<Binding>> for BindingSignature {
    fn from(sig: Signature<Binding>) -> Self {
        Self {
            inner: sig.to_bytes().to_vec(),
        }
    }
}

impl From<VerificationKey<SpendAuth>> for SpendVerificationKey {
    fn from(key: VerificationKey<SpendAuth>) -> Self {
        Self {
            inner: key.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<SpendAuthSignature> for Signature<SpendAuth> {
    type Error = anyhow::Error;
    fn try_from(value: SpendAuthSignature) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

impl TryFrom<BindingSignature> for Signature<Binding> {
    type Error = anyhow::Error;
    fn try_from(value: BindingSignature) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

impl TryFrom<SpendVerificationKey> for VerificationKey<SpendAuth> {
    type Error = anyhow::Error;
    fn try_from(value: SpendVerificationKey) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

// Fuzzy Message Detection
use crate::penumbra::crypto::decaf377_fmd::v1::Clue as ProtoClue;
use decaf377_fmd::Clue;

impl DomainType for Clue {
    type Proto = ProtoClue;
}

impl From<Clue> for ProtoClue {
    fn from(msg: Clue) -> Self {
        ProtoClue { inner: msg.into() }
    }
}

impl TryFrom<ProtoClue> for Clue {
    type Error = anyhow::Error;

    fn try_from(proto: ProtoClue) -> Result<Self, Self::Error> {
        proto.inner[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected 68-byte clue"))
    }
}

// Consensus key
//
// The tendermint-rs PublicKey type already has a tendermint-proto type;
// this redefines its proto, because the encodings are consensus-critical
// and we don't vendor all of the tendermint protos.
use crate::penumbra::core::keys::v1::ConsensusKey;

impl DomainType for tendermint::PublicKey {
    type Proto = ConsensusKey;
}

impl From<tendermint::PublicKey> for crate::penumbra::core::keys::v1::ConsensusKey {
    fn from(v: tendermint::PublicKey) -> Self {
        Self {
            inner: v.to_bytes(),
        }
    }
}

impl TryFrom<crate::core::keys::v1::ConsensusKey> for tendermint::PublicKey {
    type Error = anyhow::Error;
    fn try_from(value: crate::core::keys::v1::ConsensusKey) -> Result<Self, Self::Error> {
        Self::from_raw_ed25519(value.inner.as_slice())
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 key"))
    }
}

// IBC-rs impls
extern crate ibc_types;

use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::core::connection::v1::ClientPaths as RawClientPaths;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;

use ibc_types::core::channel::ChannelEnd;
use ibc_types::core::client::Height;
use ibc_types::core::connection::{ClientPaths, ConnectionEnd};
use ibc_types::lightclients::tendermint::client_state::ClientState;
use ibc_types::lightclients::tendermint::consensus_state::ConsensusState;

impl DomainType for ClientPaths {
    type Proto = RawClientPaths;
}

impl DomainType for ConnectionEnd {
    type Proto = RawConnectionEnd;
}

impl DomainType for ChannelEnd {
    type Proto = RawChannel;
}
impl DomainType for Height {
    type Proto = RawHeight;
}

impl DomainType for ClientState {
    type Proto = ibc_proto::google::protobuf::Any;
}
impl DomainType for ConsensusState {
    type Proto = ibc_proto::google::protobuf::Any;
}

impl<T> From<T> for IbcRelay
where
    T: ibc_types::DomainType + Send + Sync + 'static,
    <T as TryFrom<<T as ibc_types::DomainType>::Proto>>::Error: Send + Sync + std::error::Error,
{
    fn from(v: T) -> Self {
        let value_bytes = v.encode_to_vec();
        let any = pbjson_types::Any {
            type_url: <T as ibc_types::DomainType>::Proto::type_url(),
            value: value_bytes.into(),
        };

        Self {
            raw_action: Some(any),
        }
    }
}

mod tendermint_rpc_compat {
    use crate::util::tendermint_proxy::v1 as penumbra_pb;

    // === get_tx ===

    impl From<tendermint_rpc::endpoint::tx::Response> for penumbra_pb::GetTxResponse {
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

    impl From<tendermint::abci::types::ExecTxResult> for penumbra_pb::TxResult {
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
                        attributes.into_iter().map(penumbra_pb::Tag::from)
                    })
                    .collect(),
            }
        }
    }

    impl From<tendermint::abci::EventAttribute> for penumbra_pb::Tag {
        fn from(
            tendermint::abci::EventAttribute {
                key,
                value,
                index: _,
            }: tendermint::abci::EventAttribute,
        ) -> Self {
            Self {
                key: key.into_bytes(),
                value: value.into_bytes(),
                // TODO(kate): this was set to false previously, but it should probably use the
                // index field from the tendermint object. for now, carry out a refactor and avoid
                // changing behavior while doing so.
                index: false,
            }
        }
    }

    // === broadcast_tx_async ===

    impl From<tendermint_rpc::endpoint::broadcast::tx_async::Response>
        for penumbra_pb::BroadcastTxAsyncResponse
    {
        fn from(
            tendermint_rpc::endpoint::broadcast::tx_async::Response {
                code,
                data,
                log,
                hash,
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
        for penumbra_pb::BroadcastTxSyncResponse
    {
        fn from(
            tendermint_rpc::endpoint::broadcast::tx_sync::Response {
                code,
                data,
                log,
                hash,
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

    impl From<tendermint_rpc::endpoint::status::Response> for penumbra_pb::GetStatusResponse {
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

    impl From<tendermint_rpc::endpoint::status::SyncInfo> for penumbra_pb::SyncInfo {
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

    impl TryFrom<tendermint_rpc::endpoint::abci_query::AbciQuery> for penumbra_pb::AbciQueryResponse {
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
}
