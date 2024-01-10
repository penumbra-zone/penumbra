use crate::{component::ics02_validation, IBC_PROOF_SPECS};
use ibc_proto::google::protobuf::Any;
use ibc_types::core::client::Height;
use ibc_types::core::connection::{ChainId, ConnectionId};
use ibc_types::lightclients::tendermint::TrustThreshold;
use penumbra_proto::{penumbra::core::component::ibc::v1alpha1 as pb, DomainType};

#[derive(Clone, Debug)]
pub struct ClientCounter(pub u64);

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

impl DomainType for VerifiedHeights {
    type Proto = pb::VerifiedHeights;
}

impl TryFrom<pb::VerifiedHeights> for VerifiedHeights {
    type Error = anyhow::Error;

    fn try_from(msg: pb::VerifiedHeights) -> Result<Self, Self::Error> {
        let heights = msg.heights.into_iter().map(TryFrom::try_from).collect();
        match heights {
            Ok(heights) => Ok(VerifiedHeights { heights }),
            Err(e) => anyhow::bail!(format!("invalid height: {e}")),
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

// validate the parameters of an AnyClientState, verifying that it is a valid Penumbra client
// state.
pub fn validate_penumbra_client_state(
    client_state: Any,
    chain_id: &str,
    current_height: u64,
) -> anyhow::Result<()> {
    let tm_client_state = ics02_validation::get_tendermint_client_state(client_state)?;

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
