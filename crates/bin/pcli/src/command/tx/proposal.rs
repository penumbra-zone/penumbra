use anyhow::{Context, Result};

use penumbra_chain::params::ChainParameters;
use penumbra_transaction::{
    plan::TransactionPlan,
    proposal::{Proposal, ProposalPayload},
};

#[derive(Debug, clap::Subcommand)]
pub enum ProposalCmd {
    /// Make a template file for a new proposal.
    Template {
        /// The file to output the template to.
        #[clap(long, global = true)]
        file: Option<camino::Utf8PathBuf>,
        /// The kind of the proposal to template [one of: signaling, emergency, parameter-change, or dao-spend].
        #[clap(subcommand)]
        kind: ProposalKindCmd,
    },
    /// Submit a new governance proposal.
    Submit {
        /// The proposal to vote on, in TOML format.
        #[clap(long)]
        file: camino::Utf8PathBuf,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
    },
    /// Withdraw a governance proposal that you previously submitted.
    Withdraw {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
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
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
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
    /// Generate a template for a DAO spend proposal.
    DaoSpend {
        /// The transaction plan to include in the proposal, in JSON format.
        ///
        /// If not specified, the default empty transaction plan will be included, to be replaced
        /// in the template before submission.
        #[clap(long)]
        transaction_plan: Option<camino::Utf8PathBuf>,
    },
}

impl ProposalKindCmd {
    /// Generate a default proposal of a particular kind.
    pub fn template_proposal(&self, chain_params: &ChainParameters, id: u64) -> Result<Proposal> {
        let title = "A short title (at most 80 characters)".to_string();
        let description = "A longer description (at most 10,000 characters)".to_string();
        let payload = match self {
            ProposalKindCmd::Signaling => ProposalPayload::Signaling { commit: None },
            ProposalKindCmd::Emergency => ProposalPayload::Emergency { halt_chain: false },
            ProposalKindCmd::ParameterChange => ProposalPayload::ParameterChange {
                old: Box::new(chain_params.clone()),
                new: Box::new(chain_params.clone()),
            },
            ProposalKindCmd::DaoSpend { transaction_plan } => {
                if let Some(file) = transaction_plan {
                    ProposalPayload::DaoSpend {
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
                    ProposalPayload::DaoSpend {
                        transaction_plan: TransactionPlan::default(),
                    }
                }
            }
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
