use std::collections::HashSet;

use proptest::prelude::*;

use penumbra_tct::{self as real, spec};

mod simulate;
use simulate::Simulate;

proptest! {
    #[test]
    fn test_simulate(
        actions in prop::collection::vec(
            any::<simulate::eternity::Action>(),
            1..10
        )
    ) {
        let mut spec = spec::eternity::Builder::default();
        let mut real = real::Eternity::default();
        actions.simulate(&mut spec, &mut real);
        let spec = spec.build();
    }
}
