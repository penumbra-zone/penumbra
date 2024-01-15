use anyhow::{Context, Result};

use penumbra_app::params::AppParameters;
use penumbra_governance::{proposal::ChangedAppParameters, Proposal, ProposalPayload};
use penumbra_proto::DomainType;
use penumbra_transaction::plan::TransactionPlan;

#[derive(Debug, clap::Subcommand)]
pub enum ProposalCmd {
    /// Make a template file for a new proposal.
    Template {
        /// The file to output the template to.
        #[clap(long, global = true)]
        file: Option<camino::Utf8PathBuf>,
        /// The kind of the proposal to template [one of: signaling, emergency, parameter-change, or community-pool-spend].
        #[clap(subcommand)]
        kind: ProposalKindCmd,
    },
    /// Submit a new governance proposal.
    Submit {
        /// The proposal to vote on, in TOML format.
        #[clap(long)]
        file: camino::Utf8PathBuf,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The amount of the staking token to deposit alongside the proposal.
        #[clap(long)]
        deposit_amount: u64,
    },
    /// Withdraw a governance proposal that you previously submitted.
    Withdraw {
        /// The proposal id to withdraw.
        proposal_id: u64,
        /// A short description of the reason for the proposal being withdrawn, meant to be
        /// displayed to users.
        #[clap(long)]
        reason: String,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
    },
    /// Claim a governance proposal deposit for a proposal you submitted that has finished voting.
    ///
    /// This consumes the voting or withdrawn proposal NFT and mints an NFT representing whether the
    /// proposal passed, failed, or was slashed. In the case of a slash, the deposit is not returned
    /// by this action; in other cases, it is returned to you.
    DepositClaim {
        /// The proposal id to claim the deposit for.
        proposal_id: u64,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum ProposalKindCmd {
    /// Generate a template for a signaling proposal.
    Signaling,
    /// Generate a template for an emergency proposal.
    Emergency,
    /// Generate a template for a parameter change proposal.
    ParameterChange,
    /// Generate a template for a Community Pool spend proposal.
    CommunityPoolSpend {
        /// The transaction plan to include in the proposal, in JSON format.
        ///
        /// If not specified, the default empty transaction plan will be included, to be replaced
        /// in the template before submission.
        #[clap(long)]
        transaction_plan: Option<camino::Utf8PathBuf>,
    },
    /// Generate a template for an upgrade propopsal,
    UpgradePlan,
}

impl ProposalKindCmd {
    /// Generate a default proposal of a particular kind.
    pub fn template_proposal(&self, app_params: &AppParameters, id: u64) -> Result<Proposal> {
        let title = "A short title (at most 80 characters)".to_string();
        let description = "A longer description (at most 10,000 characters)".to_string();
        let payload = match self {
            ProposalKindCmd::Signaling => ProposalPayload::Signaling { commit: None },
            ProposalKindCmd::Emergency => ProposalPayload::Emergency { halt_chain: false },
            ProposalKindCmd::ParameterChange => ProposalPayload::ParameterChange {
                old: Box::new(app_params.as_changed_params()),
                new: Box::new(ChangedAppParameters {
                    chain_params: None,
                    community_pool_params: None,
                    ibc_params: None,
                    stake_params: None,
                    fee_params: None,
                    governance_params: None,
                    distributions_params: None,
                }),
            },
            ProposalKindCmd::CommunityPoolSpend { transaction_plan } => {
                if let Some(file) = transaction_plan {
                    ProposalPayload::CommunityPoolSpend {
                        transaction_plan: serde_json::from_reader(
                            std::fs::File::open(file).with_context(|| {
                                format!("Failed to open transaction plan file {:?}", file)
                            })?,
                        )
                        .with_context(|| {
                            format!("Failed to parse transaction plan file {:?}", file)
                        })?,
                    }
                } else {
                    ProposalPayload::CommunityPoolSpend {
                        transaction_plan: TransactionPlan::default().encode_to_vec(),
                    }
                }
            }
            ProposalKindCmd::UpgradePlan { .. } => ProposalPayload::UpgradePlan { height: 0 },
        };

        Ok(Proposal {
            id,
            title,
            description,
            payload,
        })
    }
}

impl ProposalCmd {
    pub fn offline(&self) -> bool {
        match self {
            ProposalCmd::Template { .. } => false,
            ProposalCmd::Submit { .. } => false,
            ProposalCmd::Withdraw { .. } => false,
            ProposalCmd::DepositClaim { .. } => false,
        }
    }
}
