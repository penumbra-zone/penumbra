use std::collections::BTreeMap;

use penumbra_sdk_asset::asset;
use penumbra_sdk_num::{fixpoint::U128x128, Percentage};

/// The type we use for "shares" of the vote pie.
pub type Share = U128x128;

// exists to isolate decisions about division edge cases.
fn create_share(portion: u128, total: u128) -> Share {
    Share::ratio(portion, total.max(1)).expect("not dividing by 0")
}

/// A gauge is used to tally up votes in the tournament.
#[derive(Clone, Debug)]
pub struct Gauge<'s> {
    total: u128,
    asset_tally: BTreeMap<asset::Id, (u128, BTreeMap<&'s [u8], u128>)>,
}

impl<'s> Gauge<'s> {
    /// The state with no votes.
    pub fn empty() -> Self {
        Self {
            total: 0u128,
            asset_tally: BTreeMap::new(),
        }
    }

    /// Tally a vote into this gauge.
    ///
    /// Voting twice is the same as voting once with that combined power:
    /// ```
    ///  vote(A, p1, V) ; vote(A, p2, V) = vote(A, p1 + p2, V)`
    /// ```
    pub fn tally(&mut self, vote: asset::Id, power: u64, voter: &'s [u8]) {
        let power = u128::from(power);
        // Increment the total vote power, then per asset, then per voter.
        self.total += power;
        let asset_entry = self
            .asset_tally
            .entry(vote)
            .or_insert((0u128, BTreeMap::new()));
        asset_entry.0 += power;
        *asset_entry.1.entry(voter).or_insert(0u128) += power;
    }

    /// Finish tallying up votes, producing a finalized gauge for calculating rewards.
    ///
    /// `gauge_threshold` is the percentage of the vote an asset much reach in order
    pub fn finalize(
        self,
        gauge_threshold: Percentage,
        max_voters_per_asset: usize,
    ) -> FinalizedGauge<'s> {
        let gauge_share = Share::from(gauge_threshold);
        // First, let's figure out what assets remain after the threshold.
        let assets = {
            // We'll accumulate the power of the filtered assets here,
            let mut filtered_power = 0u128;
            // and those assets here. We store the raw power for now.
            let mut filtered_assets = Vec::<(u128, asset::Id)>::new();
            for (&asset, &(power, _)) in self.asset_tally.iter() {
                let share = create_share(power, self.total);
                // Disregard unpopular assets.
                if share < Share::from(gauge_share) {
                    continue;
                }
                filtered_power += power;
                filtered_assets.push((power, asset));
            }
            // Now, we need to figure out what share of the remaining power each asset is.
            let assets = filtered_assets
                .into_iter()
                .map(|(power, asset)| (create_share(power, filtered_power), asset))
                .collect::<Vec<_>>();

            assets
        };
        // Now, let's figure out the remaining voters, and the share they have of *that* pie.
        let voters = {
            let mut voters_power = 0u128;
            // The idea here is to take the top N voters for each asset, and put them in this map.
            let mut filtered_voters = BTreeMap::<&'s [u8], u128>::new();
            for (_, asset) in assets.iter() {
                let ranked_voters_for_this_asset = self.asset_tally[asset]
                    .1
                    .iter()
                    .map(|(voter, power)| (power, voter))
                    .collect::<BTreeMap<_, _>>();
                // Now, by iterating in reverse, we can take the top N.
                ranked_voters_for_this_asset
                    .into_iter()
                    .rev()
                    .take(max_voters_per_asset)
                    .for_each(|(power, voter)| {
                        voters_power += power;
                        *filtered_voters.entry(voter).or_insert(0u128) += power;
                    });
            }
            // Now, convert this into the share each voter has;
            let voters = filtered_voters
                .into_iter()
                .map(|(voter, power)| (create_share(power, voters_power), voter))
                .collect::<Vec<_>>();
            voters
        };

        FinalizedGauge { assets, voters }
    }
}

/// The result of the gauge after tallying votes.
///
/// This allows easily querying which assets received what share of the vote,
/// and which voters
#[derive(Clone, Debug)]
pub struct FinalizedGauge<'s> {
    assets: Vec<(Share, asset::Id)>,
    voters: Vec<(Share, &'s [u8])>,
}

impl<'s> FinalizedGauge<'s> {
    pub fn asset_shares(&self) -> impl Iterator<Item = (Share, asset::Id)> + use<'_> {
        self.assets.iter().copied()
    }

    pub fn voter_shares(&self) -> impl Iterator<Item = (Share, &'s [u8])> + use<'s, '_> {
        self.voters.iter().copied()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use penumbra_sdk_governance::VotingReceiptToken;
    use proptest::prelude::*;
    use rand_chacha::rand_core::SeedableRng as _;

    fn finalize(gauge: Gauge) -> FinalizedGauge {
        gauge.finalize(Percentage::from_percent(10), 10)
    }

    fn trace(gauge: Gauge) -> (BTreeMap<asset::Id, Share>, BTreeMap<Vec<u8>, Share>) {
        let result = finalize(gauge);
        (
            result.asset_shares().map(|(s, a)| (a, s)).collect(),
            result
                .voter_shares()
                .map(|(s, v)| (v.to_vec(), s))
                .collect(),
        )
    }

    // so that we can easily pass in references to slices.
    const ALL_BYTES: [u8; 256] = {
        let mut arr = [0u8; 256];
        let mut i = 0;
        while i < 256 {
            arr[i] = i as u8;
            i += 1;
        }
        arr
    };

    fn addr_for(byte: u8) -> &'static [u8] {
        let i = usize::from(byte);
        &ALL_BYTES[i..i + 1]
    }

    fn asset_for(byte: u8) -> asset::Id {
        asset::Id(u64::from(byte).into())
    }

    fn voting_power_combines_inner(votes: Vec<(u8, u32, u8)>) {
        // Calculate gauge without deduplication.
        let gauge = {
            let mut out = Gauge::empty();
            for (asset, power, voter) in votes.iter().copied() {
                out.tally(asset_for(asset), power.into(), addr_for(voter));
            }
            out
        };
        let gauge_dedup = {
            let mut deduped = BTreeMap::new();
            for (asset, power, voter) in votes.into_iter() {
                *deduped.entry((asset, voter)).or_insert(0u64) += u64::from(power);
            }
            let mut out = Gauge::empty();
            for ((asset, voter), power) in deduped.into_iter() {
                out.tally(asset_for(asset), power, addr_for(voter));
            }
            out
        };
        assert_eq!(trace(gauge), trace(gauge_dedup));
    }

    proptest! {
        #[test]
        fn voting_power_combines(votes in proptest::collection::vec((0u8..10, any::<u32>(), any::<u8>()), 0..100)) {
            voting_power_combines_inner(votes);
        }
    }

    #[test]
    fn test_no_votes() {
        let (assets, voters) = trace(finalize(Gauge::empty()));
        assert!(assets.is_empty());
        assert!(voters.is_empty());
    }
}
