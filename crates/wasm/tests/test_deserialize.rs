use penumbra_proto::core::component::ibc::v1alpha1::Ics20Withdrawal as PbIcs20Withdrawal;
use penumbra_shielded_pool::Ics20Withdrawal;

#[test]
fn height_properly_serializes_from_json() {
    let data = r#"
        {
          "amount": {
            "lo": "12000000"
          },
          "denom": {
            "denom": "upenumbra"
          },
          "destinationChainAddress": "xyz",
          "returnAddress": {
            "inner": "by+DwROtdzWZu+W+gQ+e7pJ328aBf4Lng1dtnnkH971ebSC4O1+fQE+QmMNQ0iEg1/ARaF6yop4BurwW0Z1B7v0/o3AYchf6IEMYBxGyN18="
          },
          "timeoutHeight": {
            "revisionNumber": "5",
            "revisionHeight": "3928271"
          },
          "timeoutTime": "1701471437169",
          "sourceChannel": "channel-0"
        }
    "#;

    let withdrawal_proto: PbIcs20Withdrawal = serde_json::from_str(data).unwrap();
    let height = withdrawal_proto.clone().timeout_height.unwrap();
    assert_eq!(height.revision_number, 5u64);
    assert_eq!(height.revision_height, 3928271u64);

    let domain_type: Ics20Withdrawal = withdrawal_proto.try_into().unwrap();
    assert_eq!(domain_type.timeout_height.revision_number, 5u64);
    assert_eq!(domain_type.timeout_height.revision_height, 3928271u64);
}
