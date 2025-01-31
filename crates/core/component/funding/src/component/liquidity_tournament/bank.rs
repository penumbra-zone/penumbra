use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_community_pool::StateWriteExt as _;
use penumbra_sdk_dex::component::PositionManager as _;
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_distributions::component::{StateReadExt as _, StateWriteExt as _};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::{fixpoint::U128x128, Amount};
use penumbra_sdk_sct::component::clock::EpochRead as _;
use penumbra_sdk_sct::CommitmentSource;
use penumbra_sdk_shielded_pool::component::NoteManager as _;
use penumbra_sdk_txhash::TransactionId;

/// Move a fraction of the budget, up to the entire budget, from the community pool.
///
/// This will return the amount pulled (in terms of the staking token).
async fn appropriate_budget(
    mut state: impl StateWrite,
    fraction: U128x128,
) -> anyhow::Result<Amount> {
    // For our purposes, no budget is the same as an explicit budget of 0.
    let budget = state
        .get_lqt_reward_issuance_for_epoch()
        .unwrap_or_default();
    // IMO this method should have just expected to begin with.
    let portion = fraction
        .apply_to_amount(&budget)
        .expect(&format!("failed to apply {fraction} to {budget:?}"));
    // This fraction may be > 1.0, in which case we need to not over-appropriate
    let (new_budget, portion) = match budget.checked_sub(&portion) {
        Some(new_budget) => (new_budget, portion),
        // portion > budget, so eat the whole budget.
        None => (0u64.into(), budget),
    };
    state.set_lqt_reward_issuance_for_epoch(new_budget);
    state
        .community_pool_withdraw(Value {
            asset_id: *STAKING_TOKEN_ASSET_ID,
            amount: portion,
        })
        .await?;
    Ok(portion)
}

#[allow(dead_code)]
#[async_trait]
/// The bank strictly controls issuance of rewards in the liquidity tournament.
///
/// This ensures that rewards do not exceed the issuance budget, and are immediately
/// debited from the appropriate source (which happens to be the community pool),
/// and credited towards the appropriate destination (i.e. positions or new notes).
pub trait Bank: StateWrite + Sized {
    /// Move a fraction of our issuance budget towards an address, by minting a note.
    async fn reward_fraction_to_voter(
        &mut self,
        fraction: U128x128,
        voter: &Address,
        tx_hash: TransactionId,
    ) -> anyhow::Result<()> {
        let reward = appropriate_budget(&mut self, fraction).await?;
        let epoch = self
            .get_current_epoch()
            .await
            .expect("should be able to read current epoch");
        self.mint_note(
            Value {
                asset_id: *STAKING_TOKEN_ASSET_ID,
                amount: reward,
            },
            voter,
            CommitmentSource::LiquidityTournamentReward {
                epoch: epoch.index,
                tx_hash,
            },
        )
        .await?;
        Ok(())
    }

    /// Move a fraction of our issuance budget towards a position, increasing its reserves.
    async fn reward_fraction_to_position(
        &mut self,
        fraction: U128x128,
        lp: position::Id,
    ) -> anyhow::Result<()> {
        let reward = appropriate_budget(&mut self, fraction).await?;
        self.reward_position(lp, reward).await?;
        Ok(())
    }
}
