use std::{
    collections::{BTreeMap, HashMap},
    iter,
};

use anyhow::{anyhow, Result};
use decaf377_frost as frost;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use frost::round1::SigningCommitments;
use penumbra_proto::{penumbra::custody::threshold::v1alpha1 as pb, DomainType, Message};
use penumbra_transaction::{plan::TransactionPlan, AuthorizationData};
use penumbra_txhash::EffectHash;
use rand_core::CryptoRngCore;

use super::config::Config;

/// Represents the message sent by the coordinator at the start of the signing process.
///
/// This is nominally "round 1", even though it's the only message the coordinator ever sends.
#[derive(Debug, Clone)]
pub struct CoordinatorRound1 {
    plan: TransactionPlan,
}

impl CoordinatorRound1 {
    /// View the transaction plan associated with the first message.
    ///
    /// We need this method to be able to prompt users correctly.
    pub fn plan(&self) -> &TransactionPlan {
        &self.plan
    }
}

impl From<CoordinatorRound1> for pb::CoordinatorRound1 {
    fn from(value: CoordinatorRound1) -> Self {
        Self {
            plan: Some(value.plan.into()),
        }
    }
}

impl TryFrom<pb::CoordinatorRound1> for CoordinatorRound1 {
    type Error = anyhow::Error;

    fn try_from(value: pb::CoordinatorRound1) -> Result<Self, Self::Error> {
        Ok(Self {
            plan: value.plan.ok_or(anyhow!("missing plan"))?.try_into()?,
        })
    }
}

impl DomainType for CoordinatorRound1 {
    type Proto = pb::CoordinatorRound1;
}

#[derive(Debug, Clone)]
pub struct CoordinatorRound2 {
    // For each thing to sign, a map from FROST identifiers to a pair of commitments.
    all_commitments: Vec<BTreeMap<frost::Identifier, frost::round1::SigningCommitments>>,
}

fn commitments_to_pb(
    commitments: impl IntoIterator<Item = frost::round1::SigningCommitments>,
) -> pb::follower_round1::Inner {
    pb::follower_round1::Inner {
        commitments: commitments.into_iter().map(|x| x.into()).collect(),
    }
}

impl From<CoordinatorRound2> for pb::CoordinatorRound2 {
    fn from(value: CoordinatorRound2) -> Self {
        Self {
            signing_packages: value
                .all_commitments
                .into_iter()
                .map(|x| pb::coordinator_round2::PartialSigningPackage {
                    all_commitments: x
                        .into_iter()
                        .map(
                            |(id, commitment)| pb::coordinator_round2::IdentifiedCommitments {
                                identifier: id.serialize(),
                                commitments: Some(commitment.into()),
                            },
                        )
                        .collect(),
                })
                .collect(),
        }
    }
}

impl TryFrom<pb::CoordinatorRound2> for CoordinatorRound2 {
    type Error = anyhow::Error;

    fn try_from(value: pb::CoordinatorRound2) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            all_commitments: value
                .signing_packages
                .into_iter()
                .map(|x| {
                    let mut acc = BTreeMap::new();
                    for id_commitment in x.all_commitments {
                        let identifier = frost::Identifier::deserialize(&id_commitment.identifier)?;
                        if acc.contains_key(&identifier) {
                            anyhow::bail!(
                                "duplicate key when deserializing CoordinatorRound2: {:?}",
                                &identifier
                            );
                        }
                        let commitment = id_commitment
                            .commitments
                            .ok_or(anyhow!("CoordinatorRound2 missing commitments"))?
                            .try_into()?;
                        acc.insert(identifier, commitment);
                    }
                    Ok(acc)
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl DomainType for CoordinatorRound2 {
    type Proto = pb::CoordinatorRound2;
}

/// The message sent by the followers in round1 of signing.
#[derive(Debug, Clone)]
pub struct FollowerRound1 {
    /// A commitment for each spend we need to authorize.
    pub(self) commitments: Vec<frost::round1::SigningCommitments>,
    /// A verification key identifying who the sender is.
    pub(self) pk: VerificationKey,
    /// The signature over the protobuf encoding of the commitments.
    pub(self) sig: Signature,
}

impl From<FollowerRound1> for pb::FollowerRound1 {
    fn from(value: FollowerRound1) -> Self {
        Self {
            inner: Some(commitments_to_pb(value.commitments)),
            pk: Some(pb::VerificationKey {
                inner: value.pk.to_bytes().to_vec(),
            }),
            sig: Some(pb::Signature {
                inner: value.sig.to_bytes().to_vec(),
            }),
        }
    }
}

impl TryFrom<pb::FollowerRound1> for FollowerRound1 {
    type Error = anyhow::Error;

    fn try_from(value: pb::FollowerRound1) -> Result<Self, Self::Error> {
        Ok(Self {
            commitments: value
                .inner
                .ok_or(anyhow!("missing inner"))?
                .commitments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            pk: value
                .pk
                .ok_or(anyhow!("missing pk"))?
                .inner
                .as_slice()
                .try_into()?,
            sig: value
                .sig
                .ok_or(anyhow!("missing sig"))?
                .inner
                .as_slice()
                .try_into()?,
        })
    }
}

impl FollowerRound1 {
    // Make a round1 message, automatically signing the right bytes
    fn make(signing_key: &SigningKey, commitments: Vec<SigningCommitments>) -> Self {
        Self {
            commitments: commitments.clone(),
            pk: signing_key.verification_key(),
            sig: signing_key.sign(&commitments_to_pb(commitments).encode_to_vec()),
        }
    }

    // Extract the commitments from this struct, checking the signature
    fn checked_commitments(self) -> Result<(VerificationKey, Vec<SigningCommitments>)> {
        self.pk.verify(
            &self.sig,
            &commitments_to_pb(self.commitments.clone()).encode_to_vec(),
        )?;
        Ok((self.pk, self.commitments))
    }
}

impl DomainType for FollowerRound1 {
    type Proto = pb::FollowerRound1;
}

fn shares_to_pb(shares: Vec<frost::round2::SignatureShare>) -> pb::follower_round2::Inner {
    pb::follower_round2::Inner {
        shares: shares.into_iter().map(|x| x.into()).collect(),
    }
}

/// The message sent by the followers in round2 of signing.
#[derive(Debug, Clone)]
pub struct FollowerRound2 {
    /// A share of each signature we need to produce.
    pub(self) shares: Vec<frost::round2::SignatureShare>,
    /// A verification key identifying who the sender is.
    pub(self) pk: VerificationKey,
    /// The signature over the protobuf encoding of the sahres.
    pub(self) sig: Signature,
}

impl From<FollowerRound2> for pb::FollowerRound2 {
    fn from(value: FollowerRound2) -> Self {
        Self {
            inner: Some(shares_to_pb(value.shares)),
            pk: Some(pb::VerificationKey {
                inner: value.pk.to_bytes().to_vec(),
            }),
            sig: Some(pb::Signature {
                inner: value.sig.to_bytes().to_vec(),
            }),
        }
    }
}

impl TryFrom<pb::FollowerRound2> for FollowerRound2 {
    type Error = anyhow::Error;

    fn try_from(value: pb::FollowerRound2) -> Result<Self, Self::Error> {
        Ok(Self {
            shares: value
                .inner
                .ok_or(anyhow!("missing inner"))?
                .shares
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            pk: value
                .pk
                .ok_or(anyhow!("missing pk"))?
                .inner
                .as_slice()
                .try_into()?,
            sig: value
                .sig
                .ok_or(anyhow!("missing sig"))?
                .inner
                .as_slice()
                .try_into()?,
        })
    }
}

impl FollowerRound2 {
    // Make a round1 message, automatically signing the right bytes
    fn make(signing_key: &SigningKey, shares: Vec<frost::round2::SignatureShare>) -> Self {
        Self {
            shares: shares.clone(),
            pk: signing_key.verification_key(),
            sig: signing_key.sign(&shares_to_pb(shares).encode_to_vec()),
        }
    }

    // Extract the commitments from this struct, checking the signature
    fn checked_shares(self) -> Result<(VerificationKey, Vec<frost::round2::SignatureShare>)> {
        self.pk.verify(
            &self.sig,
            &shares_to_pb(self.shares.clone()).encode_to_vec(),
        )?;
        Ok((self.pk, self.shares))
    }
}

impl DomainType for FollowerRound2 {
    type Proto = pb::FollowerRound2;
}

/// Calculate the number of required signatures for a plan.
///
/// A plan can require more than one signature, hence the need for this method.
fn required_signatures(plan: &TransactionPlan) -> usize {
    plan.spend_plans().count() + plan.delegator_vote_plans().count()
}

pub struct CoordinatorState1 {
    plan: TransactionPlan,
    my_round1_reply: FollowerRound1,
    my_round1_state: FollowerState,
}

pub struct CoordinatorState2 {
    plan: TransactionPlan,
    my_round2_reply: FollowerRound2,
    effect_hash: EffectHash,
    signing_packages: Vec<frost::SigningPackage>,
}

pub struct FollowerState {
    plan: TransactionPlan,
    nonces: Vec<frost::round1::SigningNonces>,
}

pub fn coordinator_round1(
    rng: &mut impl CryptoRngCore,
    config: &Config,
    plan: TransactionPlan,
) -> Result<(CoordinatorRound1, CoordinatorState1)> {
    let message = CoordinatorRound1 { plan: plan.clone() };
    let (my_round1_reply, my_round1_state) = follower_round1(rng, config, message.clone())?;
    let state = CoordinatorState1 {
        plan,
        my_round1_reply,
        my_round1_state,
    };
    Ok((message, state))
}

pub fn coordinator_round2(
    config: &Config,
    state: CoordinatorState1,
    follower_messages: &[FollowerRound1],
) -> Result<(CoordinatorRound2, CoordinatorState2)> {
    let mut all_commitments = vec![BTreeMap::new(); required_signatures(&state.plan)];
    for message in follower_messages
        .iter()
        .cloned()
        .chain(iter::once(state.my_round1_reply))
    {
        let (pk, commitments) = message.checked_commitments()?;
        if !config.verification_keys().contains(&pk) {
            anyhow::bail!("unknown verification key: {:?}", pk);
        }
        // The public key acts as the identifier
        let identifier = frost::Identifier::derive(pk.as_bytes().as_slice())?;
        for (tree_i, com_i) in all_commitments.iter_mut().zip(commitments.into_iter()) {
            tree_i.insert(identifier, com_i);
        }
    }
    let reply = CoordinatorRound2 { all_commitments };

    let my_round2_reply = follower_round2(config, state.my_round1_state, reply.clone())?;
    let effect_hash = state.plan.effect_hash(config.fvk())?;
    let signing_packages = {
        reply
            .all_commitments
            .iter()
            .map(|tree| frost::SigningPackage::new(tree.clone(), effect_hash.as_ref()))
            .collect()
    };
    let state = CoordinatorState2 {
        plan: state.plan,
        my_round2_reply,
        effect_hash,
        signing_packages,
    };
    Ok((reply, state))
}

pub fn coordinator_round3(
    config: &Config,
    state: CoordinatorState2,
    follower_messages: &[FollowerRound2],
) -> Result<AuthorizationData> {
    let mut share_maps: Vec<HashMap<frost::Identifier, frost::round2::SignatureShare>> =
        vec![HashMap::new(); required_signatures(&state.plan)];
    for message in follower_messages
        .iter()
        .cloned()
        .chain(iter::once(state.my_round2_reply))
    {
        let (pk, shares) = message.checked_shares()?;
        if !config.verification_keys().contains(&pk) {
            anyhow::bail!("unknown verification key: {:?}", pk);
        }
        let identifier = frost::Identifier::derive(pk.as_bytes().as_slice())?;
        for (map_i, share_i) in share_maps.iter_mut().zip(shares.into_iter()) {
            map_i.insert(identifier, share_i);
        }
    }
    let mut spend_auths = state
        .plan
        .spend_plans()
        .map(|x| x.randomizer)
        .chain(state.plan.delegator_vote_plans().map(|x| x.randomizer))
        .zip(share_maps.iter())
        .zip(state.signing_packages.iter())
        .map(|((randomizer, share_map), signing_package)| {
            frost::aggregate_randomized(
                signing_package,
                &share_map,
                &config.public_key_package(),
                randomizer,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let delegator_vote_auths = spend_auths.split_off(state.plan.spend_plans().count());
    Ok(AuthorizationData {
        effect_hash: state.effect_hash,
        spend_auths,
        delegator_vote_auths,
    })
}

pub fn follower_round1(
    rng: &mut impl CryptoRngCore,
    config: &Config,
    coordinator: CoordinatorRound1,
) -> Result<(FollowerRound1, FollowerState)> {
    let required = required_signatures(&coordinator.plan);
    let (nonces, commitments) = (0..required)
        .map(|_| frost::round1::commit(&config.key_package().secret_share(), rng))
        .unzip();
    let reply = FollowerRound1::make(config.signing_key(), commitments);
    let state = FollowerState {
        plan: coordinator.plan,
        nonces,
    };
    Ok((reply, state))
}

pub fn follower_round2(
    config: &Config,
    state: FollowerState,
    coordinator: CoordinatorRound2,
) -> Result<FollowerRound2> {
    let effect_hash = state.plan.effect_hash(config.fvk())?;
    let signing_packages = coordinator
        .all_commitments
        .into_iter()
        .map(|tree| frost::SigningPackage::new(tree, effect_hash.as_ref()));
    let shares = state
        .plan
        .spend_plans()
        .map(|x| x.randomizer)
        .chain(state.plan.delegator_vote_plans().map(|x| x.randomizer))
        .zip(signing_packages)
        .zip(state.nonces.into_iter())
        .map(|((randomizer, signing_package), signer_nonces)| {
            frost::round2::sign_randomized(
                &signing_package,
                &signer_nonces,
                &config.key_package(),
                randomizer,
            )
        })
        .collect::<Result<_, _>>()?;
    Ok(FollowerRound2::make(config.signing_key(), shares))
}
