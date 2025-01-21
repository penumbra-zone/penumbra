extern crate core;

use std::str::FromStr;

use penumbra_sdk_keys::keys::{SeedPhrase, SpendKey};
use penumbra_sdk_proto::serializers::bech32str;

#[test]
fn wallet_id_to_bech32() {
    let seed = SeedPhrase::from_str("comfort ten front cycle churn burger oak absent rice ice urge result art couple benefit cabbage frequent obscure hurry trick segment cool job debate").unwrap();
    let spend_key = SpendKey::from_seed_phrase_bip39(seed, 0);
    let fvk = spend_key.full_viewing_key();
    let wallet_id = fvk.wallet_id();
    let actual_bech32_str = wallet_id.to_string();

    let expected_bech32_str =
        "penumbrawalletid15r7q7qsf3hhsgj0g530n7ng9acdacmmx9ajknjz38dyt90u9gcgsmjre75".to_string();

    assert_eq!(expected_bech32_str, actual_bech32_str);

    // Decoding returns original inner vec
    let inner_bytes = bech32str::decode(
        &expected_bech32_str,
        bech32str::wallet_id::BECH32_PREFIX,
        bech32str::Bech32m,
    )
    .unwrap();

    assert_eq!(wallet_id.0, inner_bytes.as_slice());
}
