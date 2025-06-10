use comfy_table::presets;
use comfy_table::Table;
use penumbra_sdk_asset::asset::Id;
use penumbra_sdk_asset::asset::Metadata;
use penumbra_sdk_asset::Value;
use penumbra_sdk_asset::ValueView;
use penumbra_sdk_dex::swap::SwapView;
use penumbra_sdk_dex::swap_claim::SwapClaimView;
use penumbra_sdk_dex::PositionOpen;
use penumbra_sdk_fee::Fee;
use penumbra_sdk_keys::AddressView;
use penumbra_sdk_num::Amount;
use penumbra_sdk_shielded_pool::SpendView;
use penumbra_sdk_transaction::view::action_view::OutputView;
use penumbra_sdk_transaction::TransactionView;

// Issues identified:
// TODO: FeeView
// TODO: TradingPairView
// Implemented some helper functions which may make more sense as methods on existing Structs

// helper function to create a value view from a value and optional metadata
fn create_value_view(value: Value, metadata: Option<Metadata>) -> ValueView {
    match metadata {
        Some(metadata) => ValueView::KnownAssetId {
            amount: value.amount,
            metadata,
            equivalent_values: Vec::new(),
            extended_metadata: None,
        },
        None => ValueView::UnknownAssetId {
            amount: value.amount,
            asset_id: value.asset_id,
        },
    }
}

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
                    '0' => "\u{2595}",
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
                    'D' => "\u{259D}",
                    'E' => "\u{259E}",
                    'F' => "\u{259F}",
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
// propose moving this to core/keys/src/address/view.rs
fn format_address_view(address_view: &AddressView) -> String {
    match address_view {
        AddressView::Decoded {
            address: _,
            index,
            wallet_id: _,
        } => {
            if !index.is_ephemeral() {
                format!("[account {:?}]", index.account)
            } else {
                format!("[account {:?} (one-time address)]", index.account)
            }
        }
        AddressView::Opaque { address } => {
            // The address being opaque just means we can't see the internal structure,
            // we should render the content so it can be copy-pasted.
            format!("{}", address)
        }
    }
}

// feels like these functions should be extension traits of their respective structs
// propose moving this to core/asset/src/value.rs
fn format_value_view(value_view: &ValueView) -> String {
    match value_view {
        ValueView::KnownAssetId {
            amount,
            metadata: denom,
            ..
        } => {
            let unit = denom.default_unit();
            format!("{}{}", unit.format_value(*amount), unit)
        }
        ValueView::UnknownAssetId { amount, asset_id } => {
            format!("{}{}", amount, asset_id)
        }
    }
}

fn format_amount_range(
    start: Amount,
    stop: Amount,
    asset_id: &Id,
    metadata: Option<&Metadata>,
) -> String {
    match metadata {
        Some(denom) => {
            let unit = denom.default_unit();
            format!(
                "({}..{}){}",
                unit.format_value(start),
                unit.format_value(stop),
                unit
            )
        }
        None => format!("({}..{}){}", start, stop, asset_id),
    }
}

fn format_fee(fee: &Fee) -> String {
    // TODO: Implement FeeView to show decrypted fee.
    format!("{}", fee.amount())
}

fn format_asset_id(asset_id: &Id) -> String {
    // TODO: Implement TradingPairView to show decrypted .asset_id()
    let input = &asset_id.to_string();
    let truncated = &input[0..10]; //passet1
    let ellipsis = "...";
    let end = &input[(input.len() - 3)..];
    format!("{}{}{}", truncated, ellipsis, end)
}

// When handling ValueViews inside of a Visible variant of an ActionView, handling both cases might be needlessly verbose
// potentially this makes sense as a method on the ValueView enum
// propose moving this to core/asset/src/value.rs
fn value_view_amount(value_view: &ValueView) -> Amount {
    match value_view {
        ValueView::KnownAssetId { amount, .. } | ValueView::UnknownAssetId { amount, .. } => {
            *amount
        }
    }
}

pub trait TransactionViewExt {
    /// Render this transaction view on stdout.
    fn render_terminal(&self);
}

impl TransactionViewExt for TransactionView {
    fn render_terminal(&self) {
        let fee = &self.body_view.transaction_parameters.fee;
        // the denomination should be visible here... does a FeeView exist?
        println!("Fee: {}", format_fee(&fee));

        println!(
            "Expiration Height: {}",
            &self.body_view.transaction_parameters.expiry_height
        );

        if let Some(memo_view) = &self.body_view.memo_view {
            match memo_view {
                penumbra_sdk_transaction::MemoView::Visible {
                    plaintext,
                    ciphertext: _,
                } => {
                    println!("Memo Sender: {}", &plaintext.return_address.address());
                    println!("Memo Text: \n{}\n", &plaintext.text);
                }
                penumbra_sdk_transaction::MemoView::Opaque { ciphertext } => {
                    println!("Encrypted Memo: \n{}\n", format_opaque_bytes(&ciphertext.0));
                }
            }
        }

        let mut actions_table = Table::new();
        actions_table.load_preset(presets::NOTHING);
        actions_table.set_header(vec!["Tx Action", "Description"]);

        // Iterate over the ActionViews in the TxView & display as appropriate
        for action_view in &self.body_view.action_views {
            let action: String;

            let row = match action_view {
                penumbra_sdk_transaction::ActionView::Spend(spend) => {
                    match spend {
                        SpendView::Visible { spend: _, note } => {
                            action = format!(
                                "{} -> {}",
                                format_address_view(&note.address),
                                format_value_view(&note.value)
                            );
                            ["Spend", &action]
                        }
                        SpendView::Opaque { spend } => {
                            let bytes = spend.body.nullifier.to_bytes(); // taken to be a unique value, for aesthetic reasons
                            action = format_opaque_bytes(&bytes);
                            ["Spend", &action]
                        }
                    }
                }
                penumbra_sdk_transaction::ActionView::Output(output) => {
                    match output {
                        OutputView::Visible {
                            output: _,
                            note,
                            payload_key: _,
                        } => {
                            action = format!(
                                "{} -> {}",
                                format_value_view(&note.value),
                                format_address_view(&note.address),
                            );
                            ["Output", &action]
                        }
                        OutputView::Opaque { output } => {
                            let bytes = output.body.note_payload.encrypted_note.0; // taken to be a unique value, for aesthetic reasons
                            action = format_opaque_bytes(&bytes);
                            ["Output", &action]
                        }
                    }
                }
                penumbra_sdk_transaction::ActionView::Swap(swap) => {
                    // Typical swaps are one asset for another, but we can't know that for sure.
                    match swap {
                        SwapView::Visible { swap_plaintext, .. } => {
                            let (from_asset, from_value, to_asset) = match (
                                swap_plaintext.delta_1_i.value(),
                                swap_plaintext.delta_2_i.value(),
                            ) {
                                (0, v) if v > 0 => (
                                    swap_plaintext.trading_pair.asset_2(),
                                    swap_plaintext.delta_2_i,
                                    swap_plaintext.trading_pair.asset_1(),
                                ),
                                (v, 0) if v > 0 => (
                                    swap_plaintext.trading_pair.asset_1(),
                                    swap_plaintext.delta_1_i,
                                    swap_plaintext.trading_pair.asset_2(),
                                ),
                                // The pathological case (both assets have output values).
                                _ => (
                                    swap_plaintext.trading_pair.asset_1(),
                                    swap_plaintext.delta_1_i,
                                    swap_plaintext.trading_pair.asset_1(),
                                ),
                            };

                            action = format!(
                                "{} {} for {} and paid claim fee {}",
                                from_value,
                                format_asset_id(&from_asset),
                                format_asset_id(&to_asset),
                                format_fee(&swap_plaintext.claim_fee),
                            );

                            ["Swap", &action]
                        }
                        SwapView::Opaque { swap, .. } => {
                            action = format!(
                                "Opaque swap for trading pair: {} <=> {}",
                                format_asset_id(&swap.body.trading_pair.asset_1()),
                                format_asset_id(&swap.body.trading_pair.asset_2()),
                            );
                            ["Swap", &action]
                        }
                    }
                }
                penumbra_sdk_transaction::ActionView::SwapClaim(swap_claim) => {
                    match swap_claim {
                        SwapClaimView::Visible {
                            swap_claim,
                            output_1,
                            output_2,
                            swap_tx: _,
                        } => {
                            // View service can't see SwapClaims: https://github.com/penumbra-zone/penumbra/issues/2547
                            dbg!(swap_claim);
                            let claimed_value = match (
                                value_view_amount(&output_1.value).value(),
                                value_view_amount(&output_2.value).value(),
                            ) {
                                (0, v) if v > 0 => format_value_view(&output_2.value),
                                (v, 0) if v > 0 => format_value_view(&output_1.value),
                                // The pathological case (both assets have output values).
                                _ => format!(
                                    "{} and {}",
                                    format_value_view(&output_1.value),
                                    format_value_view(&output_2.value),
                                ),
                            };

                            action = format!(
                                "Claimed {} with fee {:?}",
                                claimed_value,
                                format_fee(&swap_claim.body.fee),
                            );
                            ["Swap Claim", &action]
                        }
                        SwapClaimView::Opaque { swap_claim } => {
                            let bytes = swap_claim.body.nullifier.to_bytes(); // taken to be a unique value, for aesthetic reasons
                            action = format_opaque_bytes(&bytes);
                            ["Swap Claim", &action]
                        }
                    }
                }
                penumbra_sdk_transaction::ActionView::Ics20Withdrawal(withdrawal) => {
                    let unit = withdrawal.denom.best_unit_for(withdrawal.amount);
                    action = format!(
                        "{}{} via {} to {}",
                        unit.format_value(withdrawal.amount),
                        unit,
                        withdrawal.source_channel,
                        withdrawal.destination_chain_address,
                    );
                    ["Ics20 Withdrawal", &action]
                }
                penumbra_sdk_transaction::ActionView::PositionOpen(position_open) => {
                    let position = PositionOpen::from(position_open.clone()).position;
                    /* TODO: leaving this around since we may want it to render prices
                    let _unit_pair = DirectedUnitPair {
                        start: unit_1.clone(),
                        end: unit_2.clone(),
                    };
                    */

                    action = format!(
                        "Reserves: ({} {}, {} {}) Fee: {} ID: {}",
                        position.reserves.r1,
                        format_asset_id(&position.phi.pair.asset_1()),
                        position.reserves.r2,
                        format_asset_id(&position.phi.pair.asset_2()),
                        position.phi.component.fee,
                        position.id(),
                    );
                    ["Open Liquidity Position", &action]
                }
                penumbra_sdk_transaction::ActionView::PositionClose(_) => {
                    ["Close Liquitity Position", ""]
                }
                penumbra_sdk_transaction::ActionView::PositionWithdraw(_) => {
                    ["Withdraw Liquitity Position", ""]
                }
                penumbra_sdk_transaction::ActionView::ProposalDepositClaim(
                    proposal_deposit_claim,
                ) => {
                    action = format!(
                        "Claim Deposit for Governance Proposal #{}",
                        proposal_deposit_claim.proposal
                    );
                    [&action, ""]
                }
                penumbra_sdk_transaction::ActionView::ProposalSubmit(proposal_submit) => {
                    action = format!(
                        "Submit Governance Proposal #{}",
                        proposal_submit.proposal.id
                    );
                    [&action, ""]
                }
                penumbra_sdk_transaction::ActionView::ProposalWithdraw(proposal_withdraw) => {
                    action = format!(
                        "Withdraw Governance Proposal #{}",
                        proposal_withdraw.proposal
                    );
                    [&action, ""]
                }
                penumbra_sdk_transaction::ActionView::IbcRelay(_) => ["IBC Relay", ""],
                penumbra_sdk_transaction::ActionView::DelegatorVote(_) => ["Delegator Vote", ""],
                penumbra_sdk_transaction::ActionView::ValidatorDefinition(_) => {
                    ["Upload Validator Definition", ""]
                }
                penumbra_sdk_transaction::ActionView::ValidatorVote(_) => ["Validator Vote", ""],
                penumbra_sdk_transaction::ActionView::CommunityPoolDeposit(_) => {
                    ["Community Pool Deposit", ""]
                }
                penumbra_sdk_transaction::ActionView::CommunityPoolSpend(_) => {
                    ["Community Pool Spend", ""]
                }
                penumbra_sdk_transaction::ActionView::CommunityPoolOutput(_) => {
                    ["Community Pool Output", ""]
                }
                penumbra_sdk_transaction::ActionView::Delegate(_) => ["Delegation", ""],
                penumbra_sdk_transaction::ActionView::Undelegate(_) => ["Undelegation", ""],
                penumbra_sdk_transaction::ActionView::UndelegateClaim(_) => {
                    ["Undelegation Claim", ""]
                }
                penumbra_sdk_transaction::ActionView::ActionDutchAuctionSchedule(x) => {
                    let description = &x.action.description;

                    let input: String = format_value_view(&create_value_view(
                        description.input,
                        x.input_metadata.clone(),
                    ));
                    let output: String = format_amount_range(
                        description.min_output,
                        description.max_output,
                        &description.output_id,
                        x.output_metadata.as_ref(),
                    );
                    let start = description.start_height;
                    let stop = description.end_height;
                    let steps = description.step_count;
                    let auction_id = x.auction_id;
                    action = format!(
                        "{} -> {}, blocks {}..{}, in {} steps ({})",
                        input, output, start, stop, steps, auction_id
                    );
                    ["Dutch Auction Schedule", &action]
                }
                penumbra_sdk_transaction::ActionView::ActionDutchAuctionEnd(x) => {
                    action = format!("{}", x.auction_id);
                    ["Dutch Auction End", &action]
                }
                penumbra_sdk_transaction::ActionView::ActionDutchAuctionWithdraw(x) => {
                    let inside = x
                        .reserves
                        .iter()
                        .map(|value| format_value_view(value))
                        .collect::<Vec<_>>()
                        .as_slice()
                        .join(", ");
                    action = format!("{} -> [{}]", x.action.auction_id, inside);
                    ["Dutch Auction Withdraw", &action]
                }
                penumbra_sdk_transaction::ActionView::ActionLiquidityTournamentVote(_) => todo!(),
            };

            actions_table.add_row(row);
        }

        // Print table of actions and their descriptions
        println!("{actions_table}");
    }
}
