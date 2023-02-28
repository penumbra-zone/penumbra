use penumbra_transaction::action::ProposalKind;

#[derive(Debug, clap::Subcommand)]
pub enum ProposalCmd {
    /// Make a template file for a new proposal.
    Template {
        /// The file to output the template to.
        #[clap(long, global = true)]
        file: Option<camino::Utf8PathBuf>,
        /// The kind of the proposal to template [one of: signaling, emergency, parameter-change, or dao-spend].
        #[clap(subcommand)]
        kind: ProposalKind,
    },
    /// Submit a new governance proposal.
    Submit {
        /// The proposal to vote on, in JSON format.
        #[clap(long)]
        file: camino::Utf8PathBuf,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
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
        /// Only spend funds originally received by the given address index.
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
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
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
