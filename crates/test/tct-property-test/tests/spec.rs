// use proptest::{arbitrary::*, prelude::*};

// use penumbra_tct::Commitment;

// mod simulate;
// use simulate::{Params, Simulate};

// const MAX_TIER_ACTIONS: usize = 10;
// const MAX_USED_COMMITMENTS: usize = 10;
// const MAX_UNUSED_COMMITMENTS: usize = 2;
// const MAX_OBSERVATIONS: usize = 10;

// This macro generates a simulation test for the given module
#[allow(unused_macros)]
macro_rules! spec_vs_impl {
    ($name:ident : $module:ident) => {
        proptest! {
            #![proptest_config(ProptestConfig {
                cases: 10_000, .. ProptestConfig::default()
            })]

            #[test]
            fn $name(
                (actions, observations) in (
                    prop::collection::vec(any::<Commitment>(), 1..MAX_USED_COMMITMENTS),
                    prop::collection::vec(any::<Commitment>(), 1..MAX_UNUSED_COMMITMENTS),
                ).prop_flat_map(|(used_commitments, unused_commitments)| (
                    prop::collection::vec(
                        any_with::<simulate::$module::Action>(
                            Params {
                                commitments: used_commitments.clone(),
                                max_tier_actions: MAX_TIER_ACTIONS
                            }
                        ),
                        0..MAX_TIER_ACTIONS
                    ),
                    prop::collection::vec(
                        any_with::<simulate::$module::Observation>(
                            {
                                used_commitments.clone().extend(unused_commitments);
                                used_commitments
                            }
                        ),
                        0..MAX_OBSERVATIONS
                    )
                ))
            ) {
                // Create initial spec and real versions of the thing under test
                let mut spec = Default::default();
                let mut real = Default::default();

                // Simulate the actions on both of them simultaneously
                actions.simulate(&mut spec, &mut real);

                // Build the spec builder into an immutable tree that can be observed
                let mut spec = spec.build();

                // Observe all the same observations on both the spec and the real implementation
                observations.simulate(&mut spec, &mut real);
            }
        }
    };
}

/*
// TODO: re-enable these tests

spec_vs_impl!(eternity_spec_vs_impl: eternity);

spec_vs_impl!(epoch_spec_vs_impl: epoch);

spec_vs_impl!(block_spec_vs_impl: block);
*/
