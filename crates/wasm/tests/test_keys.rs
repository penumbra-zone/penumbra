extern crate core;

use penumbra_wasm::keys::get_wallet_id;

#[test]
fn successfully_get_wallet_id() {
    let fvk_str = "penumbrafullviewingkey1sjeaceqzgaeye2ksnz8q73mp6rpx2ykdtzs8wurrnhwdn8vqwuxhxtjdndrjc74udjh0uch0tatnrd93q50wp9pfk86h3lgpew8lsqsz2a6la".to_string();
    let actual_bech32_str = get_wallet_id(&fvk_str).unwrap();
    let expected_bech32_str =
        "penumbrawalletid15r7q7qsf3hhsgj0g530n7ng9acdacmmx9ajknjz38dyt90u9gcgsmjre75".to_string();
    assert_eq!(expected_bech32_str, actual_bech32_str);
}

#[test]
fn raises_if_fvk_invalid() {
    let fvk_str = "invalid".to_string();
    let err = get_wallet_id(&fvk_str).unwrap_err();
    assert_eq!("invalid length", err.to_string());
}
