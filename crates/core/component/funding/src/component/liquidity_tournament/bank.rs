use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::fixpoint::U128x128;

#[allow(dead_code)]
#[async_trait]
/// The bank strictly controls issuance of rewards in the liquidity tournament.
///
/// This ensures that rewards do not exceed the issuance budget, and are immediately
/// debited from the appropriate source (which happens to be the community pool),
/// and credited towards the appropriate destination (i.e. positions or new notes).
pub trait Bank: StateWrite {
    /// Move a fraction of our issuance budget towards an address, by minting a note.
    async fn reward_fraction_to_voter(
        &mut self,
        _fraction: U128x128,
        _voter: &Address,
    ) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Move a fraction of our issuance budget towards a position, increasing its reserves.
    async fn reward_fraction_to_position(
        &mut self,
        _fraction: U128x128,
        _lp: position::Id,
    ) -> anyhow::Result<()> {
        unimplemented!()
    }
}
