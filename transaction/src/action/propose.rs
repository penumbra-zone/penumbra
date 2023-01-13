use ark_ff::Zero;
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

use penumbra_crypto::{asset::Amount, Address, Balance, Fr, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::{
    core::governance::v1alpha1 as pb_gov, core::transaction::v1alpha1 as pb_tx, Protobuf,
};

use crate::{plan::TransactionPlan, ActionView, EffectHash, IsAction, TransactionPerspective};

/// The protobuf type URL for a transaction plan.
pub const TRANSACTION_PLAN_TYPE_URL: &str = "/penumbra.core.transaction.v1alpha1.TransactionPlan";

/// A governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb_gov::Proposal", into = "pb_gov::Proposal")]
pub struct Proposal {
    /// A short title describing the intent of the proposal.
    pub title: String,

    /// A natural-language description of the effect of the proposal and its justification.
    pub description: String,

    /// The specific kind and attributes of the proposal.
    pub payload: ProposalPayload,
}

impl From<Proposal> for pb_gov::Proposal {
    fn from(inner: Proposal) -> pb_gov::Proposal {
        pb_gov::Proposal {
            title: inner.title,
            description: inner.description,
            payload: Some(inner.payload.into()),
        }
    }
}

impl TryFrom<pb_gov::Proposal> for Proposal {
    type Error = anyhow::Error;

    fn try_from(inner: pb_gov::Proposal) -> Result<Proposal, Self::Error> {
        Ok(Proposal {
            title: inner.title,
            description: inner.description,
            payload: inner
                .payload
                .ok_or_else(|| anyhow::anyhow!("missing proposal payload"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb_gov::Proposal> for Proposal {}

/// The specific kind of a proposal.
#[derive(Debug, Clone)]
pub enum ProposalKind {
    /// A signaling proposal.
    Signaling,
    /// An emergency proposal.
    Emergency,
    /// A parameter change proposal.
    ParameterChange,
    /// A DAO spend proposal.
    DaoSpend,
}

impl FromStr for ProposalKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.replace(['-', '_', ' '], "").to_lowercase().as_str() {
            "signaling" => Ok(ProposalKind::Signaling),
            "emergency" => Ok(ProposalKind::Emergency),
            "parameterchange" => Ok(ProposalKind::ParameterChange),
            "daospend" => Ok(ProposalKind::DaoSpend),
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
        }
    }
}

impl ProposalKind {
    /// Generate a default proposal of a particular kind.
    pub fn template_proposal(&self, chain_id: String) -> Proposal {
        let title = "A short title describing the intent of the proposal.".to_string();
        let description = "A human readable description of the proposal.".to_string();
        let payload = match self {
            ProposalKind::Signaling => ProposalPayload::Signaling { commit: None },
            ProposalKind::Emergency => ProposalPayload::Emergency { halt_chain: false },
            ProposalKind::ParameterChange => {
                let mut new_parameters = BTreeMap::new();
                new_parameters.insert(
                    "parameter name".to_string(),
                    "new parameter value".to_string(),
                );
                ProposalPayload::ParameterChange {
                    effective_height: 0,
                    new_parameters,
                }
            }
            ProposalKind::DaoSpend => ProposalPayload::DaoSpend {
                schedule_transactions: vec![(
                    0,
                    prost_types::Any {
                        type_url: TRANSACTION_PLAN_TYPE_URL.to_string(),
                        value: TransactionPlan {
                            chain_id,
                            ..Default::default()
                        }
                        .encode_to_vec(),
                    },
                )],
                cancel_transactions: vec![(0, EffectHash::default())],
            },
        };
        Proposal {
            title,
            description,
            payload,
        }
    }
}

/// The machine-interpretable body of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb_gov::proposal::Payload",
    into = "pb_gov::proposal::Payload"
)]
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
    /// A parameter change proposal describes changes to one or more chain parameters.
    ParameterChange {
        /// The parameter changes are enacted at this height.
        effective_height: u64,
        /// The parameter changes proposed, as a pair of string keys and string values.
        new_parameters: BTreeMap<String, String>,
    },
    /// A DAO spend proposal describes proposed transaction(s) to be executed or cancelled at
    /// specific heights, with the spend authority of the DAO.
    DaoSpend {
        /// Schedule these new transactions at the given heights.
        schedule_transactions: Vec<(u64, prost_types::Any)>,
        /// Cancel these previously-scheduled transactions at the given heights.
        cancel_transactions: Vec<(u64, EffectHash)>,
    },
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

impl Protobuf<pb_gov::proposal::Payload> for ProposalPayload {}

impl From<ProposalPayload> for pb_gov::proposal::Payload {
    fn from(value: ProposalPayload) -> pb_gov::proposal::Payload {
        pb_gov::proposal::Payload {
            payload: Some(match value {
                ProposalPayload::Signaling { commit } => {
                    pb_gov::proposal::payload::Payload::Signaling(pb_gov::proposal::Signaling {
                        commit,
                    })
                }
                ProposalPayload::Emergency { halt_chain } => {
                    pb_gov::proposal::payload::Payload::Emergency(pb_gov::proposal::Emergency {
                        halt_chain,
                    })
                }
                ProposalPayload::ParameterChange {
                    effective_height,
                    new_parameters,
                } => pb_gov::proposal::payload::Payload::ParameterChange(
                    pb_gov::proposal::ParameterChange {
                        effective_height,
                        new_parameters: new_parameters
                            .into_iter()
                            .map(|(parameter, value)| {
                                pb_gov::proposal::parameter_change::SetParameter {
                                    parameter,
                                    value,
                                }
                            })
                            .collect(),
                    },
                ),
                ProposalPayload::DaoSpend {
                    schedule_transactions,
                    cancel_transactions,
                } => pb_gov::proposal::payload::Payload::DaoSpend(pb_gov::proposal::DaoSpend {
                    schedule_transactions: schedule_transactions
                        .into_iter()
                        .map(|(execute_at_height, transaction)| {
                            pb_gov::proposal::dao_spend::ScheduleTransaction {
                                execute_at_height,
                                transaction: Some(transaction),
                            }
                        })
                        .collect(),
                    cancel_transactions: cancel_transactions
                        .into_iter()
                        .map(|(scheduled_at_height, effect_hash)| {
                            pb_gov::proposal::dao_spend::CancelTransaction {
                                scheduled_at_height,
                                effect_hash: Some(effect_hash.into()),
                            }
                        })
                        .collect(),
                }),
            }),
        }
    }
}

impl TryFrom<pb_gov::proposal::Payload> for ProposalPayload {
    type Error = anyhow::Error;

    fn try_from(msg: pb_gov::proposal::Payload) -> Result<Self, Self::Error> {
        let payload = msg
            .payload
            .ok_or_else(|| anyhow::anyhow!("missing proposal payload"))?;

        match payload {
            pb_gov::proposal::payload::Payload::Signaling(inner) => {
                Ok(ProposalPayload::Signaling {
                    commit: inner.commit,
                })
            }
            pb_gov::proposal::payload::Payload::Emergency(inner) => {
                Ok(ProposalPayload::Emergency {
                    halt_chain: inner.halt_chain,
                })
            }
            pb_gov::proposal::payload::Payload::ParameterChange(inner) => {
                Ok(ProposalPayload::ParameterChange {
                    effective_height: inner.effective_height,
                    new_parameters: inner
                        .new_parameters
                        .into_iter()
                        .map(|inner| (inner.parameter, inner.value))
                        .collect(),
                })
            }
            pb_gov::proposal::payload::Payload::DaoSpend(inner) => Ok(ProposalPayload::DaoSpend {
                schedule_transactions: inner
                    .schedule_transactions
                    .into_iter()
                    .map(|inner| {
                        Ok((
                            inner.execute_at_height,
                            inner
                                .transaction
                                .ok_or_else(|| {
                                    anyhow::anyhow!("missing transaction in `DaoSpend` schedule")
                                })?
                                .try_into()?,
                        ))
                    })
                    .collect::<Result<Vec<_>, anyhow::Error>>()?,
                cancel_transactions: inner
                    .cancel_transactions
                    .into_iter()
                    .map(|inner| {
                        Ok((
                            inner.scheduled_at_height,
                            inner
                                .effect_hash
                                .ok_or_else(|| {
                                    anyhow::anyhow!("missing auth hash in `DaoSpend` cancel")
                                })?
                                .try_into()?,
                        ))
                    })
                    .collect::<Result<Vec<_>, anyhow::Error>>()?,
            }),
        }
    }
}

/// A proposal submission describes the proposal to propose, and the (transparent, ephemeral) refund
/// address for the proposal deposit, along with a key to be used to verify the signature for a
/// withdrawal of that proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb_tx::ProposalSubmit", into = "pb_tx::ProposalSubmit")]
pub struct ProposalSubmit {
    /// The proposal to propose.
    pub proposal: Proposal,
    /// The refund address for the proposal's proposer.
    pub deposit_refund_address: Address,
    /// The amount deposited for the proposal.
    pub deposit_amount: Amount,
    /// The verification key to be used when withdrawing the proposal.
    pub withdraw_proposal_key: VerificationKey<SpendAuth>,
}

impl IsAction for ProposalSubmit {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ProposalSubmit(self.to_owned())
    }
}

impl ProposalSubmit {
    /// Compute a commitment to the value contributed to a transaction by this proposal submission.
    pub fn balance(&self) -> Balance {
        let deposit = Value {
            amount: self.deposit_amount,
            asset_id: STAKING_TOKEN_ASSET_ID.clone(),
        };

        // Proposal submissions *require* the deposit amount in order to be accepted, so they
        // contribute (-deposit) to the value balance of the transaction
        -Balance::from(deposit)
    }
}

impl From<ProposalSubmit> for pb_tx::ProposalSubmit {
    fn from(value: ProposalSubmit) -> pb_tx::ProposalSubmit {
        pb_tx::ProposalSubmit {
            proposal: Some(value.proposal.into()),
            deposit_refund_address: Some(value.deposit_refund_address.into()),
            deposit_amount: Some(value.deposit_amount.into()),
            rk: value.withdraw_proposal_key.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb_tx::ProposalSubmit> for ProposalSubmit {
    type Error = anyhow::Error;

    fn try_from(msg: pb_tx::ProposalSubmit) -> Result<Self, Self::Error> {
        Ok(ProposalSubmit {
            proposal: msg
                .proposal
                .ok_or_else(|| anyhow::anyhow!("missing proposal in `Propose`"))?
                .try_into()?,
            deposit_refund_address: msg
                .deposit_refund_address
                .ok_or_else(|| anyhow::anyhow!("missing deposit refund address in `Propose`"))?
                .try_into()?,
            deposit_amount: msg
                .deposit_amount
                .ok_or_else(|| anyhow::anyhow!("missing deposit amount in `Propose`"))?
                .try_into()?,
            withdraw_proposal_key: <[u8; 32]>::try_from(msg.rk.to_vec())
                .map_err(|_| anyhow::anyhow!("invalid length for withdraw proposal key"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb_tx::ProposalSubmit> for ProposalSubmit {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb_tx::ProposalWithdraw", into = "pb_tx::ProposalWithdraw")]
pub struct ProposalWithdraw {
    /// The proposal withdraw body.
    pub body: ProposalWithdrawBody,
    /// The signature authorizing the withdrawal.
    pub auth_sig: Signature<SpendAuth>,
}

impl IsAction for ProposalWithdraw {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        Default::default()
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ProposalWithdraw(self.to_owned())
    }
}

impl From<ProposalWithdraw> for pb_tx::ProposalWithdraw {
    fn from(value: ProposalWithdraw) -> pb_tx::ProposalWithdraw {
        pb_tx::ProposalWithdraw {
            body: Some(value.body.into()),
            auth_sig: Some(value.auth_sig.into()),
        }
    }
}

impl TryFrom<pb_tx::ProposalWithdraw> for ProposalWithdraw {
    type Error = anyhow::Error;

    fn try_from(msg: pb_tx::ProposalWithdraw) -> Result<Self, Self::Error> {
        Ok(ProposalWithdraw {
            body: msg
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body in `ProposalWithdraw`"))?
                .try_into()?,
            auth_sig: msg
                .auth_sig
                .ok_or_else(|| anyhow::anyhow!("missing auth sig in `ProposalWithdraw`"))?
                .try_into()?,
        })
    }
}

/// A withdraw-proposal body describes the original proposer's intent to withdraw their proposal
/// (this is the body, absent the signature).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb_tx::ProposalWithdrawBody",
    into = "pb_tx::ProposalWithdrawBody"
)]
pub struct ProposalWithdrawBody {
    /// The proposal ID to withdraw.
    pub proposal: u64,
    // The reason the proposal was withdrawn.
    pub reason: String,
}

impl From<ProposalWithdrawBody> for pb_tx::ProposalWithdrawBody {
    fn from(value: ProposalWithdrawBody) -> pb_tx::ProposalWithdrawBody {
        pb_tx::ProposalWithdrawBody {
            proposal: value.proposal,
            reason: value.reason,
        }
    }
}

impl TryFrom<pb_tx::ProposalWithdrawBody> for ProposalWithdrawBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb_tx::ProposalWithdrawBody) -> Result<Self, Self::Error> {
        Ok(ProposalWithdrawBody {
            proposal: msg.proposal,
            reason: msg.reason,
        })
    }
}

impl Protobuf<pb_tx::ProposalWithdrawBody> for ProposalWithdrawBody {}
