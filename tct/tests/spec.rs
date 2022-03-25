use proptest::{arbitrary::*, prelude::*};

use penumbra_tct::{self as real, spec, Commitment};

mod simulate;
use simulate::{Params, Simulate};

const MAX_TIER_ACTIONS: usize = 10;
const MAX_USED_COMMITMENTS: usize = 10;
const MAX_UNUSED_COMMITMENTS: usize = 2;
const OBSERVATIONS: usize = 10;

proptest! {
    #[test]
    fn test_simulate(
        (actions, observations) in (
            prop::collection::vec(any::<Commitment>(), 1..MAX_USED_COMMITMENTS),
            prop::collection::vec(any::<Commitment>(), 1..MAX_UNUSED_COMMITMENTS),
        ).prop_flat_map(|(used_commitments, unused_commitments)| (
            prop::collection::vec(
                any_with::<simulate::eternity::Action>(Params { commitments: used_commitments.clone(), max_tier_actions: MAX_TIER_ACTIONS }),
                0..MAX_TIER_ACTIONS
            ),
            prop::collection::vec(
                any_with::<simulate::eternity::Observation>({used_commitments.clone().extend(unused_commitments); used_commitments}),
                OBSERVATIONS
            )
        ))
    ) {
        let mut spec = spec::eternity::Builder::default();
        let mut real = real::Eternity::default();
        actions.simulate(&mut spec, &mut real);
        let mut spec = spec.build();
        observations.simulate(&mut spec, &mut real);
    }
}
