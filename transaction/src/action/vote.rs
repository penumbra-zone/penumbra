use std::{
    fmt::{self, Display},
    str::FromStr,
};

use anyhow::{anyhow, Context};
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_crypto::{
    proofs::transparent::DelegatorVoteProof, stake::IdentityKey, Amount, GovernanceKey, Nullifier,
    Value, VotingReceiptToken,
};
use penumbra_proto::{
    core::{crypto::v1alpha1::BalanceCommitment, governance::v1alpha1 as pb},
    DomainType,
};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

use crate::{Action, ActionView, IsAction, TransactionPerspective};

/// A vote on a proposal.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(try_from = "pb::Vote", into = "pb::Vote")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum Vote {
    /// The vote is to approve the proposal.
    Yes,
    /// The vote is to reject the proposal.
    No,
    /// The vote is to abstain from the proposal.
    Abstain,
}

impl FromStr for Vote {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Vote> {
        match s.replace(['-', '_', ' '], "").to_lowercase().as_str() {
            "yes" | "y" => Ok(Vote::Yes),
            "no" | "n" => Ok(Vote::No),
            "abstain" | "a" => Ok(Vote::Abstain),
            _ => Err(anyhow::anyhow!("invalid vote: {}", s)),
        }
    }
}

impl Display for Vote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Vote::Yes => write!(f, "yes"),
            Vote::No => write!(f, "no"),
            Vote::Abstain => write!(f, "abstain"),
        }
    }
}

impl From<Vote> for pb::Vote {
    fn from(value: Vote) -> Self {
        pb_from_vote(value)
    }
}

// Factored out so it can be used in a const
const fn pb_from_vote(vote: Vote) -> pb::Vote {
    match vote {
        Vote::Yes => pb::Vote {
            vote: pb::vote::Vote::Yes as i32,
        },
        Vote::No => pb::Vote {
            vote: pb::vote::Vote::No as i32,
        },
        Vote::Abstain => pb::Vote {
            vote: pb::vote::Vote::Abstain as i32,
        },
    }
}

impl TryFrom<pb::Vote> for Vote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::Vote) -> Result<Self, Self::Error> {
        let Some(vote_state) = pb::vote::Vote::from_i32(msg.vote) else {
            return Err(anyhow!("invalid vote state"))
        };
        match vote_state {
            pb::vote::Vote::Abstain => Ok(Vote::Abstain),
            pb::vote::Vote::Yes => Ok(Vote::Yes),
            pb::vote::Vote::No => Ok(Vote::No),
            pb::vote::Vote::Unspecified => Err(anyhow!("unspecified vote state")),
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::proptest;

    proptest! {
        #[test]
        fn vote_roundtrip_serialize(vote: super::Vote) {
            let pb_vote: super::pb::Vote = vote.into();
            let vote2 = super::Vote::try_from(pb_vote).unwrap();
            assert_eq!(vote, vote2);
        }
    }
}

impl DomainType for Vote {
    type Proto = pb::Vote;
}

/// A vote by a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVote", into = "pb::ValidatorVote")]
pub struct ValidatorVote {
    /// The body of the validator vote.
    pub body: ValidatorVoteBody,
    /// The signature authorizing the vote (signed with governance key over the body).
    pub auth_sig: Signature<SpendAuth>,
}

impl IsAction for ValidatorVote {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        Default::default()
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ValidatorVote(self.to_owned())
    }
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
}

impl From<ValidatorVoteBody> for pb::ValidatorVoteBody {
    fn from(value: ValidatorVoteBody) -> Self {
        pb::ValidatorVoteBody {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            identity_key: Some(value.identity_key.into()),
            governance_key: Some(value.governance_key.into()),
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
        })
    }
}

impl DomainType for ValidatorVoteBody {
    type Proto = pb::ValidatorVoteBody;
}

#[derive(Debug, Clone)]
pub struct DelegatorVote {
    pub body: DelegatorVoteBody,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: DelegatorVoteProof,
}

impl IsAction for DelegatorVote {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        Value {
            asset_id: VotingReceiptToken::new(self.body.proposal).id(),
            amount: self.body.unbonded_amount,
        }
        .commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        todo!()
    }
}

/// The body of a delegator vote.
#[derive(Debug, Clone)]
pub struct DelegatorVoteBody {
    /// The proposal ID the vote is for.
    pub proposal: u64,
    /// The epoch the proposal started at.
    pub start_epoch: u16,
    /// The block within that epoch the proposal started at.
    pub start_block: u16,
    /// The vote on the proposal.
    pub vote: Vote, // With flow encryption, this will be a triple of flow ciphertexts
    /// The value of the staked note being used to vote.
    pub value: Value, // With flow encryption, this will be a triple of balance commitments, and a public denomination
    /// The unbonded amount equivalent to the value above
    pub unbonded_amount: Amount,
    /// The nullifier of the staked note being used to vote.
    pub nullifier: Nullifier,
    /// The randomized validating key for the spend authorization signature.
    pub rk: VerificationKey<SpendAuth>,
}

impl From<DelegatorVoteBody> for pb::DelegatorVoteBody {
    fn from(value: DelegatorVoteBody) -> Self {
        pb::DelegatorVoteBody {
            proposal: value.proposal,
            start_epoch_and_block_position: (value.start_epoch as u32) << 16
                | value.start_block as u32,
            vote: Some(value.vote.into()),
            value: Some(value.value.into()),
            unbonded_amount: Some(value.unbonded_amount.into()),
            nullifier: value.nullifier.to_bytes().into(),
            rk: value.rk.to_bytes().into(),
        }
    }
}

impl TryFrom<pb::DelegatorVoteBody> for DelegatorVoteBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DelegatorVoteBody) -> Result<Self, Self::Error> {
        Ok(DelegatorVoteBody {
            proposal: msg.proposal,
            start_epoch: (msg.start_epoch_and_block_position >> 16) as u16,
            start_block: (msg.start_epoch_and_block_position & 0xffff) as u16,
            vote: msg
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `DelegatorVote`"))?
                .try_into()?,
            value: msg
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value in `DelegatorVote`"))?
                .try_into()?,
            unbonded_amount: msg
                .unbonded_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonded amount in `DelegatorVote`"))?
                .try_into()?,
            nullifier: msg
                .nullifier
                .try_into()
                .context("invalid nullifier in `DelegatorVote`")?,
            rk: {
                let rk_bytes: [u8; 32] = (msg.rk[..])
                    .try_into()
                    .context("expected 32-byte rk in `DelegatorVote`")?;
                rk_bytes
                    .try_into()
                    .context("invalid  rk in `DelegatorVote`")?
            },
        })
    }
}

impl DomainType for DelegatorVoteBody {
    type Proto = pb::DelegatorVoteBody;
}

impl From<DelegatorVote> for pb::DelegatorVote {
    fn from(value: DelegatorVote) -> Self {
        pb::DelegatorVote {
            body: Some(value.body.into()),
            auth_sig: Some(value.auth_sig.into()),
            proof: value.proof.into(),
        }
    }
}

impl TryFrom<pb::DelegatorVote> for DelegatorVote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DelegatorVote) -> Result<Self, Self::Error> {
        Ok(DelegatorVote {
            body: msg
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body in `DelegatorVote`"))?
                .try_into()?,
            auth_sig: msg
                .auth_sig
                .ok_or_else(|| anyhow::anyhow!("missing auth sig in `DelegatorVote`"))?
                .try_into()?,
            proof: msg.proof[..].try_into()?,
        })
    }
}

impl From<DelegatorVote> for Action {
    fn from(value: DelegatorVote) -> Self {
        Action::DelegatorVote(value)
    }
}
