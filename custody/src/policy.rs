//! A set of basic spend authorization policies.

use std::collections::HashSet;

use penumbra_crypto::Address;
use penumbra_transaction::plan::ActionPlan;
use serde::{Deserialize, Serialize};

use crate::{AuthorizeRequest, PreAuthorization};

/// A trait for checking whether a transaction plan is allowed by a policy.
pub trait Policy {
    /// Checks whether the proposed transaction plan is allowed by this policy.
    fn check(&self, request: &AuthorizeRequest) -> Result<(), anyhow::Error>;
}

/// A set of basic spend authorization policies.
///
/// These policies are intended to be simple enough that they can be written by
/// hand in a config file.  More complex policy logic than than should be
/// implemented by a custom implementation of the [`Policy`] trait.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum AuthPolicy {
    /// Only allow transactions whose outputs are controlled by one of the
    /// allowed destination addresses.
    DestinationAllowList {
        allowed_destination_addresses: Vec<Address>,
    },
    /// Intended for relayers, only allows `Spend`, `Output`, and `IbcAction`
    /// actions in transactions.
    ///
    /// This policy should be combined with an `AllowList` to prevent sending
    /// funds outside of the relayer account.
    OnlyIbcRelay,
    /// Require specific pre-authorizations for submitted [`TransactionPlan`](penumbra_transaction::plan::TransactionPlan)s.
    PreAuthorization(PreAuthorizationPolicy),
}

/// A set of pre-authorization policies.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
// We need to use a different tag name here, so we can stack it with the
// SpendPolicy tag; in toml, for instance, this will turn into
// [[spend_policy]]
// type = 'PreAuthorization'
// method = 'Ed25519'
#[serde(tag = "method")]
pub enum PreAuthorizationPolicy {
    Ed25519 {
        /// The number of distinct pre-authorizations required to authorize a transaction plan.
        ///
        /// Each `allowed_signer`'s contributions count only once towards this total.
        required_signatures: u32,
        /// A list of pre-authorization keys that can be used to authorize a transaction plan.
        #[serde(with = "ed25519_vec_base64")]
        allowed_signers: Vec<ed25519_consensus::VerificationKey>,
    },
}

/// A serde helper to serialize pre-authorization keys as base64-encoded data.
/// Because Go's encoding/json will encode byte[] as base64-encoded strings,
/// and Go's Ed25519 keys are byte[] values, this hopefully makes it easier to
/// copy-paste pre-authorization keys from Go programs into the Rust config.
// TODO: remove this after <https://github.com/penumbra-zone/ed25519-consensus/issues/7>
mod ed25519_vec_base64 {
    pub fn serialize<S: serde::Serializer>(
        keys: &[ed25519_consensus::VerificationKey],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::Serialize;
        let mut base64_keys = Vec::with_capacity(keys.len());
        for key in keys {
            base64_keys.push(base64::encode(key.as_bytes()));
        }
        base64_keys.serialize(serializer)
    }
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Vec<ed25519_consensus::VerificationKey>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::Deserialize;
        let base64_keys: Vec<String> = Vec::deserialize(deserializer)?;
        let mut vks = Vec::with_capacity(base64_keys.len());
        for base64_key in base64_keys {
            let bytes = base64::decode(base64_key).map_err(serde::de::Error::custom)?;
            let vk = ed25519_consensus::VerificationKey::try_from(bytes.as_slice())
                .map_err(serde::de::Error::custom)?;
            vks.push(vk);
        }
        Ok(vks)
    }
}

impl Policy for AuthPolicy {
    fn check(&self, request: &AuthorizeRequest) -> Result<(), anyhow::Error> {
        let plan = &request.plan;
        match self {
            AuthPolicy::DestinationAllowList {
                allowed_destination_addresses,
            } => {
                for output in plan.output_plans() {
                    if !allowed_destination_addresses.contains(&output.dest_address) {
                        return Err(anyhow::anyhow!(
                            "output {:?} has dest_address not in allow list",
                            output
                        ));
                    }
                }
                for swap in plan.swap_plans() {
                    if !allowed_destination_addresses.contains(&swap.swap_plaintext.claim_address) {
                        return Err(anyhow::anyhow!(
                            "swap {:?} has claim_address not in allow list",
                            swap
                        ));
                    }
                }
                Ok(())
            }
            AuthPolicy::OnlyIbcRelay => {
                for action in &plan.actions {
                    match action {
                        ActionPlan::Spend { .. }
                        | ActionPlan::Output { .. }
                        | ActionPlan::IBCAction { .. } => {}
                        _ => {
                            return Err(anyhow::anyhow!(
                                "action {:?} not allowed by OnlyRelay policy",
                                action
                            ));
                        }
                    }
                }
                Ok(())
            }
            AuthPolicy::PreAuthorization(policy) => policy.check(request),
        }
    }
}

impl Policy for PreAuthorizationPolicy {
    fn check(&self, request: &AuthorizeRequest) -> Result<(), anyhow::Error> {
        match self {
            PreAuthorizationPolicy::Ed25519 {
                required_signatures,
                allowed_signers,
            } => {
                let ed25519_pre_auths =
                    request
                        .pre_authorizations
                        .iter()
                        .filter_map(|pre_auth| match pre_auth {
                            PreAuthorization::Ed25519(pre_auth) => Some(pre_auth),
                            // _ => None,
                        });

                let mut allowed_signers = allowed_signers.iter().cloned().collect::<HashSet<_>>();
                let mut seen_signers = HashSet::new();

                for pre_auth in ed25519_pre_auths {
                    // Remove the signer from the allowed signers set, so that
                    // each signer can only submit one pre-authorization.
                    if let Some(signer) = allowed_signers.take(&pre_auth.vk) {
                        pre_auth.verify_plan(&request.plan)?;
                        seen_signers.insert(signer);
                    }
                }

                if seen_signers.len() < *required_signatures as usize {
                    return Err(anyhow::anyhow!(
                        "required {} pre-authorization signatures but only saw {}",
                        required_signatures,
                        seen_signers.len(),
                    ));
                }
                Ok(())
            }
        }
    }
}
