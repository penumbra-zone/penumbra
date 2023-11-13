use penumbra_proto::core::component::ibc::v1alpha1::Ics20Withdrawal;

#[test]
fn successfully_serialization() {
    let raw = r#"
    {
      "amount": {
         "lo": "2000000"
    },
      "denom": {
        "denom": "ugm"
    },
      "destinationChainAddress": "osmo18275ps0x4gtvg8p52u577lcrve03jt2wmk8nc7",
      "returnAddress": {
        "altBech32m": "penumbra17a3cctdxve60qk02dwtvxgslmmqydddzuw3hgpy33c3fyqc6n6hke6fnkaqhmnf3g6gwlhm38a626sx9xhnfnxa9xnug2hl5u96tf962u9mtrkd2sxjqnesp8ezrmqwehsfs9f"
    },
      "timeoutHeight": {
        "revisionNumber": "5",
        "revisionHeight": "1000000"
    },
      "sourceChannel": "0"
    }
    "#;

    let w: Ics20Withdrawal = serde_json::from_str(raw).unwrap();

    // String to u64 works ✅
    assert_eq!(w.amount.clone().unwrap().lo, 2000000u64);
    assert_eq!(w.amount.unwrap().hi, 0u64);

    // String to u64 fails ❌
    assert_eq!(w.timeout_height.clone().unwrap().revision_number, 5u64);
    assert_eq!(
        w.timeout_height.clone().unwrap().revision_height,
        1000000u64
    );
}
