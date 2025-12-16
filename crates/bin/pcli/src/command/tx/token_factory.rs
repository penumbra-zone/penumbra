//! Token factory transaction commands.
//!
//! This module provides the CLI interface for creating and minting tokens.
//!
//! Two token types are supported:
//! - **Fairlaunch**: Fixed supply, no minting capability (`--enable-mint false`)
//! - **Mintable**: Outputs a MintCapability for future minting (`--enable-mint true`)
//!
//! The mint capability is a unique asset that must be spent to mint additional tokens.
//! This provides cryptographic proof of minting authority.

use anyhow::{Context, Result};
use clap::Parser;
use penumbra_sdk_asset::asset;
use penumbra_sdk_fee::FeeTier;
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_num::Amount;
use penumbra_sdk_token_factory::{ActionTokenFactoryCreate, ActionTokenFactoryMint, TokenFactoryId};
use penumbra_sdk_wallet::plan::Planner;
use rand_core::OsRng;

use crate::App;

/// Token factory commands for creating and minting tokens.
#[derive(Debug, Parser)]
pub enum TokenFactoryCmd {
    /// Create a new token.
    ///
    /// By default creates a fairlaunch token with fixed supply.
    /// Use --enable-mint to also output a MintCapability for future minting.
    #[clap(display_order = 100)]
    Create {
        /// The name/symbol for the token (e.g., "MYTOKEN").
        #[clap(long)]
        name: String,
        /// The initial supply to create, written as a number (e.g., "1000000").
        /// Can be 0 for bridge tokens that will be minted on demand.
        #[clap(long)]
        supply: String,
        /// If true, outputs a MintCapability allowing future minting.
        /// If false (default), creates a fairlaunch token with fixed supply.
        #[clap(long, default_value = "false")]
        enable_mint: bool,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Mint additional tokens using a mint capability.
    ///
    /// This consumes MintCapability(seq=N) from your balance and produces
    /// MintCapability(seq=N+1) plus the minted tokens.
    #[clap(display_order = 110)]
    Mint {
        /// The token factory ID (hex-encoded 32 bytes).
        #[clap(long)]
        token_id: String,
        /// The current sequence number of your mint capability.
        #[clap(long)]
        seq: u64,
        /// Amount to mint.
        #[clap(long)]
        amount: String,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
}

impl TokenFactoryCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let gas_prices = app
            .view
            .as_mut()
            .context("view service must be initialized")?
            .gas_prices(penumbra_sdk_proto::view::v1::GasPricesRequest {})
            .await?
            .into_inner()
            .gas_prices
            .expect("gas prices must be available")
            .try_into()?;

        match self {
            TokenFactoryCmd::Create {
                name,
                supply,
                enable_mint,
                source,
                fee_tier,
            } => {
                // Generate random nonce for token ID
                let mut nonce_bytes = [0u8; 32];
                rand::RngCore::fill_bytes(&mut OsRng, &mut nonce_bytes);
                let token_id = TokenFactoryId::new(nonce_bytes);

                // Parse supply
                let supply: Amount = Amount::from(
                    supply
                        .parse::<u128>()
                        .context("invalid supply amount")?
                );

                // Create metadata for the token
                let metadata = asset::Metadata::try_from(
                    penumbra_sdk_proto::core::asset::v1::Metadata {
                        base: token_id.denom(),
                        display: name.clone(),
                        name: name.clone(),
                        symbol: name.clone(),
                        ..Default::default()
                    }
                )?;

                let action = ActionTokenFactoryCreate::new(token_id.clone(), metadata, supply, *enable_mint)
                    .context("invalid create parameters")?;

                if *enable_mint {
                    println!("Creating mintable token:");
                    println!("  ID: {}", hex::encode(token_id.as_bytes()));
                    println!("  Denom: {}", token_id.denom());
                    println!("  Initial supply: {}", supply);
                    println!("  MintCapability(seq=0) will be output");
                } else {
                    println!("Creating fairlaunch token:");
                    println!("  ID: {}", hex::encode(token_id.as_bytes()));
                    println!("  Denom: {}", token_id.denom());
                    println!("  Fixed supply: {}", supply);
                }

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .token_factory_create(action);
                let plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;

                println!("Token created successfully.");
                Ok(())
            }

            TokenFactoryCmd::Mint {
                token_id,
                seq,
                amount,
                source,
                fee_tier,
            } => {
                // Parse token ID from hex
                let token_id_bytes: [u8; 32] = hex::decode(token_id)
                    .context("invalid hex for token_id")?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("token_id must be exactly 32 bytes"))?;
                let token_id = TokenFactoryId::new(token_id_bytes);

                // Parse amount
                let amount: Amount = Amount::from(
                    amount
                        .parse::<u128>()
                        .context("invalid mint amount")?
                );

                let action = ActionTokenFactoryMint::new(token_id.clone(), *seq, amount)
                    .context("invalid mint parameters")?;

                println!("Minting tokens:");
                println!("  Token ID: {}", hex::encode(token_id.as_bytes()));
                println!("  Current seq: {}", seq);
                println!("  Amount: {}", amount);
                println!("  Will consume MintCapability(seq={}) and produce MintCapability(seq={})", seq, seq + 1);

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .token_factory_mint(action);
                let plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;

                println!("Tokens minted successfully.");
                Ok(())
            }
        }
    }
}
