use std::io::{IsTerminal, Read, Write};

use anyhow::Result;
use decaf377::{Element, Fq};
use decaf377_rdsa::{Domain, Signature, VerificationKey};
use penumbra_sdk_asset::{asset::Cache, balance::Commitment};
use penumbra_sdk_custody::threshold::{SigningRequest, Terminal};
use penumbra_sdk_keys::{
    symmetric::{OvkWrappedKey, WrappedMemoKey},
    FullViewingKey, PayloadKey,
};
use penumbra_sdk_proof_params::GROTH16_PROOF_LENGTH_BYTES;
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_shielded_pool::{EncryptedBackref, Note, NoteView};
use penumbra_sdk_tct::structure::Hash;
use penumbra_sdk_transaction::{view, ActionPlan, ActionView, TransactionPlan, TransactionView};
use termion::{color, input::TermRead};
use tonic::async_trait;

use crate::transaction_view_ext::TransactionViewExt as _;

async fn read_password(prompt: &str) -> Result<String> {
    fn get_possibly_empty_string(prompt: &str) -> Result<String> {
        // The `rpassword` crate doesn't support reading from stdin, so we check
        // for an interactive session. We must support non-interactive use cases,
        // for integration with other tooling.
        if std::io::stdin().is_terminal() {
            Ok(rpassword::prompt_password(prompt)?)
        } else {
            Ok(std::io::stdin().lock().read_line()?.unwrap_or_default())
        }
    }

    let mut string: String = Default::default();
    while string.is_empty() {
        // Keep trying until the user provides an input
        string = get_possibly_empty_string(prompt)?;
    }
    Ok(string)
}

fn pretty_print_transaction_plan(
    fvk: Option<FullViewingKey>,
    plan: &TransactionPlan,
) -> anyhow::Result<()> {
    use penumbra_sdk_shielded_pool::{output, spend};

    fn dummy_sig<D: Domain>() -> Signature<D> {
        Signature::from([0u8; 64])
    }

    fn dummy_pk<D: Domain>() -> VerificationKey<D> {
        VerificationKey::try_from(Element::default().vartime_compress().0)
            .expect("creating a dummy verification key should work")
    }

    fn dummy_commitment() -> Commitment {
        Commitment(Element::default())
    }

    fn dummy_proof_spend() -> spend::SpendProof {
        spend::SpendProof::try_from(
            penumbra_sdk_proto::penumbra::core::component::shielded_pool::v1::ZkSpendProof {
                inner: vec![0u8; GROTH16_PROOF_LENGTH_BYTES],
            },
        )
        .expect("creating a dummy proof should work")
    }

    fn dummy_proof_output() -> output::OutputProof {
        output::OutputProof::try_from(
            penumbra_sdk_proto::penumbra::core::component::shielded_pool::v1::ZkOutputProof {
                inner: vec![0u8; GROTH16_PROOF_LENGTH_BYTES],
            },
        )
        .expect("creating a dummy proof should work")
    }

    fn dummy_spend() -> spend::Spend {
        spend::Spend {
            body: spend::Body {
                balance_commitment: dummy_commitment(),
                nullifier: Nullifier(Fq::default()),
                rk: dummy_pk(),
                encrypted_backref: EncryptedBackref::try_from([0u8; 0])
                    .expect("can create dummy encrypted backref"),
            },
            auth_sig: dummy_sig(),
            proof: dummy_proof_spend(),
        }
    }

    fn dummy_output() -> output::Output {
        output::Output {
            body: output::Body {
                note_payload: penumbra_sdk_shielded_pool::NotePayload {
                    note_commitment: penumbra_sdk_shielded_pool::note::StateCommitment(
                        Fq::default(),
                    ),
                    ephemeral_key: [0u8; 32]
                        .as_slice()
                        .try_into()
                        .expect("can create dummy ephemeral key"),
                    encrypted_note: penumbra_sdk_shielded_pool::NoteCiphertext([0u8; 176]),
                },
                balance_commitment: dummy_commitment(),
                ovk_wrapped_key: OvkWrappedKey([0u8; 48]),
                wrapped_memo_key: WrappedMemoKey([0u8; 48]),
            },
            proof: dummy_proof_output(),
        }
    }

    fn convert_note(cache: &Cache, fvk: &FullViewingKey, note: &Note) -> NoteView {
        NoteView {
            value: note.value().view_with_cache(cache),
            rseed: note.rseed(),
            address: fvk.view_address(note.address()),
        }
    }

    fn convert_action(
        cache: &Cache,
        fvk: &FullViewingKey,
        action: &ActionPlan,
    ) -> Option<ActionView> {
        use view::action_view::SpendView;

        match action {
            ActionPlan::Output(x) => Some(ActionView::Output(
                penumbra_sdk_shielded_pool::OutputView::Visible {
                    output: dummy_output(),
                    note: convert_note(cache, fvk, &x.output_note()),
                    payload_key: PayloadKey::from([0u8; 32]),
                },
            )),
            ActionPlan::Spend(x) => Some(ActionView::Spend(SpendView::Visible {
                spend: dummy_spend(),
                note: convert_note(cache, fvk, &x.note),
            })),
            ActionPlan::ValidatorDefinition(_) => None,
            ActionPlan::Swap(_) => None,
            ActionPlan::SwapClaim(_) => None,
            ActionPlan::ProposalSubmit(_) => None,
            ActionPlan::ProposalWithdraw(_) => None,
            ActionPlan::DelegatorVote(_) => None,
            ActionPlan::ValidatorVote(_) => None,
            ActionPlan::ProposalDepositClaim(_) => None,
            ActionPlan::PositionOpen(_) => None,
            ActionPlan::PositionClose(_) => None,
            ActionPlan::PositionWithdraw(_) => None,
            ActionPlan::Delegate(_) => None,
            ActionPlan::Undelegate(_) => None,
            ActionPlan::UndelegateClaim(_) => None,
            ActionPlan::Ics20Withdrawal(_) => None,
            ActionPlan::CommunityPoolSpend(_) => None,
            ActionPlan::CommunityPoolOutput(_) => None,
            ActionPlan::CommunityPoolDeposit(_) => None,
            ActionPlan::ActionDutchAuctionSchedule(_) => None,
            ActionPlan::ActionDutchAuctionEnd(_) => None,
            ActionPlan::ActionDutchAuctionWithdraw(_) => None,
            ActionPlan::IbcAction(_) => todo!(),
            ActionPlan::ActionLiquidityTournamentVote(_) => None,
        }
    }

    // Regardless of if we have the FVK, we can print the raw plan
    println!("{}", serde_json::to_string_pretty(plan)?);

    // The rest of the printing requires the FVK
    let fvk = match fvk {
        None => {
            return Ok(());
        }
        Some(x) => x,
    };

    let cache = Cache::with_known_assets();

    let view = TransactionView {
        anchor: penumbra_sdk_tct::Root(Hash::zero()),
        binding_sig: dummy_sig(),
        body_view: view::TransactionBodyView {
            action_views: plan
                .actions
                .iter()
                .filter_map(|x| convert_action(&cache, &fvk, x))
                .collect(),
            transaction_parameters: plan.transaction_parameters.clone(),
            detection_data: None,
            memo_view: None,
        },
    };

    view.render_terminal();

    Ok(())
}

/// For threshold custody, we need to implement this weird terminal abstraction.
///
/// This actually does stuff to stdin and stdout.
#[derive(Clone, Default)]
pub struct ActualTerminal {
    pub fvk: Option<FullViewingKey>,
}

#[async_trait]
impl Terminal for ActualTerminal {
    async fn confirm_request(&self, signing_request: &SigningRequest) -> Result<bool> {
        match signing_request {
            SigningRequest::TransactionPlan(plan) => {
                pretty_print_transaction_plan(self.fvk.clone(), plan)?;
                println!("Do you approve this transaction?");
            }
            SigningRequest::ValidatorDefinition(def) => {
                println!("{}", serde_json::to_string_pretty(def)?);
                println!("Do you approve this validator definition?");
            }
            SigningRequest::ValidatorVote(vote) => {
                println!("{}", serde_json::to_string_pretty(vote)?);
                println!("Do you approve this validator vote?");
            }
        };

        println!("Press enter to continue");
        self.read_line_raw().await?;
        Ok(true)
    }

    fn explain(&self, msg: &str) -> Result<()> {
        println!(
            "{}{}{}",
            color::Fg(color::Blue),
            msg,
            color::Fg(color::Reset)
        );
        Ok(())
    }

    async fn broadcast(&self, data: &str) -> Result<()> {
        println!(
            "\n{}{}{}\n",
            color::Fg(color::Yellow),
            data,
            color::Fg(color::Reset)
        );
        Ok(())
    }

    async fn read_line_raw(&self) -> Result<String> {
        // Use raw mode to allow reading more than 1KB/4KB of data at a time
        // See https://unix.stackexchange.com/questions/204815/terminal-does-not-accept-pasted-or-typed-lines-of-more-than-1024-characters
        use termion::raw::IntoRawMode;
        tracing::debug!("about to enter raw mode for long pasted input");

        print!("{}", color::Fg(color::Red));
        // In raw mode, the input is not mirrored into the terminal, so we need
        // to read char-by-char and echo it back.
        let mut stdout = std::io::stdout().into_raw_mode()?;

        let mut bytes = Vec::with_capacity(8192);
        for b in std::io::stdin().bytes() {
            let b = b?;
            // In raw mode, we need to handle control characters ourselves
            if b == 3 || b == 4 {
                // Ctrl-C or Ctrl-D
                return Err(anyhow::anyhow!("aborted"));
            }
            // In raw mode, the enter key might generate \r or \n, check either.
            if b == b'\n' || b == b'\r' {
                break;
            }
            // Store the byte we read and print it back to the terminal.
            bytes.push(b);
            stdout.write_all(&[b]).expect("stdout write failed");
            // Flushing may not be the most efficient but performance isn't critical here.
            stdout.flush()?;
        }
        // Drop _stdout to restore the terminal to normal mode
        std::mem::drop(stdout);
        // We consumed a newline of some kind but didn't echo it, now print
        // one out so subsequent output is guaranteed to be on a new line.
        println!("");
        print!("{}", color::Fg(color::Reset));

        tracing::debug!("exited raw mode and returned to cooked mode");

        let line = String::from_utf8(bytes)?;
        tracing::debug!(?line, "read response line");

        Ok(line)
    }

    async fn get_password(&self) -> Result<String> {
        read_password("Enter Password: ").await
    }
}

impl ActualTerminal {
    pub async fn get_confirmed_password() -> Result<String> {
        loop {
            let password = read_password("Enter Password: ").await?;
            let confirmed = read_password("Confirm Password: ").await?;
            if password != confirmed {
                println!("Password mismatch, please try again.");
                continue;
            }
            return Ok(password);
        }
    }
}
