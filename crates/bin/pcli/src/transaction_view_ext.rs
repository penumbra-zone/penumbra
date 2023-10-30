use comfy_table::presets;
use comfy_table::Table;
use penumbra_asset::ValueView;
use penumbra_keys::AddressView;
use penumbra_shielded_pool::SpendView;
use penumbra_transaction::view::action_view::OutputView;
use penumbra_transaction::TransactionView;

// a helper function to create pretty placeholders for encrypted information
fn format_opaque_bytes(bytes: &[u8]) -> String {
    if bytes.len() < 8 {
        return String::new();
    } else {
        /*
        // TODO: Hm, this can allow the same color for both, should rejig things to avoid this
        // Select foreground and background colors based on the first 8 bytes.
        let fg_color_index = bytes[0] % 8;
        let bg_color_index = bytes[4] % 8;

        // ANSI escape codes for foreground and background colors.
        let fg_color_code = 37; // 30 through 37 are foreground colors
        let bg_color_code = 40; // 40 through 47 are background colors
        */

        // to be more general, perhaps this should be configurable
        // an opaque address needs less space than an opaque memo, etc
        let max_bytes = 32;
        let rem = if bytes.len() > max_bytes {
            bytes[0..max_bytes].to_vec()
        } else {
            bytes.to_vec()
        };

        // Convert the rest of the bytes to hexadecimal.
        let hex_str = hex::encode_upper(rem);
        let opaque_chars: String = hex_str
            .chars()
            .map(|c| {
                match c {
                    '0' => "⠚", // "\u{2595}", alternatively use ASCII blocks
                    '1' => "⠁", // "\u{2581}",
                    '2' => "⠃", // "\u{2582}",
                    '3' => "⠉", // "\u{2583}",
                    '4' => "⠙", // "\u{2584}",
                    '5' => "⠑", // "\u{2585}",
                    '6' => "⠋", // "\u{2586}",
                    '7' => "⠛", // "\u{2587}",
                    '8' => "⠓", // "\u{2588}",
                    '9' => "⠊", // "\u{2589}",
                    'A' => "⠊", // "\u{259A}",
                    'B' => "⠋", // "\u{259B}",
                    'C' => "⠌", // "\u{259C}",
                    'D' => "⠍", // "\u{259D}",
                    'E' => "⠎", // "\u{259E}",
                    'F' => "⠏", // "\u{259F}",
                    _ => "",
                }
                .to_string()
            })
            .collect();

        //format!("\u{001b}[{};{}m{}", fg_color_code, bg_color_code, block_chars)
        format!("{}", opaque_chars)
    }
}

// feels like these functions should be extension traits of their respective structs
fn format_address_view(address_view: &AddressView) -> String {
    match address_view {
        AddressView::Visible {
            address,
            index,
            wallet_id,
        } => {
            format!("[address {:?}]", index.account)
        }
        AddressView::Opaque { address } => {
            // slicing off the first 8 chars to match the plaintext length for aesthetics
            format!("{}", format_opaque_bytes(&address.to_vec()[..8]))
        }
    }
}

// feels like these functions should be extension traits of their respective structs
fn format_value_view(value_view: &ValueView) -> String {
    match value_view {
        ValueView::KnownDenom { amount, denom } => {
            // TODO: This can be further tweaked depending on what DenomMetadata units should be shown. Leaving as default for now.
            format!("{} {}", amount, denom)
            // format!("{}", format_opaque_bytes(&address.to_vec()[..8])) // slicing off the first 8 chars to match the plaintext length for aesthetics
        }
        ValueView::UnknownDenom { amount, asset_id } => {
            format!("{} {}", amount, asset_id) //format_opaque_bytes(&address.to_vec()))
        }
    }
}

pub trait TransactionViewExt {
    /// Render this transaction view on stdout.
    fn render_terminal(&self);
}

impl TransactionViewExt for TransactionView {
    fn render_terminal(&self) {
        println!("⠿ Tx Hash"); // Not available here?
        println!("⠿ Tx Sig"); // Probably not needed
        println!("⠿ Anchor"); // Probably not needed
        let fee = &self.body_view.transaction_parameters.fee;
        // the denomination should be visible here... does a FeeView exist?
        println!("⠿ Fee: {} {}", &fee.amount(), &fee.asset_id());
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
                    println!("⠿ Memo Text: \n{}\n", &plaintext.text);
                }
                penumbra_transaction::MemoView::Opaque { ciphertext } => {
                    println!(
                        "⠿ Encrypted Memo: \n{}\n",
                        format_opaque_bytes(&ciphertext.0)
                    );
                }
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
                penumbra_transaction::ActionView::Output(output) => {
                    match output {
                        OutputView::Visible {
                            output: _,
                            note,
                            payload_key: _,
                        } => {
                            visible_action = format!(
                                "{} -> {}",
                                format_value_view(&note.value),
                                format_address_view(&note.address),
                            );
                            ["Output", &visible_action]
                        }
                        OutputView::Opaque { output } => {
                            let bytes = output.body.note_payload.encrypted_note.0.clone(); // taken to be a unique value, for aesthetic reasons
                            opaque_action = format_opaque_bytes(&bytes);
                            ["Output", &opaque_action]
                        }
                    }
                }
                penumbra_transaction::ActionView::Spend(spend) => {
                    match spend {
                        SpendView::Visible { spend: _, note } => {
                            visible_action = format!(
                                "{} -> {}",
                                format_address_view(&note.address),
                                format_value_view(&note.value)
                            );
                            ["Spend", &visible_action]
                        }
                        SpendView::Opaque { spend } => {
                            let bytes = spend.body.nullifier.to_bytes(); // taken to be a unique value, for aesthetic reasons
                            opaque_action = format_opaque_bytes(&bytes);
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
                penumbra_transaction::ActionView::Swap(swap) => {
                    match swap {
                        SwapView::Visible {
                            swap: _,
                            swap_plaintext: _,
                        } => ["visible", "swap"],
                        SwapView::Opaque { swap: _ } => ["opaque", "swap"],
                    }
                }
                penumbra_transaction::ActionView::SwapClaim(_av) => ["okie doke", "swap plaintext"],
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
