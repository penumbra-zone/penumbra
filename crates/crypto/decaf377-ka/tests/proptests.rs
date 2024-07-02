use decaf377_ka as ka;
use proptest::prelude::*;

fn fq_strategy() -> BoxedStrategy<decaf377::Fq> {
    any::<[u8; 32]>()
        .prop_map(|bytes| decaf377::Fq::from_le_bytes_mod_order(&bytes[..]))
        .boxed()
}

fn fr_strategy() -> BoxedStrategy<decaf377::Fr> {
    any::<[u8; 32]>()
        .prop_map(|bytes| decaf377::Fr::from_le_bytes_mod_order(&bytes[..]))
        .boxed()
}

proptest! {
    #[test]
    fn key_agreement_works(
        alice_sk in fr_strategy(),
        bob_sk in fr_strategy(),
    ) {
        let alice_sk = ka::Secret::new_from_field(alice_sk);
        let bob_sk = ka::Secret::new_from_field(bob_sk);

        let alice_pk = alice_sk.public();
        let bob_pk = bob_sk.public();

        let alice_ss = alice_sk.key_agreement_with(&bob_pk).unwrap();
        let bob_ss = bob_sk.key_agreement_with(&alice_pk).unwrap();

        assert_eq!(alice_ss, bob_ss);
    }

    #[test]
    fn diversified_key_agreement_works(
        alice_sk in fr_strategy(),
        bob_sk in fr_strategy(),
        div1 in fq_strategy(),
        div2 in fq_strategy(),
    ) {
        let alice_sk = ka::Secret::new_from_field(alice_sk);
        let bob_sk = ka::Secret::new_from_field(bob_sk);

        let gen1 = decaf377::Element::encode_to_curve(&div1);
        let gen2 = decaf377::Element::encode_to_curve(&div2);

        let alice_pk1 = alice_sk.diversified_public(&gen1);
        let alice_pk2 = alice_sk.diversified_public(&gen2);

        let bob_pk1 = bob_sk.diversified_public(&gen1);
        let bob_pk2 = bob_sk.diversified_public(&gen2);

        let bob_ss1 = bob_sk.key_agreement_with(&alice_pk1).unwrap();
        let bob_ss2 = bob_sk.key_agreement_with(&alice_pk2).unwrap();

        let alice_ss1 = alice_sk.key_agreement_with(&bob_pk1).unwrap();
        let alice_ss2 = alice_sk.key_agreement_with(&bob_pk2).unwrap();

        assert_eq!(alice_ss1, bob_ss1);
        assert_eq!(alice_ss2, bob_ss2);
    }
}
