use anyhow::Context;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use penumbra_chain::params::ChainParameters;
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType, TypeUrl};

/// A governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Proposal", into = "pb::Proposal")]
pub struct Proposal {
    /// The ID number of the proposal.
    pub id: u64,

    /// A short title describing the intent of the proposal.
    pub title: String,

    /// A natural-language description of the effect of the proposal and its justification.
    pub description: String,

    /// The specific kind and attributes of the proposal.
    pub payload: ProposalPayload,
}

/// The protobuf type URL for a transaction plan.
pub const TRANSACTION_PLAN_TYPE_URL: &str = "/penumbra.core.transaction.v1alpha1.TransactionPlan";

impl From<Proposal> for pb::Proposal {
    fn from(inner: Proposal) -> pb::Proposal {
        let mut proposal = pb::Proposal {
            id: inner.id,
            title: inner.title,
            description: inner.description,
            ..Default::default() // We're about to fill in precisely one of the fields for the payload
        };
        match inner.payload {
            ProposalPayload::Signaling { commit } => {
                proposal.signaling = Some(pb::proposal::Signaling {
                    commit: if let Some(c) = commit {
                        c
                    } else {
                        String::default()
                    },
                });
            }
            ProposalPayload::Emergency { halt_chain } => {
                proposal.emergency = Some(pb::proposal::Emergency { halt_chain });
            }
            ProposalPayload::ParameterChange { old, new } => {
                proposal.parameter_change = Some(pb::proposal::ParameterChange {
                    old_parameters: Some((*old).into()),
                    new_parameters: Some((*new).into()),
                });
            }
            ProposalPayload::DaoSpend { transaction_plan } => {
                proposal.dao_spend = Some(pb::proposal::DaoSpend {
                    transaction_plan: Some(pbjson_types::Any {
                        type_url: TRANSACTION_PLAN_TYPE_URL.to_owned(),
                        value: transaction_plan.into(),
                    }),
                });
            }
            ProposalPayload::UpgradePlan { height } => {
                proposal.upgrade_plan = Some(pb::proposal::UpgradePlan { height });
            }
        }
        proposal
    }
}

impl TryFrom<pb::Proposal> for Proposal {
    type Error = anyhow::Error;

    fn try_from(inner: pb::Proposal) -> Result<Proposal, Self::Error> {
        Ok(Proposal {
            id: inner.id,
            title: inner.title,
            description: inner.description,
            payload: if let Some(signaling) = inner.signaling {
                ProposalPayload::Signaling {
                    commit: if signaling.commit.is_empty() {
                        None
                    } else {
                        Some(signaling.commit)
                    },
                }
            } else if let Some(emergency) = inner.emergency {
                ProposalPayload::Emergency {
                    halt_chain: emergency.halt_chain,
                }
            } else if let Some(parameter_change) = inner.parameter_change {
                ProposalPayload::ParameterChange {
                    old: Box::new(
                        parameter_change
                            .old_parameters
                            .ok_or_else(|| anyhow::anyhow!("missing old parameters"))?
                            .try_into()?,
                    ),
                    new: Box::new(
                        parameter_change
                            .new_parameters
                            .ok_or_else(|| anyhow::anyhow!("missing new parameters"))?
                            .try_into()?,
                    ),
                }
            } else if let Some(dao_spend) = inner.dao_spend {
                ProposalPayload::DaoSpend {
                    transaction_plan: {
                        let transaction_plan = dao_spend
                            .transaction_plan
                            .ok_or_else(|| anyhow::anyhow!("missing transaction plan"))?;
                        if transaction_plan.type_url != TRANSACTION_PLAN_TYPE_URL {
                            anyhow::bail!(
                                "unknown transaction plan type url: {}",
                                transaction_plan.type_url
                            );
                        }
                        transaction_plan.value.to_vec()
                    },
                }
            } else if let Some(upgrade_plan) = inner.upgrade_plan {
                ProposalPayload::UpgradePlan {
                    height: upgrade_plan.height,
                }
            } else {
                anyhow::bail!("missing proposal payload or unknown proposal type");
            },
        })
    }
}

impl TypeUrl for Proposal {
    const TYPE_URL: &'static str = "/penumbra.core.governance.v1alpha1.Proposal";
}

impl DomainType for Proposal {
    type Proto = pb::Proposal;
}

/// A human-readable TOML-serializable version of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalToml {
    pub id: u64,
    pub title: String,
    pub description: String,
    #[serde(flatten)]
    pub payload: ProposalPayloadToml,
}

impl From<Proposal> for ProposalToml {
    fn from(proposal: Proposal) -> ProposalToml {
        ProposalToml {
            id: proposal.id,
            title: proposal.title,
            description: proposal.description,
            payload: proposal.payload.into(),
        }
    }
}

impl TryFrom<ProposalToml> for Proposal {
    type Error = anyhow::Error;

    fn try_from(proposal: ProposalToml) -> Result<Proposal, Self::Error> {
        Ok(Proposal {
            id: proposal.id,
            title: proposal.title,
            description: proposal.description,
            payload: proposal.payload.try_into()?,
        })
    }
}

/// The specific kind of a proposal.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
pub enum ProposalKind {
    /// A signaling proposal.
    #[cfg_attr(feature = "clap", clap(display_order = 100))]
    Signaling,
    /// An emergency proposal.
    #[cfg_attr(feature = "clap", clap(display_order = 200))]
    Emergency,
    /// A parameter change proposal.
    #[cfg_attr(feature = "clap", clap(display_order = 300))]
    ParameterChange,
    /// A DAO spend proposal.
    #[cfg_attr(feature = "clap", clap(display_order = 400))]
    DaoSpend,
    /// An upgrade proposal.
    #[cfg_attr(feature = "clap", clap(display_order = 500))]
    UpgradePlan,
}

impl FromStr for ProposalKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.replace(['-', '_', ' '], "").to_lowercase().as_str() {
            "signaling" => Ok(ProposalKind::Signaling),
            "emergency" => Ok(ProposalKind::Emergency),
            "parameterchange" => Ok(ProposalKind::ParameterChange),
            "daospend" => Ok(ProposalKind::DaoSpend),
            "upgrade_plan" => Ok(ProposalKind::UpgradePlan),
            _ => Err(anyhow::anyhow!("invalid proposal kind: {}", s)),
        }
    }
}

impl Proposal {
    /// Get the kind of a proposal.
    pub fn kind(&self) -> ProposalKind {
        match self.payload {
            ProposalPayload::Signaling { .. } => ProposalKind::Signaling,
            ProposalPayload::Emergency { .. } => ProposalKind::Emergency,
            ProposalPayload::ParameterChange { .. } => ProposalKind::ParameterChange,
            ProposalPayload::DaoSpend { .. } => ProposalKind::DaoSpend,
            ProposalPayload::UpgradePlan { .. } => ProposalKind::UpgradePlan,
        }
    }
}

/// The machine-interpretable body of a proposal.
#[derive(Debug, Clone)]
pub enum ProposalPayload {
    /// A signaling proposal is merely for coordination; it does not enact anything automatically by
    /// itself.
    Signaling {
        /// An optional commit hash for code that this proposal refers to.
        commit: Option<String>,
    },
    /// An emergency proposal is immediately passed when 2/3 of all validators approve it, without
    /// waiting for the voting period to conclude.
    Emergency {
        /// If `halt_chain == true`, then the chain will immediately halt when the proposal is
        /// passed.
        halt_chain: bool,
    },
    /// A parameter change proposal describes a replacement of the chain parameters, which should
    /// take effect when the proposal is passed.
    ParameterChange {
        /// The old chain parameters to be replaced.
        ///
        /// Even if the proposal passes, the update will not be applied if the chain parameters have
        /// changed *at all* from these chain parameters. Usually, this should be set to the current
        /// chain parameters at time of proposal.
        old: Box<ChainParameters>,
        /// The new chain parameters to be set.
        ///
        /// The *entire* chain parameters will be replaced with these at the time the proposal is
        /// passed.
        new: Box<ChainParameters>,
    },
    /// A DAO spend proposal describes proposed transaction(s) to be executed or cancelled at
    /// specific heights, with the spend authority of the DAO.
    DaoSpend {
        /// The transaction plan to be executed at the time the proposal is passed.
        ///
        /// This must be a transaction plan which can be executed by the DAO, which means it can't
        /// require any witness data or authorization signatures, but it may use the `DaoSpend`
        /// action.
        transaction_plan: Vec<u8>,
    },
    /// An upgrade plan proposal describes a planned upgrade to the chain. If ratified, the chain
    /// will halt at the specified height, trigger an epoch transition, and halt the chain.
    UpgradePlan { height: u64 },
}

/// A TOML-serializable version of `ProposalPayload`, meant for human consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProposalPayloadToml {
    Signaling {
        commit: Option<String>,
    },
    Emergency {
        halt_chain: bool,
    },
    ParameterChange {
        old: Box<ChainParameters>,
        new: Box<ChainParameters>,
    },
    DaoSpend {
        transaction: String,
    },
    UpgradePlan {
        height: u64,
    },
}

impl TryFrom<ProposalPayloadToml> for ProposalPayload {
    type Error = anyhow::Error;

    fn try_from(toml: ProposalPayloadToml) -> Result<Self, Self::Error> {
        Ok(match toml {
            ProposalPayloadToml::Signaling { commit } => ProposalPayload::Signaling { commit },
            ProposalPayloadToml::Emergency { halt_chain } => {
                ProposalPayload::Emergency { halt_chain }
            }
            ProposalPayloadToml::ParameterChange { old, new } => {
                ProposalPayload::ParameterChange { old, new }
            }
            ProposalPayloadToml::DaoSpend { transaction } => ProposalPayload::DaoSpend {
                transaction_plan: Bytes::from(
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, transaction)
                        .context("couldn't decode transaction plan from base64")?,
                )
                .to_vec(),
            },
            ProposalPayloadToml::UpgradePlan { height } => ProposalPayload::UpgradePlan { height },
        })
    }
}

impl From<ProposalPayload> for ProposalPayloadToml {
    fn from(payload: ProposalPayload) -> Self {
        match payload {
            ProposalPayload::Signaling { commit } => ProposalPayloadToml::Signaling { commit },
            ProposalPayload::Emergency { halt_chain } => {
                ProposalPayloadToml::Emergency { halt_chain }
            }
            ProposalPayload::ParameterChange { old, new } => {
                ProposalPayloadToml::ParameterChange { old, new }
            }
            ProposalPayload::DaoSpend { transaction_plan } => ProposalPayloadToml::DaoSpend {
                transaction: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    transaction_plan,
                ),
            },
            ProposalPayload::UpgradePlan { height } => ProposalPayloadToml::UpgradePlan { height },
        }
    }
}

impl ProposalPayload {
    pub fn is_signaling(&self) -> bool {
        matches!(self, ProposalPayload::Signaling { .. })
    }

    pub fn is_emergency(&self) -> bool {
        matches!(self, ProposalPayload::Emergency { .. })
    }

    pub fn is_parameter_change(&self) -> bool {
        matches!(self, ProposalPayload::ParameterChange { .. })
    }

    pub fn is_dao_spend(&self) -> bool {
        matches!(self, ProposalPayload::DaoSpend { .. })
    }
}
