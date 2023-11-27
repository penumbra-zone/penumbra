use std::{
    collections::{BTreeMap, HashMap},
    iter,
};

use anyhow::{anyhow, Result};
use decaf377_frost as frost;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use frost::round1::SigningCommitments;
use penumbra_chain::EffectHash;
use penumbra_proto::{penumbra::custody::threshold::v1alpha1 as pb, DomainType, Message};
use penumbra_transaction::{plan::TransactionPlan, AuthorizationData};
use rand_core::CryptoRngCore;

use super::config::Config;

/// Represents the message sent by the coordinator at the start of the signing process.
///
/// This is nominally "round 1", even though it's the only message the coordinator ever sends.
#[derive(Debug, Clone)]
pub struct CoordinatorRound1 {
    plan: TransactionPlan,
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
