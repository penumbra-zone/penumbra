use penumbra_proto::core::component::ibc::v1alpha1::Ics20Withdrawal as PbIcs20Withdrawal;
use penumbra_shielded_pool::Ics20Withdrawal;

#[test]
fn test_height_err() {
    let data = r#"
       {
    "amount": {
        "lo": "12000000000000000000"
    },
    "denom": {
        "denom": "upenumbra"
    },
    "destinationChainAddress": "penumbra19zz058ttl8vhsypztc0gyl9yfs7jcn3906kgd3pzeh944klh8vf2ttx7qvscxwtuecw92cy6n55ttjn482q7ufpzwj5yem9xcvecrd2zc6vgctxzc3k7mnpg0lk8vved00e3g0",
    "returnAddress": {
        "inner": "UV9+hAPukWd2BCmjDdkgLL3V3dphBd92LUggxhSRRqX4bfP+utX/Z0B72WdzxbCaBbBv64eFVoCi4cdpGq6OSS6WDKRBLUcI3tTwO5qkugY="
    },
    "timeoutHeight": {
        "revisionNumber": "5",
        "revisionHeight": "1000000"
    },
    "timeoutTime": "1702798358943",
    "sourceChannel": "channel-0"
}
    "#;

    let withdrawal_proto: PbIcs20Withdrawal = serde_json::from_str(data).unwrap();
    let height = withdrawal_proto.clone().timeout_height.unwrap();
    assert_eq!(height.revision_number, 5u64); // 5 != 0 ❌
    assert_eq!(height.revision_height, 1000000u64); // 1000000 != 0 ❌

    let domain_type: Ics20Withdrawal = withdrawal_proto.try_into().unwrap();
    assert_eq!(domain_type.timeout_height.revision_number, 5u64);
    assert_eq!(domain_type.timeout_height.revision_height, 1000000u64);
}
