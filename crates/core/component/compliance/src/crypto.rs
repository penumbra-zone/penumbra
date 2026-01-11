//! Cryptographic primitives for compliance encryption/decryption.
//!
//! This module implements ECDH-based encryption for compliance data using:
//! - Elliptic curve Diffie-Hellman for shared secret derivation
//! - Poseidon stream cipher (ZK-friendly, circuit-compatible)

use anyhow::Context;
use decaf377::{Element, Fq, Fr};
use once_cell::sync::Lazy;
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::keys::{AddressComplianceKey, KeyType};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use rand_core::{CryptoRng, RngCore};

use crate::structs::ComplianceCiphertext;

/// Domain separator for Poseidon stream cipher seed derivation.
pub static COMPLIANCE_STREAM_CIPHER_DOMAIN: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(
        blake2b_simd::blake2b(b"penumbra.compliance.poseidon_stream").as_bytes(),
    )
});

/// The "black hole" compliance key for unregulated assets.
pub static BLACK_HOLE_ACK: Lazy<Element> = Lazy::new(|| Element::GENERATOR);

/// Decrypted compliance data.
#[derive(Clone, Debug)]
pub struct DecryptedComplianceData {
    /// The asset ID being transacted.
    pub asset_id: asset::Id,
    /// The amount being transacted.
    pub amount: Amount,
    /// The "self" diversified generator (the party who can decrypt this ciphertext).
    pub self_diversified_generator: Element,
    /// The "self" transmission key.
    pub self_transmission_key: [u8; 32],
    /// The counterparty diversified generator (the other party in the transaction).
    pub counterparty_diversified_generator: Element,
    /// The counterparty transmission key.
    pub counterparty_transmission_key: [u8; 32],
}

/// Encrypt compliance details using Poseidon stream cipher.
///
/// This function creates a circuit-compatible ciphertext using Poseidon hashing,
/// ensuring the encryption logic matches exactly what can be verified in R1CS constraints.
///
/// # Arguments
/// * `rng` - Random number generator for ephemeral key generation
/// * `self_ack` - The "self" party's wallet compliance key (who can decrypt this)
/// * `self_address` - The "self" party's address (who can decrypt this)
/// * `date` - The date (day index) for key derivation
/// * `asset_id` - The asset being sent
/// * `amount` - The amount being sent
/// * `counterparty_address` - The other party in the transaction
///
/// # Returns
/// A tuple of `(ComplianceCiphertext, Fr ephemeral_secret)` where:
/// - The ciphertext contains the EPK and encrypted field elements as bytes
/// - The ephemeral secret must be provided to the circuit as a private witness
///
/// ## Tiered Encryption
///
/// Each ciphertext part is encrypted with a different key type:
/// - detection_tag: Encrypted with Detection key (for asset_id scanning)
/// - encrypted_core: Encrypted with Core key (amount + self address)
/// - encrypted_extension: Encrypted with Extension key (counterparty address)
///
/// This enables selective disclosure - issuers can share only Detection keys
/// with scanners, or Detection + Core keys with auditors.
pub fn encrypt_compliance_details(
    mut rng: impl RngCore + CryptoRng,
    self_ack: &AddressComplianceKey,
    self_address: &Address,
    date: u64,
    asset_id: asset::Id,
    amount: Amount,
    counterparty_address: Address,
) -> anyhow::Result<(ComplianceCiphertext, Fr)> {
    // 1. Extract the diversified generator B_d from the self address
    let diversified_generator = self_address.diversified_generator();
    let diversifier = self_address.diversifier();

    // 2. Derive THREE daily public keys (one per key type)
    let pk_detection = self_ack.derive_daily_public_key(KeyType::Detection, date, &diversifier);
    let pk_core = self_ack.derive_daily_public_key(KeyType::Core, date, &diversifier);
    let pk_extension = self_ack.derive_daily_public_key(KeyType::Extension, date, &diversifier);

    // 3. Generate ephemeral secret r (shared across all key types)
    let ephemeral_secret = Fr::rand(&mut rng);

    // 4. Compute ephemeral public key: R = r * B_d
    let epk = diversified_generator * ephemeral_secret;

    // 5. Compute THREE shared secrets (one per key type)
    let ss_detection = pk_detection * ephemeral_secret;
    let ss_core = pk_core * ephemeral_secret;
    let ss_extension = pk_extension * ephemeral_secret;

    // 6. Derive THREE Poseidon stream cipher seeds
    let epk_fq = epk.vartime_compress_to_field();
    let seed_detection = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_detection.vartime_compress_to_field(), epk_fq),
    );
    let seed_core = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_core.vartime_compress_to_field(), epk_fq),
    );
    let seed_extension = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_extension.vartime_compress_to_field(), epk_fq),
    );

    // 7. Prepare plaintext segments
    //
    // Detection (32 bytes = 1 Fq element):
    //   - asset_id (32 bytes) - already an Fq, safe to encode directly
    //
    // Core (80 bytes → 3 Fq elements using 31-byte chunks):
    //   - amount (16 bytes)
    //   - self_diversified_generator (32 bytes)
    //   - self_transmission_key (32 bytes)
    //   Total: 80 bytes, ceil(80/31) = 3 Fq elements = 96 bytes ciphertext
    //
    // Extension (64 bytes → 3 Fq elements using 31-byte chunks):
    //   - counterparty_diversified_generator (32 bytes)
    //   - counterparty_transmission_key (32 bytes)
    //   Total: 64 bytes, ceil(64/31) = 3 Fq elements = 96 bytes ciphertext
    //
    // NOTE: We use 31-byte chunks because Fq field order is ~2^252, so 32-byte
    // values could overflow and get reduced mod q, corrupting the data.

    // Detection plaintext: asset_id only
    let detection_plaintext = asset_id.0;

    // Core plaintext: amount + self address
    let mut core_bytes = Vec::with_capacity(80);
    core_bytes.extend_from_slice(&amount.to_le_bytes());
    core_bytes.extend_from_slice(&self_address.diversified_generator().vartime_compress().0);
    core_bytes.extend_from_slice(&self_address.transmission_key().0);

    // Extension plaintext: counterparty address
    let mut extension_bytes = Vec::with_capacity(64);
    extension_bytes.extend_from_slice(
        &counterparty_address
            .diversified_generator()
            .vartime_compress()
            .0,
    );
    extension_bytes.extend_from_slice(&counterparty_address.transmission_key().0);

    // 8. Encrypt detection_tag (1 Fq element, counter = 0)
    let detection_keystream =
        poseidon377::hash_2(&seed_detection, (Fq::from(0u64), seed_detection));
    let detection_ciphertext = detection_plaintext + detection_keystream;
    let detection_tag: [u8; 32] = detection_ciphertext.to_bytes();

    // 9. Encrypt core data (pack into Fq elements using 31-byte chunks, then encrypt)
    let mut encrypted_core = Vec::new();
    for (i, chunk) in core_bytes.chunks(31).enumerate() {
        let mut buf = [0u8; 32];
        buf[0..chunk.len()].copy_from_slice(chunk);
        let plaintext_fq = Fq::from_le_bytes_mod_order(&buf);
        let counter = Fq::from(i as u64);
        let keystream = poseidon377::hash_2(&seed_core, (counter, seed_core));
        let ciphertext_fq = plaintext_fq + keystream;
        encrypted_core.extend_from_slice(&ciphertext_fq.to_bytes());
    }

    // 10. Encrypt extension data (pack into Fq elements using 31-byte chunks, then encrypt)
    let mut encrypted_extension = Vec::new();
    for (i, chunk) in extension_bytes.chunks(31).enumerate() {
        let mut buf = [0u8; 32];
        buf[0..chunk.len()].copy_from_slice(chunk);
        let plaintext_fq = Fq::from_le_bytes_mod_order(&buf);
        let counter = Fq::from(i as u64);
        let keystream = poseidon377::hash_2(&seed_extension, (counter, seed_extension));
        let ciphertext_fq = plaintext_fq + keystream;
        encrypted_extension.extend_from_slice(&ciphertext_fq.to_bytes());
    }

    Ok((
        ComplianceCiphertext::new(epk, detection_tag, encrypted_core, encrypted_extension),
        ephemeral_secret,
    ))
}

/// Encrypt compliance details for BOTH sender and receiver (dual encryption).
///
/// This creates two ciphertexts:
/// 1. Sender ciphertext: Encrypted for sender_ack, contains (amount, asset, sender_addr, receiver_addr)
/// 2. Receiver ciphertext: Encrypted for receiver_ack, contains (amount, asset, receiver_addr, sender_addr)
///
/// # Arguments
/// * `rng` - Random number generator
/// * `sender_ack` - Sender's wallet compliance key
/// * `sender_address` - Sender's address
/// * `receiver_ack` - Receiver's wallet compliance key
/// * `receiver_address` - Receiver's address
/// * `date` - Day index for key derivation
/// * `asset_id` - Asset being transacted
/// * `amount` - Amount being transacted
///
/// # Returns
/// Tuple of (sender_ciphertext, sender_ephemeral, receiver_ciphertext, receiver_ephemeral)
pub fn encrypt_compliance_details_dual(
    mut rng: impl RngCore + CryptoRng,
    sender_ack: &AddressComplianceKey,
    sender_address: &Address,
    receiver_ack: &AddressComplianceKey,
    receiver_address: &Address,
    date: u64,
    asset_id: asset::Id,
    amount: Amount,
) -> anyhow::Result<(ComplianceCiphertext, Fr, ComplianceCiphertext, Fr)> {
    // Encrypt for sender (sender can decrypt their own sends)
    let (sender_ciphertext, sender_ephemeral) = encrypt_compliance_details(
        &mut rng,
        sender_ack,
        sender_address,
        date,
        asset_id,
        amount,
        receiver_address.clone(),
    )?;

    // Encrypt for receiver (receiver can decrypt their own receives)
    let (receiver_ciphertext, receiver_ephemeral) = encrypt_compliance_details(
        &mut rng,
        receiver_ack,
        receiver_address,
        date,
        asset_id,
        amount,
        sender_address.clone(),
    )?;

    Ok((
        sender_ciphertext,
        sender_ephemeral,
        receiver_ciphertext,
        receiver_ephemeral,
    ))
}

/// Decrypt compliance details using Poseidon stream cipher with tiered keys.
///
/// This function reverses the encryption process to recover the original data.
/// Each ciphertext part requires its corresponding shared secret:
/// - detection_tag: decrypted with ss_detection
/// - encrypted_core: decrypted with ss_core
/// - encrypted_extension: decrypted with ss_extension
///
/// # Arguments
/// * `ss_detection` - Shared secret for detection key: `dmk_detection * EPK`
/// * `ss_core` - Shared secret for core key: `dmk_core * EPK`
/// * `ss_extension` - Shared secret for extension key: `dmk_extension * EPK`
/// * `epk` - The ephemeral public key from the ciphertext
/// * `ciphertext` - The compliance ciphertext to decrypt
///
/// # Returns
/// The decrypted compliance data, or an error if decryption fails
pub fn decrypt_compliance_details(
    ss_detection: &Element,
    ss_core: &Element,
    ss_extension: &Element,
    epk: &Element,
    ciphertext: &ComplianceCiphertext,
) -> anyhow::Result<DecryptedComplianceData> {
    // 1. Derive seeds for each key type
    let epk_fq = epk.vartime_compress_to_field();

    let seed_detection = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_detection.vartime_compress_to_field(), epk_fq),
    );
    let seed_core = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_core.vartime_compress_to_field(), epk_fq),
    );
    let seed_extension = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_extension.vartime_compress_to_field(), epk_fq),
    );

    // 2. Decrypt detection_tag (asset_id) - 1 Fq element
    let detection_ciphertext_fq = Fq::from_le_bytes_mod_order(&ciphertext.detection_tag);
    let detection_keystream =
        poseidon377::hash_2(&seed_detection, (Fq::from(0u64), seed_detection));
    let asset_id_fq = detection_ciphertext_fq - detection_keystream;

    // Validate asset_id
    let asset_id_bytes = asset_id_fq.to_bytes();
    let asset_id_validated =
        Fq::from_bytes_checked(&asset_id_bytes).map_err(|_| anyhow::anyhow!("invalid asset_id"))?;
    let asset_id = asset::Id(asset_id_validated);

    // 3. Decrypt core data (amount + self address) - 3 Fq elements (80 bytes plaintext)
    let mut core_plaintext_bytes = Vec::new();
    for (i, chunk) in ciphertext.encrypted_core.chunks(32).enumerate() {
        let mut buf = [0u8; 32];
        buf[0..chunk.len()].copy_from_slice(chunk);
        let ciphertext_fq = Fq::from_le_bytes_mod_order(&buf);
        let counter = Fq::from(i as u64);
        let keystream = poseidon377::hash_2(&seed_core, (counter, seed_core));
        let plaintext_fq = ciphertext_fq - keystream;
        // Take 31 bytes from each Fq (to match 31-byte chunk encoding)
        let fq_bytes = plaintext_fq.to_bytes();
        let bytes_to_take = 31.min(80 - core_plaintext_bytes.len());
        core_plaintext_bytes.extend_from_slice(&fq_bytes[0..bytes_to_take]);
    }

    // 4. Decrypt extension data (counterparty address) - 3 Fq elements (64 bytes plaintext)
    let mut extension_plaintext_bytes = Vec::new();
    for (i, chunk) in ciphertext.encrypted_extension.chunks(32).enumerate() {
        let mut buf = [0u8; 32];
        buf[0..chunk.len()].copy_from_slice(chunk);
        let ciphertext_fq = Fq::from_le_bytes_mod_order(&buf);
        let counter = Fq::from(i as u64);
        let keystream = poseidon377::hash_2(&seed_extension, (counter, seed_extension));
        let plaintext_fq = ciphertext_fq - keystream;
        let fq_bytes = plaintext_fq.to_bytes();
        let bytes_to_take = 31.min(64 - extension_plaintext_bytes.len());
        extension_plaintext_bytes.extend_from_slice(&fq_bytes[0..bytes_to_take]);
    }

    // 5. Parse core plaintext: amount (16) || self_div_gen (32) || self_trans_key (32)
    if core_plaintext_bytes.len() < 80 {
        anyhow::bail!(
            "core plaintext too short: expected 80 bytes, got {}",
            core_plaintext_bytes.len()
        );
    }

    let amount_bytes: [u8; 16] = core_plaintext_bytes[0..16]
        .try_into()
        .context("failed to extract amount")?;
    let amount = Amount::from_le_bytes(amount_bytes);

    let self_div_gen_bytes: [u8; 32] = core_plaintext_bytes[16..48]
        .try_into()
        .context("failed to extract self diversified generator")?;
    let self_div_gen = decaf377::Encoding(self_div_gen_bytes)
        .vartime_decompress()
        .map_err(|_| anyhow::anyhow!("failed to decompress self diversified generator"))?;

    let self_trans_key_bytes: [u8; 32] = core_plaintext_bytes[48..80]
        .try_into()
        .context("failed to extract self transmission key")?;

    // 6. Parse extension plaintext: counterparty_div_gen (32) || counterparty_trans_key (32)
    if extension_plaintext_bytes.len() < 64 {
        anyhow::bail!(
            "extension plaintext too short: expected 64 bytes, got {}",
            extension_plaintext_bytes.len()
        );
    }

    let counterparty_div_gen_bytes: [u8; 32] = extension_plaintext_bytes[0..32]
        .try_into()
        .context("failed to extract counterparty diversified generator")?;
    let counterparty_div_gen = decaf377::Encoding(counterparty_div_gen_bytes)
        .vartime_decompress()
        .map_err(|_| anyhow::anyhow!("failed to decompress counterparty diversified generator"))?;

    let counterparty_trans_key_bytes: [u8; 32] = extension_plaintext_bytes[32..64]
        .try_into()
        .context("failed to extract counterparty transmission key")?;

    // 7. Return the decrypted data
    Ok(DecryptedComplianceData {
        asset_id,
        amount,
        self_diversified_generator: self_div_gen,
        self_transmission_key: self_trans_key_bytes,
        counterparty_diversified_generator: counterparty_div_gen,
        counterparty_transmission_key: counterparty_trans_key_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_sdk_keys::keys::{Diversifier, MasterComplianceKey};
    use rand_core::OsRng;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut rng = OsRng;

        // Setup: Create a master key and derive a wallet key
        let msk = Fr::rand(&mut rng);
        let master_key = MasterComplianceKey::new(msk);

        let diversifier = Diversifier([1u8; 16]);
        let address_key = master_key.derive_address_key(&diversifier);

        // Create a test address using the SAME diversifier (crucial for ECDH to work)
        let random_scalar = Fr::rand(&mut rng);
        let random_point = decaf377::Element::GENERATOR * random_scalar;
        let pk_d = decaf377_ka::Public(random_point.vartime_compress().0);

        let mut ck_d_bytes = [0u8; 32];
        rng.fill_bytes(&mut ck_d_bytes);
        let ck_d = decaf377_fmd::ClueKey(ck_d_bytes);

        let recipient_address =
            Address::from_components(diversifier, pk_d, ck_d).expect("valid address components");

        // Encryption parameters
        let date = 19000u64;
        let asset_id = asset::Id(Fq::from(42u64));
        let amount = Amount::from(1000u128);
        let counterparty_address = Address::dummy(&mut rng);

        // Encrypt
        let (ciphertext, _ephemeral_secret) = encrypt_compliance_details(
            &mut rng,
            &address_key,
            &recipient_address,
            date,
            asset_id,
            amount,
            counterparty_address.clone(),
        )
        .expect("encryption should succeed");

        // Issuer side: Derive all three daily keys and compute shared secrets
        let daily_keys = master_key.derive_daily_keys(date);
        let ss_detection = ciphertext.epk * daily_keys.detection.inner();
        let ss_core = ciphertext.epk * daily_keys.core.inner();
        let ss_extension = ciphertext.epk * daily_keys.extension.inner();

        // Decrypt
        let decrypted = decrypt_compliance_details(
            &ss_detection,
            &ss_core,
            &ss_extension,
            &ciphertext.epk,
            &ciphertext,
        )
        .expect("decryption should succeed");

        // Verify
        assert_eq!(decrypted.asset_id, asset_id, "asset_id should match");
        assert_eq!(decrypted.amount, amount, "amount should match");
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let mut rng = OsRng;

        // Setup: Create a master key and derive a wallet key
        let msk = Fr::rand(&mut rng);
        let master_key = MasterComplianceKey::new(msk);

        let diversifier = Diversifier([1u8; 16]);
        let address_key = master_key.derive_address_key(&diversifier);

        let recipient_address = Address::dummy(&mut rng);
        let counterparty_address = Address::dummy(&mut rng);

        // Encrypt
        let date = 19000u64;
        let (ciphertext, _ephemeral_secret) = encrypt_compliance_details(
            &mut rng,
            &address_key,
            &recipient_address,
            date,
            asset::Id(Fq::from(42u64)),
            Amount::from(1000u128),
            counterparty_address,
        )
        .expect("encryption should succeed");

        // Try to decrypt with a DIFFERENT master key
        let wrong_msk = Fr::rand(&mut rng);
        let wrong_master_key = MasterComplianceKey::new(wrong_msk);
        let wrong_daily_keys = wrong_master_key.derive_daily_keys(date);
        let wrong_ss_detection = ciphertext.epk * wrong_daily_keys.detection.inner();
        let wrong_ss_core = ciphertext.epk * wrong_daily_keys.core.inner();
        let wrong_ss_extension = ciphertext.epk * wrong_daily_keys.extension.inner();

        // Decryption should fail (wrong shared secret will produce garbage plaintext)
        let result = decrypt_compliance_details(
            &wrong_ss_detection,
            &wrong_ss_core,
            &wrong_ss_extension,
            &ciphertext.epk,
            &ciphertext,
        );
        assert!(result.is_err(), "decryption with wrong key should fail");
    }

    #[test]
    fn test_decrypt_with_wrong_date_fails() {
        let mut rng = OsRng;

        let msk = Fr::rand(&mut rng);
        let master_key = MasterComplianceKey::new(msk);

        let diversifier = Diversifier([1u8; 16]);
        let address_key = master_key.derive_address_key(&diversifier);

        let recipient_address = Address::dummy(&mut rng);
        let counterparty_address = Address::dummy(&mut rng);

        // Encrypt for date 19000
        let date = 19000u64;
        let (ciphertext, _ephemeral_secret) = encrypt_compliance_details(
            &mut rng,
            &address_key,
            &recipient_address,
            date,
            asset::Id(Fq::from(42u64)),
            Amount::from(1000u128),
            counterparty_address,
        )
        .expect("encryption should succeed");

        // Try to decrypt with a DIFFERENT date
        let wrong_date = 19001u64;
        let wrong_daily_keys = master_key.derive_daily_keys(wrong_date);
        let wrong_ss_detection = ciphertext.epk * wrong_daily_keys.detection.inner();
        let wrong_ss_core = ciphertext.epk * wrong_daily_keys.core.inner();
        let wrong_ss_extension = ciphertext.epk * wrong_daily_keys.extension.inner();

        // Decryption with wrong date should either fail OR return garbage data
        let result = decrypt_compliance_details(
            &wrong_ss_detection,
            &wrong_ss_core,
            &wrong_ss_extension,
            &ciphertext.epk,
            &ciphertext,
        );
        match result {
            Err(_) => {
                // Expected: decryption failed (random bytes didn't form valid field elements)
            }
            Ok(decrypted) => {
                // Also valid: decryption succeeded but produced garbage data
                assert!(
                    decrypted.asset_id != asset::Id(Fq::from(42u64))
                        || decrypted.amount != Amount::from(1000u128),
                    "decryption with wrong date should not produce correct data"
                );
            }
        }
    }
}
