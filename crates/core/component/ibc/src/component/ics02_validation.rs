use anyhow::{anyhow, Result};
use ibc_proto::google::protobuf::Any;
use ibc_types::lightclients::tendermint::client_state::{
    ClientState as TendermintClientState, TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use ibc_types::lightclients::tendermint::consensus_state::{
    ConsensusState as TendermintConsensusState, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc_types::lightclients::tendermint::header::{
    Header as TendermintHeader, TENDERMINT_HEADER_TYPE_URL,
};
use ibc_types::lightclients::tendermint::misbehaviour::{
    Misbehaviour as TendermintMisbehavior, TENDERMINT_MISBEHAVIOUR_TYPE_URL,
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
pub fn is_tendermint_misbehavior(misbehavior: &Any) -> bool {
    misbehavior.type_url.as_str() == TENDERMINT_MISBEHAVIOUR_TYPE_URL
}

pub fn get_tendermint_misbehavior(misbehavior: Any) -> Result<TendermintMisbehavior> {
    if is_tendermint_misbehavior(&misbehavior) {
        TendermintMisbehavior::try_from(misbehavior)
            .map_err(|e| anyhow!(format!("failed to deserialize tendermint misbehavior: {e}")))
    } else {
        anyhow::bail!(format!(
            "expected a tendermint light client misbehavior, got: {}",
            misbehavior.type_url.as_str()
        ))
    }
}

pub fn get_tendermint_header(header: Any) -> Result<TendermintHeader> {
    if is_tendermint_header_state(&header) {
        TendermintHeader::try_from(header)
            .map_err(|e| anyhow!(format!("failed to deserialize tendermint header: {e}")))
    } else {
        anyhow::bail!(format!(
            "expected a tendermint light client header, got: {}",
            header.type_url.as_str()
        ))
    }
}

pub fn get_tendermint_consensus_state(consensus_state: Any) -> Result<TendermintConsensusState> {
    if is_tendermint_consensus_state(&consensus_state) {
        TendermintConsensusState::try_from(consensus_state).map_err(|e| {
            anyhow!(format!(
                "failed to deserialize tendermint consensus state: {e}"
            ))
        })
    } else {
        anyhow::bail!(format!(
            "expected tendermint consensus state, got: {}",
            consensus_state.type_url.as_str()
        ))
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
        anyhow::bail!(format!(
            "expected tendermint client state, got: {}",
            client_state.type_url.as_str()
        ))
    }
}
