use anyhow::anyhow;

use ibc_proto::google::protobuf::Any;
use ibc_types2::core::client::Height;
use ibc_types2::core::connection::{ChainId, ConnectionId};
use penumbra_chain::component::PENUMBRA_PROOF_SPECS;
use penumbra_proto::{core::ibc::v1alpha1 as pb, DomainType, TypeUrl};

#[derive(Clone, Debug)]
pub struct ClientCounter(pub u64);

impl TypeUrl for ClientCounter {
    const TYPE_URL: &'static str = "/penumbra.core.ibc.v1alpha1.ClientCounter";
}

impl DomainType for ClientCounter {
    type Proto = pb::ClientCounter;
}

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
pub struct VerifiedHeights {
    pub heights: Vec<Height>,
}

impl TypeUrl for VerifiedHeights {
    const TYPE_URL: &'static str = "/penumbra.core.ibc.v1alpha1.VerifiedHeights";
}

impl DomainType for VerifiedHeights {
    type Proto = pb::VerifiedHeights;
}

impl TryFrom<pb::VerifiedHeights> for VerifiedHeights {
    type Error = anyhow::Error;

    fn try_from(msg: pb::VerifiedHeights) -> Result<Self, Self::Error> {
        let heights = msg.heights.into_iter().map(TryFrom::try_from).collect();
        match heights {
            Ok(heights) => Ok(VerifiedHeights { heights }),
            Err(e) => Err(anyhow!(format!("invalid height: {e}"))),
        }
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

impl TypeUrl for ClientConnections {
    const TYPE_URL: &'static str = "/penumbra.core.ibc.v1alpha1.ClientConnections";
}

impl DomainType for ClientConnections {
    type Proto = pb::ClientConnections;
}

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

pub(crate) mod ics02_validation {
    use anyhow::{anyhow, Result};
    use ibc_proto::google::protobuf::Any;
    use ibc_types2::lightclients::tendermint::client_state::{
        ClientState as TendermintClientState, TENDERMINT_CLIENT_STATE_TYPE_URL,
    };
    use ibc_types2::lightclients::tendermint::consensus_state::{
        ConsensusState as TendermintConsensusState, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
    };
    use ibc_types2::lightclients::tendermint::header::{
        Header as TendermintHeader, TENDERMINT_HEADER_TYPE_URL,
    };

    pub fn is_tendermint_header_state(header: &Any) -> bool {
        header.type_url.as_str() == TENDERMINT_HEADER_TYPE_URL
    }
    pub fn is_tendermint_consensus_state(consensus_state: &Any) -> bool {
        consensus_state.type_url.as_str() == TENDERMINT_CONSENSUS_STATE_TYPE_URL
    }
    pub fn is_tendermint_client_state(client_state: &Any) -> bool {
        client_state.type_url.as_str() == TENDERMINT_CLIENT_STATE_TYPE_URL
    }

    pub fn get_tendermint_header(header: Any) -> Result<TendermintHeader> {
        if is_tendermint_header_state(&header) {
            TendermintHeader::try_from(header)
                .map_err(|e| anyhow!(format!("failed to deserialize tendermint header: {e}")))
        } else {
            Err(anyhow!(format!(
                "expected a tendermint light client header, got: {}",
                header.type_url.as_str()
            )))
        }
    }

    pub fn get_tendermint_consensus_state(
        consensus_state: Any,
    ) -> Result<TendermintConsensusState> {
        if is_tendermint_consensus_state(&consensus_state) {
            TendermintConsensusState::try_from(consensus_state).map_err(|e| {
                anyhow!(format!(
                    "failed to deserialize tendermint consensus state: {e}"
                ))
            })
        } else {
            Err(anyhow!(format!(
                "expected tendermint consensus state, got: {}",
                consensus_state.type_url.as_str()
            )))
        }
    }
    pub fn get_tendermint_client_state(client_state: Any) -> Result<TendermintClientState> {
        if is_tendermint_client_state(&client_state) {
            TendermintClientState::try_from(client_state).map_err(|e| {
                anyhow!(format!(
                    "failed to deserialize tendermint client state: {e}"
                ))
            })
        } else {
            Err(anyhow!(format!(
                "expected tendermint client state, got: {}",
                client_state.type_url.as_str()
            )))
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
    client_state: Any,
    chain_id: &str,
    current_height: u64,
) -> Result<(), anyhow::Error> {
    let tm_client_state = ics02_validation::get_tendermint_client_state(client_state)?;

    if tm_client_state.frozen_height().is_some() {
        return Err(anyhow::anyhow!("invalid client state: frozen"));
    }

    // NOTE: Chain ID validation is actually not standardized yet. see
    // https://github.com/informalsystems/ibc-rs/pull/304#discussion_r503917283
    let chain_id = ChainId::from_string(chain_id);
    if chain_id != tm_client_state.chain_id {
        return Err(anyhow::anyhow!(
            "invalid client state: chain id does not match"
        ));
    }

    // check that the revision number is the same as our chain ID's version
    if tm_client_state.latest_height().revision_number() != chain_id.version() {
        return Err(anyhow::anyhow!(
            "invalid client state: revision number does not match"
        ));
    }

    // check that the latest height isn't gte the current block height
    if tm_client_state.latest_height().revision_height() >= current_height {
        return Err(anyhow::anyhow!(
                "invalid client state: latest height is greater than or equal to the current block height"
            ));
    }

    // check client proof specs match penumbra proof specs
    if PENUMBRA_PROOF_SPECS.clone() != tm_client_state.proof_specs {
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
