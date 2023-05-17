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

#[test]
#[should_panic]
fn multiply_large_failure() {
    let a = 1788000000000000000000u128;
    let b = 1000000000000000000000u128;
    let a_fp: U128x128 = a.into();
    let b_fp: U128x128 = b.into();
    let _c_fp = (a_fp * b_fp).expect("overflow loudly!");
}
