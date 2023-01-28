use ark_ff::Zero;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

use penumbra_crypto::{
    asset::{self, Amount, Denom},
    balance, Balance, Fr, ProposalNft, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{core::governance::v1alpha1 as pb, Protobuf};

use crate::{plan::TransactionPlan, ActionView, EffectHash, IsAction, TransactionPerspective};

/// The protobuf type URL for a transaction plan.
pub const TRANSACTION_PLAN_TYPE_URL: &str = "/penumbra.core.transaction.v1alpha1.TransactionPlan";

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
                proposal.signaling = Some(pb::proposal::Signaling { commit });
            }
            ProposalPayload::Emergency { halt_chain } => {
                proposal.emergency = Some(pb::proposal::Emergency { halt_chain });
            }
            ProposalPayload::ParameterChange {
                effective_height,
                new_parameters,
            } => {
                proposal.parameter_change =
                    Some(pb::proposal::ParameterChange {
                        effective_height,
                        new_parameters: new_parameters
                            .into_iter()
                            .map(|(parameter, value)| {
                                pb::proposal::parameter_change::SetParameter { parameter, value }
                            })
                            .collect(),
                    });
            }
            ProposalPayload::DaoSpend {
                schedule_transactions,
                cancel_transactions,
            } => {
                proposal.dao_spend = Some(pb::proposal::DaoSpend {
                    schedule_transactions: schedule_transactions
                        .into_iter()
                        .map(|(execute_at_height, transaction)| {
                            pb::proposal::dao_spend::ScheduleTransaction {
                                execute_at_height,
                                transaction: Some(transaction),
                            }
                        })
                        .collect(),
                    cancel_transactions: cancel_transactions
                        .into_iter()
                        .map(|(scheduled_at_height, effect_hash)| {
                            pb::proposal::dao_spend::CancelTransaction {
                                scheduled_at_height,
                                effect_hash: Some(effect_hash.into()),
                            }
                        })
                        .collect(),
                });
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
                    commit: signaling.commit,
                }
            } else if let Some(emergency) = inner.emergency {
                ProposalPayload::Emergency {
                    halt_chain: emergency.halt_chain,
                }
            } else if let Some(parameter_change) = inner.parameter_change {
                ProposalPayload::ParameterChange {
                    effective_height: parameter_change.effective_height,
                    new_parameters: parameter_change
                        .new_parameters
                        .into_iter()
                        .map(|set_parameter| (set_parameter.parameter, set_parameter.value))
                        .collect(),
                }
            } else if let Some(dao_spend) = inner.dao_spend {
                ProposalPayload::DaoSpend {
                        schedule_transactions: dao_spend
                            .schedule_transactions
                            .into_iter()
                            .map(|schedule_transaction| {
                                Ok::<_, anyhow::Error>((
                                    schedule_transaction.execute_at_height,
                                    schedule_transaction.transaction.ok_or_else(|| {
                                        anyhow::anyhow!("missing transaction in scheduled transaction")
                                    })?,
                                ))
                            })
                            .collect::<Result<_, _>>()?,
                        cancel_transactions: dao_spend
                            .cancel_transactions
                            .into_iter()
                            .map(|cancel_transaction| {
                                Ok::<_, anyhow::Error>((
                                    cancel_transaction.scheduled_at_height,
                                    cancel_transaction
                                        .effect_hash
                                        .ok_or_else(|| {
                                            anyhow::anyhow!("missing effect hash in cancellation of scheduled transaction")
                                        })?
                                        .try_into()?,
                                ))
                            })
                            .collect::<Result<_, _>>()?,
                    }
            } else {
                return Err(anyhow::anyhow!(
                    "missing proposal payload or unknown proposal type"
                ));
            },
        })
    }
}

impl Protobuf<pb::Proposal> for Proposal {}

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
    pub fn template_proposal(&self, chain_id: String, id: u64) -> Proposal {
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
                    pbjson_types::Any {
                        type_url: TRANSACTION_PLAN_TYPE_URL.to_string(),
                        value: TransactionPlan {
                            chain_id,
                            ..Default::default()
                        }
                        .encode_to_vec()
                        .into(),
                    },
                )],
                cancel_transactions: vec![(0, EffectHash::default())],
            },
        };
        Proposal {
            id,
            title,
            description,
            payload,
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
        schedule_transactions: Vec<(u64, pbjson_types::Any)>,
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

/// A proposal submission describes the proposal to propose, and the (transparent, ephemeral) refund
/// address for the proposal deposit, along with a key to be used to verify the signature for a
/// withdrawal of that proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalSubmit", into = "pb::ProposalSubmit")]
pub struct ProposalSubmit {
    /// The proposal to propose.
    pub proposal: Proposal,
    /// The amount deposited for the proposal.
    pub deposit_amount: Amount,
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

        let proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::voting(self.proposal.id).denom().into(),
        };

        // Proposal submissions *require* the deposit amount in order to be accepted, so they
        // contribute (-deposit) to the value balance of the transaction, and they contribute a
        // single proposal NFT to the value balance:
        Balance::from(proposal_nft) - Balance::from(deposit)
    }
}

impl From<ProposalSubmit> for pb::ProposalSubmit {
    fn from(value: ProposalSubmit) -> pb::ProposalSubmit {
        pb::ProposalSubmit {
            proposal: Some(value.proposal.into()),
            deposit_amount: Some(value.deposit_amount.into()),
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
            deposit_amount: msg
                .deposit_amount
                .ok_or_else(|| anyhow::anyhow!("missing deposit amount in `Propose`"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb::ProposalSubmit> for ProposalSubmit {}

impl IsAction for ProposalWithdraw {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ProposalWithdraw(self.to_owned())
    }
}

impl ProposalWithdraw {
    /// Compute a commitment to the value contributed to a transaction by this proposal submission.
    pub fn balance(&self) -> Balance {
        let voting_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::voting(self.proposal).denom().into(),
        };
        let withdrawn_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: ProposalNft::withdrawn(self.proposal).denom().into(),
        };

        // Proposal withdrawals consume the submitted proposal and produce a withdrawn proposal:
        Balance::from(withdrawn_proposal_nft) - Balance::from(voting_proposal_nft)
    }
}

/// A withdrawal of a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalWithdraw", into = "pb::ProposalWithdraw")]
pub struct ProposalWithdraw {
    /// The proposal ID to withdraw.
    pub proposal: u64,
    // The reason the proposal was withdrawn.
    pub reason: String,
}

impl From<ProposalWithdraw> for pb::ProposalWithdraw {
    fn from(value: ProposalWithdraw) -> pb::ProposalWithdraw {
        pb::ProposalWithdraw {
            proposal: value.proposal,
            reason: value.reason,
        }
    }
}

impl TryFrom<pb::ProposalWithdraw> for ProposalWithdraw {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalWithdraw) -> Result<Self, Self::Error> {
        Ok(ProposalWithdraw {
            proposal: msg.proposal,
            reason: msg.reason,
        })
    }
}

impl Protobuf<pb::ProposalWithdraw> for ProposalWithdraw {}

/// A claim for the initial submission deposit for a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ProposalDepositClaim",
    into = "pb::ProposalDepositClaim"
)]
pub struct ProposalDepositClaim {
    /// The proposal ID to claim the deposit for.
    pub proposal: u64,
    /// The amount of the deposit.
    pub deposit_amount: Amount,
    /// The outcome of the proposal.
    pub outcome: Outcome<()>,
}

impl From<ProposalDepositClaim> for pb::ProposalDepositClaim {
    fn from(value: ProposalDepositClaim) -> pb::ProposalDepositClaim {
        pb::ProposalDepositClaim {
            proposal: value.proposal,
            deposit_amount: Some(value.deposit_amount.into()),
            outcome: Some(value.outcome.into()),
        }
    }
}

impl TryFrom<pb::ProposalDepositClaim> for ProposalDepositClaim {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalDepositClaim) -> Result<Self, Self::Error> {
        Ok(ProposalDepositClaim {
            proposal: msg.proposal,
            deposit_amount: msg
                .deposit_amount
                .ok_or_else(|| anyhow::anyhow!("missing deposit amount in `ProposalDepositClaim`"))?
                .try_into()?,
            outcome: msg
                .outcome
                .ok_or_else(|| anyhow::anyhow!("missing outcome in `ProposalDepositClaim`"))?
                .try_into()?,
        })
    }
}

impl IsAction for ProposalDepositClaim {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ProposalDepositClaim(self.clone())
    }
}

impl ProposalDepositClaim {
    /// Compute the balance contributed to the transaction by this proposal deposit claim.
    pub fn balance(&self) -> Balance {
        let deposit = Value {
            amount: self.deposit_amount,
            asset_id: STAKING_TOKEN_ASSET_ID.clone(),
        };

        let (voting_or_withdrawn_proposal_denom, claimed_proposal_denom): (Denom, Denom) =
            match self.outcome {
                Outcome::Passed => (
                    ProposalNft::voting(self.proposal).denom(),
                    ProposalNft::passed(self.proposal).denom(),
                ),
                Outcome::Failed {
                    withdrawn: Withdrawn::No,
                } => (
                    ProposalNft::voting(self.proposal).denom(),
                    ProposalNft::failed(self.proposal).denom(),
                ),
                Outcome::Failed {
                    withdrawn: Withdrawn::WithReason { .. },
                } => (
                    ProposalNft::withdrawn(self.proposal).denom(),
                    ProposalNft::failed(self.proposal).denom(),
                ),
                Outcome::Vetoed {
                    withdrawn: Withdrawn::No,
                } => (
                    ProposalNft::voting(self.proposal).denom(),
                    ProposalNft::vetoed(self.proposal).denom(),
                ),
                Outcome::Vetoed {
                    withdrawn: Withdrawn::WithReason { .. },
                } => (
                    ProposalNft::withdrawn(self.proposal).denom(),
                    ProposalNft::vetoed(self.proposal).denom(),
                ),
            };

        // NFT to be consumed
        let voting_or_withdrawn_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: asset::Id::from(voting_or_withdrawn_proposal_denom),
        };

        // NFT to be created
        let claimed_proposal_nft = Value {
            amount: Amount::from(1u64),
            asset_id: asset::Id::from(claimed_proposal_denom),
        };

        // Proposal deposit claims consume the submitted or withdrawn proposal and produce a claimed
        // proposal and the deposit:
        let mut balance =
            Balance::from(claimed_proposal_nft) - Balance::from(voting_or_withdrawn_proposal_nft);

        // Only issue a refund if the proposal was not vetoed
        if self.outcome.should_be_refunded() {
            balance += Balance::from(deposit);
        }

        balance
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalState", into = "pb::ProposalState")]
pub enum State {
    Voting,
    Withdrawn { reason: String },
    Finished { outcome: Outcome<String> },
    Claimed { outcome: Outcome<String> },
}

impl State {
    pub fn withdrawn(self) -> Withdrawn<String> {
        match self {
            State::Voting => Withdrawn::No,
            State::Withdrawn { reason } => Withdrawn::WithReason { reason },
            State::Finished { outcome } => match outcome {
                Outcome::Passed => Withdrawn::No,
                Outcome::Failed { withdrawn } | Outcome::Vetoed { withdrawn } => withdrawn,
            },
            State::Claimed { outcome } => match outcome {
                Outcome::Passed => Withdrawn::No,
                Outcome::Failed { withdrawn } | Outcome::Vetoed { withdrawn } => withdrawn,
            },
        }
    }
}

impl Protobuf<pb::ProposalState> for State {}

impl From<State> for pb::ProposalState {
    fn from(s: State) -> Self {
        let state = match s {
            State::Voting => pb::proposal_state::State::Voting(pb::proposal_state::Voting {}),
            State::Withdrawn { reason } => {
                pb::proposal_state::State::Withdrawn(pb::proposal_state::Withdrawn { reason })
            }
            State::Finished { outcome } => {
                pb::proposal_state::State::Finished(pb::proposal_state::Finished {
                    outcome: Some(outcome.into()),
                })
            }
            State::Claimed { outcome } => {
                pb::proposal_state::State::Finished(pb::proposal_state::Finished {
                    outcome: Some(outcome.into()),
                })
            }
        };
        pb::ProposalState { state: Some(state) }
    }
}

impl TryFrom<pb::ProposalState> for State {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalState) -> Result<Self, Self::Error> {
        Ok(
            match msg
                .state
                .ok_or_else(|| anyhow::anyhow!("missing proposal state"))?
            {
                pb::proposal_state::State::Voting(pb::proposal_state::Voting {}) => State::Voting,
                pb::proposal_state::State::Withdrawn(pb::proposal_state::Withdrawn { reason }) => {
                    State::Withdrawn { reason }
                }
                pb::proposal_state::State::Finished(pb::proposal_state::Finished { outcome }) => {
                    State::Finished {
                        outcome: outcome
                            .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
                            .try_into()?,
                    }
                }
                pb::proposal_state::State::Claimed(pb::proposal_state::Claimed { outcome }) => {
                    State::Claimed {
                        outcome: outcome
                            .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
                            .try_into()?,
                    }
                }
            },
        )
    }
}

// This is parameterized by `W`, the withdrawal reason, so that we can use `()` where a reason
// doesn't need to be specified. When this is the case, the serialized format in protobufs uses an
// empty string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    try_from = "pb::ProposalOutcome",
    into = "pb::ProposalOutcome",
    bound = "W: Clone, pb::ProposalOutcome: From<Outcome<W>>, Outcome<W>: TryFrom<pb::ProposalOutcome, Error = anyhow::Error>"
)]
pub enum Outcome<W> {
    Passed,
    Failed { withdrawn: Withdrawn<W> },
    Vetoed { withdrawn: Withdrawn<W> },
}

impl<W> Outcome<W> {
    /// Determines if the outcome should be refunded (i.e. it was not vetoed).
    pub fn should_be_refunded(&self) -> bool {
        !self.is_vetoed()
    }

    pub fn is_vetoed(&self) -> bool {
        matches!(self, Outcome::Vetoed { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Outcome::Failed { .. } | Outcome::Vetoed { .. })
    }

    pub fn is_passed(&self) -> bool {
        matches!(self, Outcome::Passed)
    }

    pub fn as_ref(&self) -> Outcome<&W> {
        match self {
            Outcome::Passed => Outcome::Passed,
            Outcome::Failed { withdrawn } => Outcome::Failed {
                withdrawn: withdrawn.as_ref(),
            },
            Outcome::Vetoed { withdrawn } => Outcome::Vetoed {
                withdrawn: withdrawn.as_ref(),
            },
        }
    }

    pub fn map<X>(self, f: impl FnOnce(W) -> X) -> Outcome<X> {
        match self {
            Outcome::Passed => Outcome::Passed,
            Outcome::Failed { withdrawn } => Outcome::Failed {
                withdrawn: Option::from(withdrawn).map(f).into(),
            },
            Outcome::Vetoed { withdrawn } => Outcome::Vetoed {
                withdrawn: Option::from(withdrawn).map(f).into(),
            },
        }
    }
}

// This is parameterized by `W`, the withdrawal reason, so that we can use `()` where a reason
// doesn't need to be specified. When this is the case, the serialized format in protobufs uses an
// empty string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Withdrawn<W> {
    No,
    WithReason { reason: W },
}

impl<W> Withdrawn<W> {
    pub fn as_ref(&self) -> Withdrawn<&W> {
        match self {
            Withdrawn::No => Withdrawn::No,
            Withdrawn::WithReason { reason } => Withdrawn::WithReason { reason },
        }
    }
}

impl<W> From<Option<W>> for Withdrawn<W> {
    fn from(reason: Option<W>) -> Self {
        match reason {
            Some(reason) => Withdrawn::WithReason { reason },
            None => Withdrawn::No,
        }
    }
}

impl<W> From<Withdrawn<W>> for Option<W> {
    fn from(withdrawn: Withdrawn<W>) -> Self {
        match withdrawn {
            Withdrawn::No => None,
            Withdrawn::WithReason { reason } => Some(reason),
        }
    }
}

impl TryFrom<Withdrawn<String>> for Withdrawn<()> {
    type Error = anyhow::Error;

    fn try_from(withdrawn: Withdrawn<String>) -> Result<Self, Self::Error> {
        Ok(match withdrawn {
            Withdrawn::No => Withdrawn::No,
            Withdrawn::WithReason { reason } => {
                if reason.is_empty() {
                    Withdrawn::WithReason { reason: () }
                } else {
                    anyhow::bail!("withdrawn reason is not empty")
                }
            }
        })
    }
}

impl Protobuf<pb::ProposalOutcome> for Outcome<String> {}

impl From<Outcome<String>> for pb::ProposalOutcome {
    fn from(o: Outcome<String>) -> Self {
        let outcome = match o {
            Outcome::Passed => {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {})
            }
            Outcome::Failed { withdrawn } => {
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn_with_reason: withdrawn.into(),
                })
            }
            Outcome::Vetoed { withdrawn } => {
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn_with_reason: withdrawn.into(),
                })
            }
        };
        pb::ProposalOutcome {
            outcome: Some(outcome),
        }
    }
}

impl TryFrom<pb::ProposalOutcome> for Outcome<String> {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalOutcome) -> Result<Self, Self::Error> {
        Ok(
            match msg
                .outcome
                .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
            {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {}) => {
                    Outcome::Passed
                }
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn_with_reason,
                }) => Outcome::Failed {
                    withdrawn: withdrawn_with_reason.into(),
                },
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn_with_reason,
                }) => Outcome::Vetoed {
                    withdrawn: withdrawn_with_reason.into(),
                },
            },
        )
    }
}

impl Protobuf<pb::ProposalOutcome> for Outcome<()> {}

impl From<Outcome<()>> for pb::ProposalOutcome {
    fn from(o: Outcome<()>) -> Self {
        let outcome = match o {
            Outcome::Passed => {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {})
            }
            Outcome::Failed { withdrawn } => {
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn_with_reason: <Option<()>>::from(withdrawn).map(|()| "".to_string()),
                })
            }
            Outcome::Vetoed { withdrawn } => {
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn_with_reason: <Option<()>>::from(withdrawn).map(|()| "".to_string()),
                })
            }
        };
        pb::ProposalOutcome {
            outcome: Some(outcome),
        }
    }
}

impl TryFrom<pb::ProposalOutcome> for Outcome<()> {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalOutcome) -> Result<Self, Self::Error> {
        Ok(
            match msg
                .outcome
                .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
            {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {}) => {
                    Outcome::Passed
                }
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn_with_reason,
                }) => Outcome::Failed {
                    withdrawn: <Withdrawn<String>>::from(withdrawn_with_reason).try_into()?,
                },
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn_with_reason,
                }) => Outcome::Vetoed {
                    withdrawn: <Withdrawn<String>>::from(withdrawn_with_reason).try_into()?,
                },
            },
        )
    }
}
