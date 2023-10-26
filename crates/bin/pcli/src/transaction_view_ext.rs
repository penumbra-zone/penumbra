use comfy_table::presets;
use comfy_table::Table;
use penumbra_asset::asset::Cache;
use penumbra_dex::swap::SwapView;
use penumbra_keys::keys::IncomingViewingKey;
use penumbra_keys::Address;
use penumbra_keys::AddressView;
use penumbra_keys::FullViewingKey;
use penumbra_shielded_pool::NoteView;
use penumbra_shielded_pool::SpendView;
use penumbra_transaction::view::action_view::OutputView;
use penumbra_transaction::TransactionView;

// a helper function to create pretty placeholders for encrypted information
fn format_opaque_bytes(bytes: &[u8]) -> String {
    if bytes.len() < 8 {
        return String::new();
    } else {
        // Select foreground and background colors based on the first 8 bytes.
        // let fg_color_index = bytes[0] % 8;
        // let bg_color_index = bytes[4] % 8;

        // ANSI escape codes for foreground and background colors.
        let fg_color_code = 37; // 30 through 37 are foreground colors
        let bg_color_code = 40; // 40 through 47 are background colors

        // TODO: Hm, this can allow the same color for both, should rejig things to avoid this

        let max_bytes = 32;
        let rem = if bytes.len() > max_bytes {
            bytes[0..max_bytes].to_vec()
        } else {
            bytes.to_vec()
        };

        // Convert the rest of the bytes to hexadecimal.
        let hex_str = hex::encode_upper(rem);
        let block_chars: String = hex_str
            .chars()
            .map(|c| {
                match c {
                    '0' => "\u{2595}", // TODO: what are these codes
                    '1' => "\u{2581}",
                    '2' => "\u{2582}",
                    '3' => "\u{2583}",
                    '4' => "\u{2584}",
                    '5' => "\u{2585}",
                    '6' => "\u{2586}",
                    '7' => "\u{2587}",
                    '8' => "\u{2588}",
                    '9' => "\u{2589}",
                    'A' => "\u{259A}",
                    'B' => "\u{259B}",
                    'C' => "\u{259C}",
                    'D' => "\u{258D}",
                    'E' => "\u{259E}",
                    'F' => "\u{259F}",
                    _ => "",
                }
                .to_string()
            })
            .collect();

        format!(
            "\u{001b}[{};{}m{}",
            fg_color_code, bg_color_code, block_chars
        )
    }
}

fn format_visible_address(
    asset_cache: &Cache,
    ivk: &IncomingViewingKey,
    decrypted_note: &NoteView,
) -> String {
    format!(
        "{} spent {}",
        format_address(ivk, &decrypted_note.address.address()),
        decrypted_note.value.value().format(asset_cache),
    )
}

// Turns an `Address` into a `String` representation; either a short-form for addresses
// not associated with the `ivk`, or in the form of `[account: {account}]` for
// addresses associated with the `ivk`.
fn format_address(ivk: &IncomingViewingKey, address: &Address) -> String {
    if ivk.views_address(address) {
        let account = ivk.index_for_diversifier(address.diversifier()).account;

        format!("[account {account:?}]")
    } else {
        address.display_short_form()
    }
}

pub trait TransactionViewExt {
    /// Render this transaction view on stdout.
    fn render_terminal(&self, fvk: FullViewingKey, asset_cache: Cache);
}

impl TransactionViewExt for TransactionView {
    // TODO: this can probably be generic over different key types, decoding only what the key can see.
    fn render_terminal(&self, fvk: FullViewingKey, asset_cache: Cache) {
        // tx id
        // anchor hash
        // tx_sig?
        // fee
        // detection data?
        // memo view

        println!("⠿ Tx Metadata");
        println!("⠿ Anchor");
        println!(
            "⠿ Fee: {}",
            &self.body_view.transaction_parameters.fee.value().amount
        );
        println!(
            "⠿ Expiration Height: {}",
            &self.body_view.transaction_parameters.expiry_height
        );

        if let Some(memo_view) = &self.body_view.memo_view {
            match memo_view {
                penumbra_transaction::MemoView::Visible {
                    plaintext,
                    ciphertext: _,
                } => {
                    println!("⠿ Memo Sender: {}", &plaintext.return_address.address());
                    println!("⠿ Memo Text: {}\n", &plaintext.text);
                }
                penumbra_transaction::MemoView::Opaque { ciphertext: _ } => (),
            }
        }

        let mut actions_table = Table::new();
        actions_table.load_preset(presets::NOTHING);
        actions_table.set_header(vec!["Tx Action", "Description"]);

        // Iterate over the ActionViews in the TxV & display as appropriate
        for action_view in &self.body_view.action_views {
            let opaque_action: String;
            let visible_action: String;

            let row = match action_view {
                penumbra_transaction::ActionView::Swap(swap) => match swap {
                    SwapView::Visible {
                        swap: _,
                        swap_plaintext: _,
                    } => ["visible", "swap"],
                    SwapView::Opaque { swap: _ } => ["opaque", "swap"],
                },
                penumbra_transaction::ActionView::SwapClaim(_av) => ["okie doke", "swap plaintext"],
                penumbra_transaction::ActionView::Output(output) => {
                    match output {
                        OutputView::Visible {
                            output: _,
                            note,
                            payload_key: _,
                        } => {
                            visible_action = format!(
                                "{} -> {}",
                                note.value.value().format(&asset_cache),
                                format_address(fvk.incoming(), &note.address.address())
                            );
                            ["Output", &visible_action]
                        }
                        OutputView::Opaque { output } => {
                            let bytes = output.body.note_payload.encrypted_note.0.clone();
                            opaque_action = format_opaque_bytes(&bytes) + "\x1b[0m"; // ANSI code to reset colors
                            ["Output", &opaque_action]
                        }
                    }
                }
                penumbra_transaction::ActionView::Spend(spend) => {
                    match spend {
                        SpendView::Visible { spend: _, note } => {
                            visible_action = format!(
                                "{} -> {}",
                                format_address(fvk.incoming(), &note.address.address()),
                                note.value.value().format(&asset_cache)
                            );
                            ["Spend", &visible_action]
                        }
                        SpendView::Opaque { spend } => {
                            let bytes = spend.body.nullifier.to_bytes();
                            opaque_action = format_opaque_bytes(&bytes) + "\x1b[0m"; // ANSI code to reset colors
                            ["Spend", &opaque_action]
                        }
                    }
                }

                _ => ["NYI", "NYI"], // this match should be exhaustive
            };

            actions_table.add_row(row);
        }

        // Print table of actions and their descriptions
        println!("{actions_table}");
    }
}

/*

penumbra_transaction::ActionView::Spend(av) => ["temp", "test"],
penumbra_transaction::ActionView::Delegate(av) => ["temp", "test"],
penumbra_transaction::ActionView::Undelegate(av) => ["temp", "test"],
penumbra_transaction::ActionView::UndelegateClaim(av) => ["temp", "test"],
penumbra_transaction::ActionView::IbcAction(av) => ["temp", "test"],
penumbra_transaction::ActionView::ProposalSubmit(av) => ["temp", "test"],
penumbra_transaction::ActionView::ProposalWithdraw(av) => ["temp", "test"],
penumbra_transaction::ActionView::ProposalDepositClaim(av) => ["temp", "test"],
penumbra_transaction::ActionView::ValidatorVote(av) => ["temp", "test"],
penumbra_transaction::ActionView::DelegatorVote(av) => ["temp", "test"],
penumbra_transaction::ActionView::PositionOpen(av) => ["temp", "test"],
penumbra_transaction::ActionView::PositionClose(av) => ["temp", "test"],
penumbra_transaction::ActionView::PositionWithdraw(av) => ["temp", "test"],
penumbra_transaction::ActionView::PositionRewardClaim(av) => ["temp", "test"],
penumbra_transaction::ActionView::Ics20Withdrawal(i) => ["temp", "test"],
penumbra_transaction::ActionView::DaoDeposit(av) => ["temp", "test"],
penumbra_transaction::ActionView::DaoSpend(av) => ["temp", "test"],
penumbra_transaction::ActionView::DaoOutput(av) => ["temp", "test"],

*/
