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
use dialoguer::Confirm;
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_dex::{
    lp::{
        position::{self, Position},
        LpNft, Reserves,
    },
    DirectedTradingPair,
};
use penumbra_sdk_fee::FeeTier;
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_num::Amount;
use penumbra_sdk_shielded_pool::ActionBurnPlan;
use penumbra_sdk_token_factory::{
    ActionTokenFactoryCreate, ActionTokenFactoryMint, MintCapability, TokenFactoryId,
};
use penumbra_sdk_view::{Planner, ViewClient};
use rand_core::{CryptoRngCore, OsRng};

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
        /// If set, outputs a MintCapability allowing future minting.
        /// If omitted (default), creates a fairlaunch token with fixed supply.
        #[clap(long)]
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
    /// Fair launch a new token with a bonding curve.
    ///
    /// Creates a new token and immediately commits the entire supply to a
    /// bonding curve. The mint capability and all LP NFTs are burned, making
    /// the liquidity immutable. No frontrunning is possible because all DEX
    /// operations are batched per block.
    ///
    /// The bonding curve is approximated by a series of constant-price LP
    /// positions at ascending prices.
    #[clap(display_order = 120)]
    Launch {
        /// The name/symbol for the token (e.g., "MYTOKEN").
        #[clap(long)]
        name: String,
        /// The total supply to create for the bonding curve.
        #[clap(long)]
        supply: String,
        /// The quote asset to pair against (e.g., "penumbra" or "usdc").
        /// This is the asset used to purchase the new token.
        #[clap(long)]
        quote_asset: String,
        /// The starting price (quote asset per token) at the bottom of the curve.
        #[clap(long)]
        start_price: f64,
        /// The ending price (quote asset per token) at the top of the curve.
        #[clap(long)]
        end_price: f64,
        /// The type of bonding curve.
        #[clap(long, value_enum, default_value = "linear")]
        curve: CurveType,
        /// Number of LP positions to create along the curve.
        #[clap(long, default_value = "32")]
        num_positions: u32,
        /// Fee in basis points for each position (0-5000).
        #[clap(long, default_value = "0")]
        fee_bps: u32,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
        /// Skip confirmation prompt.
        #[clap(short, long)]
        yes: bool,
    },
}

/// The type of bonding curve to use for fair launches.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CurveType {
    /// Linear price increase from start to end.
    Linear,
    /// Exponential price increase (steeper at higher prices).
    Exponential,
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
                // `display` must reference one of the denom units; the only unit
                // is the base (`factory/{hex}`), so display == base. The friendly
                // name is carried by `name`/`symbol`.
                let metadata = asset::Metadata::try_from(
                    penumbra_sdk_proto::core::asset::v1::Metadata {
                        base: token_id.denom(),
                        display: token_id.denom(),
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

            TokenFactoryCmd::Launch {
                name,
                supply,
                quote_asset,
                start_price,
                end_price,
                curve,
                num_positions,
                fee_bps,
                source,
                fee_tier,
                yes,
            } => {
                // validate inputs
                if *start_price <= 0.0 || *end_price <= 0.0 {
                    anyhow::bail!("prices must be positive");
                }
                if *start_price >= *end_price {
                    anyhow::bail!("start_price must be less than end_price");
                }
                if *num_positions < 2 {
                    anyhow::bail!("need at least 2 positions for a bonding curve");
                }
                if *fee_bps > 5000 {
                    anyhow::bail!("fee cannot exceed 5000 bps (50%)");
                }

                // look up quote asset from the asset registry
                let asset_cache = app.view().assets().await?;
                let quote_metadata = asset_cache
                    .iter()
                    .find(|(_id, m)| m.symbol().to_lowercase() == quote_asset.to_lowercase()
                        || m.base_denom().denom.to_lowercase() == quote_asset.to_lowercase())
                    .map(|(_id, m)| m.clone())
                    .ok_or_else(|| anyhow::anyhow!(
                        "unknown quote asset '{}'. try 'penumbra' or check available assets with 'pcli view balance'",
                        quote_asset
                    ))?;
                let quote_asset_id = quote_metadata.id();

                // generate token id
                let mut nonce_bytes = [0u8; 32];
                rand::RngCore::fill_bytes(&mut OsRng, &mut nonce_bytes);
                let token_id = TokenFactoryId::new(nonce_bytes);

                // parse supply
                let supply: Amount = Amount::from(
                    supply
                        .parse::<u128>()
                        .context("invalid supply amount")?
                );

                // create metadata
                // `display` must reference one of the denom units; the only unit
                // is the base (`factory/{hex}`), so display == base. The friendly
                // name is carried by `name`/`symbol`.
                let metadata = asset::Metadata::try_from(
                    penumbra_sdk_proto::core::asset::v1::Metadata {
                        base: token_id.denom(),
                        display: token_id.denom(),
                        name: name.clone(),
                        symbol: name.clone(),
                        ..Default::default()
                    }
                )?;

                // the new token's asset id
                let new_token_id = metadata.id();

                // build bonding curve positions
                let positions = build_bonding_curve_positions(
                    OsRng,
                    new_token_id,
                    quote_asset_id,
                    supply,
                    *start_price,
                    *end_price,
                    *curve,
                    *num_positions,
                    *fee_bps,
                );

                println!("################################################################################");
                println!("########################### FAIR LAUNCH SUMMARY ################################");
                println!("################################################################################");
                println!("\nToken: {}", name);
                println!("Token ID: {}", hex::encode(token_id.as_bytes()));
                println!("Denom: {}", token_id.denom());
                println!("Total supply: {}", supply);
                println!("Quote asset: {} ({})", quote_metadata.symbol(), quote_metadata);
                println!("Price range: {} - {} {}/{}", start_price, end_price, quote_metadata.symbol(), name);
                println!("Curve type: {:?}", curve);
                println!("Positions: {}", num_positions);
                println!("Fee per position: {} bps", fee_bps);
                println!("\nThis transaction will:");
                println!("  1. Create the token with full supply");
                println!("  2. Burn the mint capability (no more tokens can ever be minted)");
                println!("  3. Open {} LP positions forming a bonding curve", num_positions);
                println!("  4. Burn all LP NFTs (liquidity is permanently locked)");
                println!("\nAfter this transaction, the entire supply will be available on the");
                println!("bonding curve. Because penumbra batches all dex operations per block,");
                println!("there is no frontrunning - everyone gets equal access.\n");

                if !*yes
                    && !Confirm::new()
                        .with_prompt("proceed with fair launch?")
                        .interact()?
                {
                    return Ok(());
                }

                // create action with enable_mint=true so we get the mint cap to burn
                let create_action = ActionTokenFactoryCreate::new(
                    token_id.clone(),
                    metadata.clone(),
                    supply,
                    true, // enable mint so we get the cap
                ).context("invalid create parameters")?;

                // The mint capability the create action produces (seq 0). Burning it
                // makes supply immutable. Derive it via the MintCapability type so the
                // asset id matches what `create` emits (factory/{id}/mint/0) rather
                // than a hand-built denom that would target a phantom asset.
                let mint_cap_value = MintCapability::initial(token_id.clone()).value();

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                // 1. create the token
                planner.token_factory_create(create_action);

                // 2. burn the mint capability
                let mint_cap_burn = ActionBurnPlan::new(&mut OsRng, mint_cap_value);
                planner.action_burn(mint_cap_burn);

                // 3. open all positions and burn their lpnfts
                for position in &positions {
                    planner.position_open(position.clone());
                    // Burn the opened-position LP NFT so the position can never be
                    // closed or withdrawn — the liquidity is locked. Derive the asset
                    // id via LpNft so it matches what `position_open` actually produces.
                    let lp_nft_value = Value {
                        amount: Amount::from(1u64),
                        asset_id: LpNft::new(position.id(), position::State::Opened).asset_id(),
                    };
                    let lp_burn = ActionBurnPlan::new(&mut OsRng, lp_nft_value);
                    planner.action_burn(lp_burn);
                }

                let plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;

                let tx_id = app.build_and_submit_transaction(plan).await?;
                println!("\nFair launch successful!");
                println!("Transaction ID: {}", tx_id);
                println!("\nToken {} is now live on the bonding curve.", name);
                println!("The entire supply is locked in immutable liquidity positions.");
                Ok(())
            }
        }
    }
}

/// Build LP positions that approximate a bonding curve.
///
/// The positions are funded with the new token and spread across the price range.
/// When bought, tokens move from low-price positions to buyers, and the quote asset
/// accumulates in those positions.
fn build_bonding_curve_positions<R: CryptoRngCore>(
    mut rng: R,
    token_id: asset::Id,
    quote_id: asset::Id,
    total_supply: Amount,
    start_price: f64,
    end_price: f64,
    curve_type: CurveType,
    num_positions: u32,
    fee_bps: u32,
) -> Vec<Position> {
    let mut positions = Vec::with_capacity(num_positions as usize);

    // trading pair: token -> quote (buying tokens costs quote asset)
    let pair = DirectedTradingPair::new(token_id, quote_id);

    // calculate token allocation per position
    // for a bonding curve, we want more tokens at lower prices
    let supply_per_position = total_supply.value() as f64 / num_positions as f64;

    for i in 0..num_positions {
        let t = i as f64 / (num_positions - 1) as f64;

        // calculate price at this position based on curve type
        let price = match curve_type {
            CurveType::Linear => start_price + t * (end_price - start_price),
            CurveType::Exponential => {
                // exponential interpolation: start * (end/start)^t
                start_price * (end_price / start_price).powf(t)
            }
        };

        // for bonding curves, we want supply distributed so early buyers get more tokens
        // simpler approach: equal supply per position
        let tokens_at_position = Amount::from(supply_per_position as u128);

        // p and q define the exchange rate: p units of asset2 per q units of asset1
        // price = p/q means 1 token costs `price` quote
        // we use a scaling factor for precision
        let scale = 1_000_000u128;
        let p = Amount::from((price * scale as f64) as u128);
        let q = Amount::from(scale);

        // reserves: we're selling tokens, so r1 has tokens, r2 is empty
        let reserves = Reserves {
            r1: tokens_at_position,
            r2: Amount::zero(),
        };

        let position = Position::new(&mut rng, pair, fee_bps, p, q, reserves);
        positions.push(position);
    }

    positions
}
