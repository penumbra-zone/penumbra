use std::str::FromStr;

use ibc::core::ics02_client::trust_threshold::TrustThreshold;
use ibc::core::ics23_commitment::specs::ProofSpecs;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::downcast;
use ibc::{
    clients::ics07_tendermint::{
        client_state, consensus_state, consensus_state::ConsensusState as TendermintConsensusState,
    },
    core::{
        ics02_client::{
            client_consensus::AnyConsensusState, client_state::AnyClientState, height::Height,
        },
        ics24_host::identifier::ClientId,
    },
};
use penumbra_proto::{ibc as pb, Protobuf};
use tendermint_proto::Protobuf as TmProtobuf;

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";
pub const TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.tendermint.v1.ConsensusState";

#[derive(Clone, Debug)]
pub struct ClientCounter(pub u64);

impl Protobuf<pb::ClientCounter> for ClientCounter {}

impl TryFrom<pb::ClientCounter> for ClientCounter {
    type Error = anyhow::Error;

    fn try_from(p: pb::ClientCounter) -> Result<Self, Self::Error> {
        Ok(ClientCounter(p.counter))
    }
}

impl From<ClientCounter> for pb::ClientCounter {
    fn from(c: ClientCounter) -> Self {
        pb::ClientCounter { counter: c.0 }
    }
}

#[derive(Clone, Debug)]
pub struct ClientState(pub AnyClientState);

impl Protobuf<prost_types::Any> for ClientState {}

impl TryFrom<prost_types::Any> for ClientState {
    type Error = anyhow::Error;

    fn try_from(raw: prost_types::Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => Ok(ClientState(AnyClientState::Tendermint(
                client_state::ClientState::decode_vec(&raw.value)
                    .map_err(|_| anyhow::anyhow!("could not decode tendermint client state"))?,
            ))),

            _ => Err(anyhow::anyhow!("unknown client type: {}", raw.type_url)),
        }
    }
}

impl From<ClientState> for prost_types::Any {
    fn from(value: ClientState) -> Self {
        match value {
            ClientState(AnyClientState::Tendermint(value)) => prost_types::Any {
                type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
                value: value
                    .encode_vec()
                    .expect("encoding to `Any` from `ClientState::Tendermint`"),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConsensusState(pub AnyConsensusState);

impl ConsensusState {
    pub fn as_tendermint(&self) -> Result<TendermintConsensusState, anyhow::Error> {
        match &self.0 {
            AnyConsensusState::Tendermint(state) => Ok(state.clone()),
            _ => return Err(anyhow::anyhow!("not a tendermint consensus state")),
        }
    }
}

impl From<TendermintConsensusState> for ConsensusState {
    fn from(value: TendermintConsensusState) -> Self {
        ConsensusState(AnyConsensusState::Tendermint(value))
    }
}

impl Protobuf<pb::ConsensusState> for ConsensusState {}

impl TryFrom<pb::ConsensusState> for ConsensusState {
    type Error = anyhow::Error;

    fn try_from(raw: pb::ConsensusState) -> Result<Self, Self::Error> {
        let state = raw
            .consensus_state
            .ok_or_else(|| anyhow::anyhow!("missing consensus state"))?;
        match state.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                Ok(ConsensusState(AnyConsensusState::Tendermint(
                    consensus_state::ConsensusState::decode_vec(&state.value).map_err(|_| {
                        anyhow::anyhow!("could not decode tendermint consensus state")
                    })?,
                )))
            }

            _ => Err(anyhow::anyhow!(
                "unknown consensus state type: {}",
                state.type_url
            )),
        }
    }
}

impl From<ConsensusState> for pb::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        match value {
            ConsensusState(AnyConsensusState::Tendermint(value)) => pb::ConsensusState {
                consensus_state: Some(prost_types::Any {
                    type_url: TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
                    value: value
                        .encode_vec()
                        .expect("encoding to `Any` from `ConsensusState::Tendermint`"),
                }),
            },
        }
    }
}

/// ClientData encapsulates the data that represents an ICS-02 client, stored in the Penumbra
/// state.
#[derive(Clone, Debug)]
pub struct ClientData {
    pub client_id: ClientId,
    pub client_state: ClientState,
    pub processed_time: String,
    pub processed_height: u64,
}

impl ClientData {
    pub fn new(
        client_id: ClientId,
        client_state: AnyClientState,
        processed_time: String,
        processed_height: u64,
    ) -> Self {
        ClientData {
            client_id,
            client_state: ClientState(client_state),
            processed_time,
            processed_height,
        }
    }
    pub fn with_new_client_state(
        &self,
        new_client_state: AnyClientState,
        new_processed_time: String,
        new_processed_height: u64,
    ) -> Self {
        ClientData {
            client_id: self.client_id.clone(),
            client_state: ClientState(new_client_state),
            processed_time: new_processed_time,
            processed_height: new_processed_height,
        }
    }
}

impl Protobuf<pb::ClientData> for ClientData {}

impl TryFrom<pb::ClientData> for ClientData {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ClientData) -> Result<Self, Self::Error> {
        Ok(ClientData {
            client_id: ClientId::from_str(&msg.client_id)?,
            client_state: ClientState::try_from(msg.client_state.unwrap())?,
            processed_time: msg.processed_time,
            processed_height: msg.processed_height,
        })
    }
}

impl From<ClientData> for pb::ClientData {
    fn from(d: ClientData) -> Self {
        Self {
            client_id: d.client_id.to_string(),
            client_state: Some(d.client_state.into()),
            processed_time: d.processed_time,
            processed_height: d.processed_height,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedHeights {
    pub heights: Vec<Height>,
}

impl Protobuf<pb::VerifiedHeights> for VerifiedHeights {}

impl TryFrom<pb::VerifiedHeights> for VerifiedHeights {
    type Error = anyhow::Error;

    fn try_from(msg: pb::VerifiedHeights) -> Result<Self, Self::Error> {
        Ok(VerifiedHeights {
            heights: msg.heights.into_iter().map(|h| h.into()).collect(),
        })
    }
}

impl From<VerifiedHeights> for pb::VerifiedHeights {
    fn from(d: VerifiedHeights) -> Self {
        Self {
            heights: d.heights.into_iter().map(|h| h.into()).collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClientConnections {
    pub connection_ids: Vec<ConnectionId>,
}

impl Protobuf<pb::ClientConnections> for ClientConnections {}

impl TryFrom<pb::ClientConnections> for ClientConnections {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ClientConnections) -> Result<Self, Self::Error> {
        Ok(ClientConnections {
            connection_ids: msg
                .connections
                .into_iter()
                .map(|h| h.parse())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<ClientConnections> for pb::ClientConnections {
    fn from(d: ClientConnections) -> Self {
        Self {
            connections: d
                .connection_ids
                .into_iter()
                .map(|h| h.as_str().to_string())
                .collect(),
        }
    }
}

// Check that the trust threshold is:
//
// a) non-zero
// b) greater or equal to 1/3
// c) strictly less than 1
fn validate_trust_threshold(trust_threshold: TrustThreshold) -> Result<(), anyhow::Error> {
    if trust_threshold.denominator() == 0 {
        return Err(anyhow::anyhow!(
            "trust threshold denominator cannot be zero"
        ));
    }

    if trust_threshold.numerator() * 3 < trust_threshold.denominator() {
        return Err(anyhow::anyhow!("trust threshold must be greater than 1/3"));
    }

    if trust_threshold.numerator() >= trust_threshold.denominator() {
        return Err(anyhow::anyhow!(
            "trust threshold must be strictly less than 1"
        ));
    }

    Ok(())
}

// validate the parameters of an AnyClientState, verifying that it is a valid Penumbra client
// state.
pub fn validate_penumbra_client_state(
    client_state: AnyClientState,
    chain_id: &str,
    current_height: u64,
) -> Result<(), anyhow::Error> {
    let tm_client_state = downcast!(client_state => AnyClientState::Tendermint)
        .ok_or_else(|| anyhow::anyhow!("invalid client state: not a Tendermint client state"))?;

    if tm_client_state.frozen_height.is_some() {
        return Err(anyhow::anyhow!("invalid client state: frozen"));
    }

    // NOTE: Chain ID validation is actually not standardized yet. see
    // https://github.com/informalsystems/ibc-rs/pull/304#discussion_r503917283
    let chain_id = ChainId::from_string(&chain_id);
    if chain_id != tm_client_state.chain_id {
        return Err(anyhow::anyhow!(
            "invalid client state: chain id does not match"
        ));
    }

    // check that the revision number is the same as our chain ID's version
    if tm_client_state.latest_height.revision_number != chain_id.version() {
        return Err(anyhow::anyhow!(
            "invalid client state: revision number does not match"
        ));
    }

    // check that the latest height isn't gte the current block height
    if tm_client_state.latest_height.revision_height >= current_height {
        return Err(anyhow::anyhow!(
                "invalid client state: latest height is greater than or equal to the current block height"
            ));
    }

    // check client proof specs match penumbra proof specs
    let penumbra_proof_specs: ProofSpecs = vec![jmt::ics23_spec()].into();
    if penumbra_proof_specs != tm_client_state.proof_specs {
        return Err(anyhow::anyhow!(
            "invalid client state: proof specs do not match"
        ));
    }

    // check that the trust level is correct
    validate_trust_threshold(tm_client_state.trust_level)?;

    // TODO: check that the unbonding period is correct
    //
    // - check unbonding period is greater than trusting period
    if tm_client_state.unbonding_period < tm_client_state.trusting_period {
        return Err(anyhow::anyhow!(
            "invalid client state: unbonding period is less than trusting period"
        ));
    }

    // TODO: check upgrade path

    Ok(())
}
