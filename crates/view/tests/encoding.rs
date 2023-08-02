use penumbra_proto::penumbra::core::transaction::v1alpha1 as pb;
use penumbra_transaction::Id;

#[test]
fn tx_hash_proto_roundtrip() {
    let literal = "bed8fb72fa69e893fde505f2bb8233ab75a5e574905b68924a26f9ba6de863c4";

    let hash = hex::decode(literal).unwrap();

    let proto = pb::Id {
        hash: hash.clone().into(),
    };

    let mut id = [0u8; 32];

    id.copy_from_slice(&hash);

    let id: penumbra_transaction::Id = Id(id);

    let from_proto: penumbra_transaction::Id = Id::try_from(proto.clone()).unwrap();

    let from_id: pb::Id = pb::Id::from(id);

    assert_eq!(proto.clone(), from_id);

    assert_eq!(id, from_proto);

    assert_eq!(literal, hex::encode(proto.clone().hash));

    assert_eq!(literal, hex::encode(id.0));
}
