use crate::IBC_PROOF_SPECS;
use anyhow::{anyhow, Result};
use ibc_proto::google::protobuf::Any;
use ibc_types::{
    core::connection::ChainId,
    lightclients::tendermint::{
        client_state::{ClientState as TendermintClientState, TENDERMINT_CLIENT_STATE_TYPE_URL},
        consensus_state::{
            ConsensusState as TendermintConsensusState, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
        },
        header::{Header as TendermintHeader, TENDERMINT_HEADER_TYPE_URL},
        misbehaviour::{Misbehaviour as TendermintMisbehavior, TENDERMINT_MISBEHAVIOUR_TYPE_URL},
        TrustThreshold,
    },
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

// validate the parameters of an AnyClientState, verifying that it is a valid Penumbra client
// state.
pub fn validate_penumbra_client_state(
    client_state: Any,
    chain_id: &str,
    current_height: u64,
) -> anyhow::Result<()> {
    let tm_client_state = get_tendermint_client_state(client_state)?;

    if tm_client_state.frozen_height.is_some() {
        anyhow::bail!("invalid client state: frozen");
    }

    // NOTE: Chain ID validation is actually not standardized yet. see
    // https://github.com/informalsystems/ibc-rs/pull/304#discussion_r503917283
    let chain_id = ChainId::from_string(chain_id);
    if chain_id != tm_client_state.chain_id {
        anyhow::bail!("invalid client state: chain id does not match");
    }

    // check that the revision number is the same as our chain ID's version
    if tm_client_state.latest_height().revision_number() != chain_id.version() {
        anyhow::bail!("invalid client state: revision number does not match");
    }

    // check that the latest height isn't gte the current block height
    if tm_client_state.latest_height().revision_height() >= current_height {
        anyhow::bail!(
            "invalid client state: latest height is greater than or equal to the current block height"
        );
    }

    // check client proof specs match penumbra proof specs
    if IBC_PROOF_SPECS.clone() != tm_client_state.proof_specs {
        // allow legacy proof specs without prehash_key_before_comparison
        let mut spec_with_prehash_key = tm_client_state.proof_specs.clone();
        spec_with_prehash_key[0].prehash_key_before_comparison = true;
        spec_with_prehash_key[1].prehash_key_before_comparison = true;
        if IBC_PROOF_SPECS.clone() != spec_with_prehash_key {
            anyhow::bail!("invalid client state: proof specs do not match");
        }
    }

    // check that the trust level is correct
    validate_trust_threshold(tm_client_state.trust_level)?;

    // TODO: check that the unbonding period is correct
    //
    // - check unbonding period is greater than trusting period
    if tm_client_state.unbonding_period < tm_client_state.trusting_period {
        anyhow::bail!("invalid client state: unbonding period is less than trusting period");
    }

    // TODO: check upgrade path

    Ok(())
}

// Check that the trust threshold is:
//
// a) non-zero
// b) greater or equal to 1/3
// c) strictly less than 1
fn validate_trust_threshold(trust_threshold: TrustThreshold) -> anyhow::Result<()> {
    if trust_threshold.denominator() == 0 {
        anyhow::bail!("trust threshold denominator cannot be zero");
    }

    if trust_threshold.numerator() * 3 < trust_threshold.denominator() {
        anyhow::bail!("trust threshold must be greater than 1/3");
    }

    if trust_threshold.numerator() >= trust_threshold.denominator() {
        anyhow::bail!("trust threshold must be strictly less than 1");
    }

    Ok(())
}
