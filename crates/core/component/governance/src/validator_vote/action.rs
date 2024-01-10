use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_proto::{penumbra::core::component::governance::v1alpha1 as pb, DomainType};
use penumbra_stake::{GovernanceKey, IdentityKey};
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::vote::Vote;

/// A vote by a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVote", into = "pb::ValidatorVote")]
pub struct ValidatorVote {
    /// The body of the validator vote.
    pub body: ValidatorVoteBody,
    /// The signature authorizing the vote (signed with governance key over the body).
    pub auth_sig: Signature<SpendAuth>,
}

impl EffectingData for ValidatorVote {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
        //self.body.effect_hash()
    }
}

/*
impl EffectingData for ValidatorVoteBody {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}
*/

impl DomainType for ValidatorVote {
    type Proto = pb::ValidatorVote;
}

impl From<ValidatorVote> for pb::ValidatorVote {
    fn from(msg: ValidatorVote) -> Self {
        Self {
            body: Some(msg.body.into()),
            auth_sig: Some(msg.auth_sig.into()),
        }
    }
}

impl TryFrom<pb::ValidatorVote> for ValidatorVote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVote) -> Result<Self, Self::Error> {
        Ok(Self {
            body: msg
                .body
                .ok_or_else(|| anyhow::anyhow!("missing validator vote body"))?
                .try_into()?,
            auth_sig: msg
                .auth_sig
                .ok_or_else(|| anyhow::anyhow!("missing validator auth sig"))?
                .try_into()?,
        })
    }
}

/// A public vote as a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVoteBody", into = "pb::ValidatorVoteBody")]
pub struct ValidatorVoteBody {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: Vote,
    /// The identity of the validator who is voting.
    pub identity_key: IdentityKey,
    /// The governance key for the validator who is voting.
    pub governance_key: GovernanceKey,
    /// A comment or justification of the vote. Limited to 1 KB.
    pub reason: ValidatorVoteReason,
}

impl From<ValidatorVoteBody> for pb::ValidatorVoteBody {
    fn from(value: ValidatorVoteBody) -> Self {
        pb::ValidatorVoteBody {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            identity_key: Some(value.identity_key.into()),
            governance_key: Some(value.governance_key.into()),
            reason: Some(value.reason.into()),
        }
    }
}

impl TryFrom<pb::ValidatorVoteBody> for ValidatorVoteBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVoteBody) -> Result<Self, Self::Error> {
        Ok(ValidatorVoteBody {
            proposal: msg.proposal,
            vote: msg
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `ValidatorVote`"))?
                .try_into()?,
            identity_key: msg
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing validator identity in `ValidatorVote`"))?
                .try_into()?,
            governance_key: msg
                .governance_key
                .ok_or_else(|| {
                    anyhow::anyhow!("missing validator governance key in `ValidatorVote`")
                })?
                .try_into()?,
            reason: msg
                .reason
                .ok_or_else(|| {
                    anyhow::anyhow!("missing validator governance key in `ValidatorVote`")
                })?
                .try_into()?,
        })
    }
}

impl DomainType for ValidatorVoteBody {
    type Proto = pb::ValidatorVoteBody;
}

/// A comment or justification of the vote. Limited to 1 KB.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVoteReason", into = "pb::ValidatorVoteReason")]
pub struct ValidatorVoteReason(pub String);

impl From<ValidatorVoteReason> for pb::ValidatorVoteReason {
    fn from(value: ValidatorVoteReason) -> Self {
        pb::ValidatorVoteReason { reason: value.0 }
    }
}

impl TryFrom<pb::ValidatorVoteReason> for ValidatorVoteReason {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVoteReason) -> Result<Self, Self::Error> {
        Ok(ValidatorVoteReason(msg.reason))
    }
}

impl DomainType for ValidatorVoteReason {
    type Proto = pb::ValidatorVoteReason;
}
