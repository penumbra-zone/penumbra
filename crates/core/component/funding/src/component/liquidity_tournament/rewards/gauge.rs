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
pub struct Gauge {
    total: u128,
    asset_tally: BTreeMap<asset::Id, (u128, BTreeMap<Vec<u8>, u128>)>,
}

impl Gauge {
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
    /// ```text
    ///  vote(A, p1, V) ; vote(A, p2, V) = vote(A, p1 + p2, V)`
    /// ```
    pub fn tally(&mut self, vote: asset::Id, power: u64, voter: Vec<u8>) {
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
    ) -> FinalizedGauge {
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
            let mut filtered_voters = BTreeMap::<Vec<u8>, u128>::new();
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
                        *filtered_voters.entry(voter.clone()).or_insert(0u128) += power;
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
pub struct FinalizedGauge {
    assets: Vec<(Share, asset::Id)>,
    voters: Vec<(Share, Vec<u8>)>,
}

impl FinalizedGauge {
    pub fn asset_shares(&self) -> impl Iterator<Item = (Share, asset::Id)> + use<'_> {
        self.assets.iter().copied()
    }

    pub fn voter_shares(&self) -> impl Iterator<Item = (Share, &[u8])> + use<'_> {
        self.voters.iter().map(|(s, v)| (*s, v.as_slice()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

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

    fn addr_for(byte: u8) -> Vec<u8> {
        vec![byte]
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

    fn almost_equal(share: Share, target: Share) -> bool {
        // 9 9s of precision
        share <= target
            && target.checked_sub(&share).unwrap() < Share::ratio(1, 1_000_000_000u64).unwrap()
    }

    fn shares_sum_to_1_inner(votes: Vec<(u8, u32, u8)>) {
        let gauge = {
            let mut out = Gauge::empty();
            for (asset, power, voter) in votes.iter().copied() {
                out.tally(asset_for(asset), power.into(), addr_for(voter));
            }
            out
        };
        let (assets, voters) = trace(gauge);
        let mut asset_sum: Share = Share::default();
        let mut voter_sum: Share = Share::default();
        for (_, x) in assets {
            asset_sum = asset_sum.checked_add(&x).unwrap();
        }
        for (_, x) in voters {
            voter_sum = voter_sum.checked_add(&x).unwrap();
        }
        // Because of rounding, need to test almost 1 instead
        assert!(almost_equal(asset_sum, Share::from(1u64)));
        assert!(almost_equal(voter_sum, Share::from(1u64)));
    }

    proptest! {
        #[test]
        fn shares_sum_to_1(votes in proptest::collection::vec((0u8..10, any::<u32>(), any::<u8>()), 1..100)) {
            shares_sum_to_1_inner(votes);
        }
    }

    #[test]
    fn test_no_votes() {
        let (assets, voters) = trace(Gauge::empty());
        assert!(assets.is_empty());
        assert!(voters.is_empty());
    }

    #[test]
    fn test_basic_votes() {
        let votes: Vec<(u8, u64, u8)> =
            vec![(0, 1, 0), (0, 2, 1), (1, 4, 0), (1, 8, 2), (2, 1, 100)];
        let mut gauge = Gauge::empty();
        for (asset, power, voter) in votes {
            gauge.tally(asset_for(asset), power, addr_for(voter));
        }
        let (assets, voters) = trace(gauge);
        assert!(almost_equal(assets[&asset_for(0u8)], create_share(3, 15)));
        assert!(almost_equal(assets[&asset_for(1u8)], create_share(12, 15)));
        assert!(!assets.contains_key(&asset_for(2)));
        assert!(almost_equal(voters[&addr_for(0u8)], create_share(5, 15)));
        assert!(almost_equal(voters[&addr_for(1u8)], create_share(2, 15)));
        assert!(almost_equal(voters[&addr_for(2u8)], create_share(8, 15)));
        assert!(!voters.contains_key(&addr_for(100)));
    }
}
