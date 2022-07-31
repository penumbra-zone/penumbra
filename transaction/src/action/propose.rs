use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use penumbra_crypto::{value, Address, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::{transaction as pb, Protobuf};

use crate::{plan::TransactionPlan, AuthHash};

/// A governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Proposal", into = "pb::Proposal")]
pub struct Proposal {
    /// A natural-language description of the effect of the proposal and its justification.
    pub description: String,

    /// The specific kind and attributes of the proposal.
    pub kind: ProposalKind,
}

impl From<Proposal> for pb::Proposal {
    fn from(inner: Proposal) -> pb::Proposal {
        pb::Proposal {
            description: inner.description,
            kind: Some(inner.kind.into()),
        }
    }
}

impl TryFrom<pb::Proposal> for Proposal {
    type Error = anyhow::Error;

    fn try_from(inner: pb::Proposal) -> Result<Proposal, Self::Error> {
        Ok(Proposal {
            description: inner.description,
            kind: inner
                .kind
                .ok_or_else(|| anyhow::anyhow!("missing proposal kind"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb::Proposal> for Proposal {}

/// The machine-interpretable body of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::proposal::Kind", into = "pb::proposal::Kind")]
pub enum ProposalKind {
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
        schedule_transactions: Vec<(u64, TransactionPlan)>,
        /// Cancel these previously-scheduled transactions at the given heights.
        cancel_transactions: Vec<(u64, AuthHash)>,
    },
}

impl From<ProposalKind> for pb::proposal::Kind {
    fn from(value: ProposalKind) -> pb::proposal::Kind {
        match value {
            ProposalKind::Signaling { commit } => {
                pb::proposal::Kind::Signaling(pb::proposal::Signaling { commit })
            }
            ProposalKind::Emergency { halt_chain } => {
                pb::proposal::Kind::Emergency(pb::proposal::Emergency { halt_chain })
            }
            ProposalKind::ParameterChange {
                effective_height,
                new_parameters,
            } => pb::proposal::Kind::ParameterChange(pb::proposal::ParameterChange {
                effective_height,
                new_parameters: new_parameters
                    .into_iter()
                    .map(
                        |(parameter, value)| pb::proposal::parameter_change::SetParameter {
                            parameter,
                            value,
                        },
                    )
                    .collect(),
            }),
            ProposalKind::DaoSpend {
                schedule_transactions,
                cancel_transactions,
            } => pb::proposal::Kind::DaoSpend(pb::proposal::DaoSpend {
                schedule_transactions: schedule_transactions
                    .into_iter()
                    .map(|(execute_at_height, transaction)| {
                        pb::proposal::dao_spend::ScheduleTransaction {
                            execute_at_height,
                            transaction: Some(transaction.into()),
                        }
                    })
                    .collect(),
                cancel_transactions: cancel_transactions
                    .into_iter()
                    .map(|(scheduled_at_height, auth_hash)| {
                        pb::proposal::dao_spend::CancelTransaction {
                            scheduled_at_height,
                            auth_hash: Some(auth_hash.into()),
                        }
                    })
                    .collect(),
            }),
        }
    }
}

impl TryFrom<pb::proposal::Kind> for ProposalKind {
    type Error = anyhow::Error;

    fn try_from(msg: pb::proposal::Kind) -> Result<Self, Self::Error> {
        match msg {
            pb::proposal::Kind::Signaling(inner) => Ok(ProposalKind::Signaling {
                commit: inner.commit,
            }),
            pb::proposal::Kind::Emergency(inner) => Ok(ProposalKind::Emergency {
                halt_chain: inner.halt_chain,
            }),
            pb::proposal::Kind::ParameterChange(inner) => Ok(ProposalKind::ParameterChange {
                effective_height: inner.effective_height,
                new_parameters: inner
                    .new_parameters
                    .into_iter()
                    .map(|inner| (inner.parameter, inner.value))
                    .collect(),
            }),
            pb::proposal::Kind::DaoSpend(inner) => Ok(ProposalKind::DaoSpend {
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
                                .auth_hash
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
#[serde(try_from = "pb::ProposalSubmit", into = "pb::ProposalSubmit")]
pub struct ProposalSubmit {
    /// The proposal to propose.
    pub proposal: Proposal,
    /// The refund address for the proposal's proposer.
    pub deposit_refund_address: Address,
    /// The amount deposited for the proposal.
    pub deposit_amount: u64,
    /// The verification key to be used when withdrawing the proposal.
    pub withdraw_proposal_key: VerificationKey<SpendAuth>,
}

impl ProposalSubmit {
    /// Compute a commitment to the value contributed to a transaction by this proposal submission.
    pub fn value_commitment(&self) -> value::Commitment {
        let deposit = Value {
            amount: self.deposit_amount,
            asset_id: STAKING_TOKEN_ASSET_ID.clone(),
        }
        .commit(Fr::zero());

        let zero = Value {
            amount: 0,
            asset_id: STAKING_TOKEN_ASSET_ID.clone(),
        }
        .commit(Fr::zero());

        // Proposal submissions *require* the deposit amount in order to be accepted, so they
        // contribute (-deposit) to the value balance of the transaction
        zero - deposit
    }
}

impl From<ProposalSubmit> for pb::ProposalSubmit {
    fn from(value: ProposalSubmit) -> pb::ProposalSubmit {
        pb::ProposalSubmit {
            proposal: Some(value.proposal.into()),
            deposit_refund_address: Some(value.deposit_refund_address.into()),
            deposit_amount: value.deposit_amount,
            rk: value.withdraw_proposal_key.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::ProposalSubmit> for ProposalSubmit {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalSubmit) -> Result<Self, Self::Error> {
        Ok(ProposalSubmit {
            proposal: msg
                .proposal
                .ok_or_else(|| anyhow::anyhow!("missing proposal in `Propose`"))?
                .try_into()?,
            deposit_refund_address: msg
                .deposit_refund_address
                .ok_or_else(|| anyhow::anyhow!("missing deposit refund address in `Propose`"))?
                .try_into()?,
            deposit_amount: msg.deposit_amount,
            withdraw_proposal_key: <[u8; 32]>::try_from(msg.rk.to_vec())
                .map_err(|_| anyhow::anyhow!("invalid length for withdraw proposal key"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb::ProposalSubmit> for ProposalSubmit {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalWithdraw", into = "pb::ProposalWithdraw")]
pub struct ProposalWithdraw {
    /// The proposal withdraw body.
    pub body: ProposalWithdrawBody,
    /// The signature authorizing the withdrawal.
    pub auth_sig: Signature<SpendAuth>,
}

impl From<ProposalWithdraw> for pb::ProposalWithdraw {
    fn from(value: ProposalWithdraw) -> pb::ProposalWithdraw {
        pb::ProposalWithdraw {
            body: Some(value.body.into()),
            auth_sig: Some(value.auth_sig.into()),
        }
    }
}

impl TryFrom<pb::ProposalWithdraw> for ProposalWithdraw {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalWithdraw) -> Result<Self, Self::Error> {
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
    try_from = "pb::ProposalWithdrawBody",
    into = "pb::ProposalWithdrawBody"
)]
pub struct ProposalWithdrawBody {
    /// The proposal ID to withdraw.
    pub proposal: u64,
    /// The randomized proposal key from the original proposal.
    pub withdraw_proposal_key: VerificationKey<SpendAuth>,
}

impl From<ProposalWithdrawBody> for pb::ProposalWithdrawBody {
    fn from(value: ProposalWithdrawBody) -> pb::ProposalWithdrawBody {
        pb::ProposalWithdrawBody {
            proposal: value.proposal,
            rk: value.withdraw_proposal_key.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::ProposalWithdrawBody> for ProposalWithdrawBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalWithdrawBody) -> Result<Self, Self::Error> {
        Ok(ProposalWithdrawBody {
            proposal: msg.proposal,
            withdraw_proposal_key: msg.rk.to_vec()[..].try_into()?,
        })
    }
}

impl Protobuf<pb::ProposalWithdrawBody> for ProposalWithdrawBody {}
