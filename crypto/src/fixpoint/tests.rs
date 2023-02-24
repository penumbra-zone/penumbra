use super::*;

use proptest::prelude::*;

impl Arbitrary for U128x128 {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (any::<u128>(), any::<u128>())
            .prop_map(|(upper, lower)| Self(U256::from_words(upper, lower)))
            .boxed()
    }
}

proptest! {
    #[test]
    fn encoding_respects_ordering(lhs in any::<U128x128>(), rhs in any::<U128x128>()) {
        prop_assert_eq!(lhs.cmp(&rhs), lhs.to_bytes().cmp(&rhs.to_bytes()));
    }

}
