use anyhow::{anyhow, Context};
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_sdk_asset::{
    asset::{self, Denom, REGISTRY},
    balance, Value,
};
use penumbra_sdk_keys::Address;
use penumbra_sdk_proto::{core::component::funding::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_tct::Position;
use penumbra_sdk_txhash::{EffectHash, EffectingData};

use super::proof::LiquidityTournamentVoteProof;

/// The internal body of an LQT vote, containing the intended vote and other validity information.
///
/// c.f. [`penumbra_sdk_governance::delegator_vote::action::DelegatorVoteBody`], which is similar.
#[derive(Clone, Debug)]
pub struct LiquidityTournamentVoteBody {
    /// Which asset is being incentivized.
    ///
    /// We use the base denom to allow filtering particular asset sources (i.e. IBC transfers)a.
    pub incentivized: Denom,
    /// The address that will receive any rewards for voting.
    pub rewards_recipient: Address,
    /// The start position of the tournament.
    ///
    /// This is included to allow stateless verification, but should match the epoch of the LQT.
    pub start_position: Position,
    /// The value being used to vote with.
    ///
    /// This should be the delegation tokens for a specific validator.
    pub value: Value,
    /// The nullifier of the note being spent.
    pub nullifier: Nullifier,
    /// The key that must be used to vote.
    pub rk: VerificationKey<SpendAuth>,
}

impl LiquidityTournamentVoteBody {
    /// Get the asset id that should be incentivized.
    ///
    /// This will return None if the denom is not a base denom.
    pub fn incentivized_id(&self) -> Option<asset::Id> {
        REGISTRY
            .parse_denom(&self.incentivized.denom)
            .map(|x| x.id())
    }
}

impl DomainType for LiquidityTournamentVoteBody {
    type Proto = pb::LiquidityTournamentVoteBody;
}

impl TryFrom<pb::LiquidityTournamentVoteBody> for LiquidityTournamentVoteBody {
    type Error = anyhow::Error;

    fn try_from(proto: pb::LiquidityTournamentVoteBody) -> Result<Self, Self::Error> {
        Result::<_, Self::Error>::Ok(Self {
            incentivized: proto
                .incentivized
                .ok_or_else(|| anyhow!("missing `incentivized`"))?
                .try_into()?,
            rewards_recipient: proto
                .rewards_recipient
                .ok_or_else(|| anyhow!("missing `rewards_recipient`"))?
                .try_into()?,
            start_position: proto.start_position.into(),
            value: proto
                .value
                .ok_or_else(|| anyhow!("missing `value`"))?
                .try_into()?,
            nullifier: proto
                .nullifier
                .ok_or_else(|| anyhow!("missing `nullifier`"))?
                .try_into()?,
            rk: proto
                .rk
                .ok_or_else(|| anyhow!("missing `rk`"))?
                .try_into()?,
        })
        .with_context(|| format!("while parsing {}", std::any::type_name::<Self>()))
    }
}

impl From<LiquidityTournamentVoteBody> for pb::LiquidityTournamentVoteBody {
    fn from(value: LiquidityTournamentVoteBody) -> Self {
        Self {
            incentivized: Some(value.incentivized.into()),
            rewards_recipient: Some(value.rewards_recipient.into()),
            start_position: value.start_position.into(),
            value: Some(value.value.into()),
            nullifier: Some(value.nullifier.into()),
            rk: Some(value.rk.into()),
        }
    }
}

impl EffectingData for LiquidityTournamentVoteBody {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

/// The action used to vote in the liquidity tournament.
///
/// This vote is towards a particular asset whose liquidity should be incentivized,
/// and is weighted by the amount of delegation tokens being expended.
#[derive(Clone, Debug)]
pub struct ActionLiquidityTournamentVote {
    /// The actual body, containing the vote and other validity information.
    pub body: LiquidityTournamentVoteBody,
    /// An authorization over the body.
    pub auth_sig: Signature<SpendAuth>,
    /// A ZK proof tying in the private information for this action.
    pub proof: LiquidityTournamentVoteProof,
}

impl DomainType for ActionLiquidityTournamentVote {
    type Proto = pb::ActionLiquidityTournamentVote;
}

impl TryFrom<pb::ActionLiquidityTournamentVote> for ActionLiquidityTournamentVote {
    type Error = anyhow::Error;

    fn try_from(value: pb::ActionLiquidityTournamentVote) -> Result<Self, Self::Error> {
        Result::<_, Self::Error>::Ok(Self {
            body: value
                .body
                .ok_or_else(|| anyhow!("missing `body`"))?
                .try_into()?,
            auth_sig: value
                .auth_sig
                .ok_or_else(|| anyhow!("missing `auth_sig`"))?
                .try_into()?,
            proof: value
                .proof
                .ok_or_else(|| anyhow!("missing `proof`"))?
                .try_into()?,
        })
        .with_context(|| format!("while parsing {}", std::any::type_name::<Self>()))
    }
}

impl From<ActionLiquidityTournamentVote> for pb::ActionLiquidityTournamentVote {
    fn from(value: ActionLiquidityTournamentVote) -> Self {
        Self {
            body: Some(value.body.into()),
            auth_sig: Some(value.auth_sig.into()),
            proof: Some(value.proof.into()),
        }
    }
}

impl EffectingData for ActionLiquidityTournamentVote {
    fn effect_hash(&self) -> EffectHash {
        self.body.effect_hash()
    }
}

impl ActionLiquidityTournamentVote {
    /// This action doesn't actually produce or consume value.
    pub fn balance_commitment(&self) -> balance::Commitment {
        // This will be the commitment to zero.
        balance::Commitment::default()
    }
}
