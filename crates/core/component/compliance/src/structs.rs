use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::{keys::AddressComplianceKey, Address};
use penumbra_sdk_proto::penumbra::core::component::compliance::v1 as pb;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_tct::StateCommitment;
use serde::{Deserialize, Serialize};

/// Compliance plaintext layout constants.
/// These define the byte sizes for each field in the compliance plaintext that gets encrypted.
/// The circuit's bit-packing logic MUST match these exact sizes.
pub const AMOUNT_BYTES: usize = 16; // u128 = 16 bytes = 128 bits
pub const ASSET_ID_BYTES: usize = 32; // Fq field element = 32 bytes = 256 bits
pub const GENERATOR_BYTES: usize = 32; // Compressed curve point = 32 bytes = 256 bits
pub const KEY_BYTES: usize = 32; // Compressed curve point = 32 bytes = 256 bits
pub const ADDRESS_BYTES: usize = GENERATOR_BYTES + KEY_BYTES; // One address = 64 bytes
pub const TOTAL_PLAINTEXT_BYTES: usize =
    AMOUNT_BYTES + ASSET_ID_BYTES + ADDRESS_BYTES + ADDRESS_BYTES; // 176 bytes (self + counterparty)

/// Compliance ciphertext wire format constants.
/// These define the byte layout of the serialized ciphertext with tiered encryption.
///
/// Format:
/// - EPK: 32 bytes (ephemeral public key)
/// - detection_tag: 32 bytes (1 Fq - encrypted asset_id)
/// - encrypted_core: 96 bytes (3 Fq - encrypted amount + self_address, 80 bytes plaintext)
/// - encrypted_extension: 96 bytes (3 Fq - encrypted counterparty_address, 64 bytes plaintext)
///
/// NOTE: We use 31-byte chunks for encoding because Fq field order is ~2^252.
/// This means 80 bytes → ceil(80/31) = 3 Fq, and 64 bytes → ceil(64/31) = 3 Fq.
pub const EPK_BYTES: usize = 32; // Ephemeral public key (compressed curve point)
pub const DETECTION_TAG_BYTES: usize = 32; // 1 Fq element
pub const ENCRYPTED_CORE_BYTES: usize = 96; // 3 Fq elements (80 bytes plaintext → ceil(80/31) = 3)
pub const ENCRYPTED_EXTENSION_BYTES: usize = 96; // 3 Fq elements (64 bytes plaintext → ceil(64/31) = 3)
pub const CIPHERTEXT_PAYLOAD_BYTES: usize =
    DETECTION_TAG_BYTES + ENCRYPTED_CORE_BYTES + ENCRYPTED_EXTENSION_BYTES; // 224 bytes
pub const TOTAL_WIRE_BYTES: usize = EPK_BYTES + CIPHERTEXT_PAYLOAD_BYTES; // Total: 256 bytes
pub const NUM_CIPHERTEXT_FQS: usize = CIPHERTEXT_PAYLOAD_BYTES / 32; // Number of Fq elements: 7

// ============================================================================
// Compile-Time Consistency Checks
// ============================================================================

/// Compile-time assertion to ensure wire format constants are consistent.
const _: () = {
    assert!(
        TOTAL_PLAINTEXT_BYTES == 176,
        "TOTAL_PLAINTEXT_BYTES must be 176 (amount + asset + 2 addresses)"
    );
    assert!(TOTAL_WIRE_BYTES == 256, "TOTAL_WIRE_BYTES must be 256");
    assert!(EPK_BYTES == 32, "EPK_BYTES must be 32");
    assert!(DETECTION_TAG_BYTES == 32, "DETECTION_TAG_BYTES must be 32");
    assert!(
        ENCRYPTED_CORE_BYTES == 96,
        "ENCRYPTED_CORE_BYTES must be 96"
    );
    assert!(
        ENCRYPTED_EXTENSION_BYTES == 96,
        "ENCRYPTED_EXTENSION_BYTES must be 96"
    );
    assert!(
        CIPHERTEXT_PAYLOAD_BYTES == 224,
        "CIPHERTEXT_PAYLOAD_BYTES must be 224"
    );
    assert!(NUM_CIPHERTEXT_FQS == 7, "NUM_CIPHERTEXT_FQS must be 7");
    assert!(
        EPK_BYTES + CIPHERTEXT_PAYLOAD_BYTES == TOTAL_WIRE_BYTES,
        "EPK_BYTES + CIPHERTEXT_PAYLOAD_BYTES must equal TOTAL_WIRE_BYTES"
    );
};

/// The domain separator used to generate compliance leaf commitments.
pub(crate) static COMPLIANCE_LEAF_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.compliance.leaf").as_bytes())
});

/// A compliance leaf in the public on-chain registry for regulated assets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::ComplianceLeaf", into = "pb::ComplianceLeaf")]
pub struct ComplianceLeaf {
    /// The registered address for compliance.
    pub address: Address,
    /// The address compliance key (public key derived as MCK * B_d).
    pub key: AddressComplianceKey,
    /// The asset ID this compliance leaf applies to.
    pub asset_id: asset::Id,
}

impl ComplianceLeaf {
    /// Create a ComplianceLeaf, deriving ACK as `MCK * B_d` from the address diversifier.
    pub fn new(
        mck: &penumbra_sdk_keys::keys::MasterComplianceKey,
        address: Address,
        asset_id: asset::Id,
    ) -> Self {
        let diversifier = address.diversifier();
        let ack = mck.derive_address_key(diversifier);

        Self {
            address: address,
            key: ack,
            asset_id,
        }
    }

    /// Create the Poseidon commitment: hash_4(domain_sep, (g_d, transmission_key, ack, asset_id)).
    pub fn commit(&self) -> StateCommitment {
        // Decompose the address into field elements, matching Note::commit pattern
        let diversified_generator = self
            .address
            .diversified_generator()
            .vartime_compress_to_field();
        let transmission_key_s = Fq::from_bytes_checked(&self.address.transmission_key().0)
            .expect("transmission key is valid");

        // Convert AddressComplianceKey (curve point) to field element by compressing
        let ack_field = self.key.inner().vartime_compress_to_field();

        // Convert asset ID to field element
        let asset_id_field = self.asset_id.0;

        // Hash all components using poseidon377::hash_4
        let commit = poseidon377::hash_4(
            &COMPLIANCE_LEAF_DOMAIN_SEP,
            (
                diversified_generator,
                transmission_key_s,
                ack_field,
                asset_id_field,
            ),
        );

        StateCommitment(commit)
    }

    /// Export to JSON for off-chain sharing.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Import from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl DomainType for ComplianceLeaf {
    type Proto = pb::ComplianceLeaf;
}

impl TryFrom<pb::ComplianceLeaf> for ComplianceLeaf {
    type Error = anyhow::Error;

    fn try_from(value: pb::ComplianceLeaf) -> Result<Self, Self::Error> {
        Ok(ComplianceLeaf {
            address: value
                .address
                .ok_or_else(|| anyhow::anyhow!("missing address"))?
                .try_into()?,
            key: value
                .key
                .ok_or_else(|| anyhow::anyhow!("missing key"))?
                .try_into()?,
            asset_id: value
                .asset_id
                .ok_or_else(|| anyhow::anyhow!("missing asset_id"))?
                .try_into()?,
        })
    }
}

impl From<ComplianceLeaf> for pb::ComplianceLeaf {
    fn from(value: ComplianceLeaf) -> pb::ComplianceLeaf {
        pb::ComplianceLeaf {
            address: Some(value.address.into()),
            key: Some(value.key.into()),
            asset_id: Some(value.asset_id.into()),
        }
    }
}

/// Message to register an asset as regulated or non-regulated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::MsgRegisterAsset", into = "pb::MsgRegisterAsset")]
pub struct MsgRegisterAsset {
    /// The asset ID to register.
    pub asset_id: asset::Id,
    /// Whether this asset is regulated (requires compliance).
    pub is_regulated: bool,
}

impl DomainType for MsgRegisterAsset {
    type Proto = pb::MsgRegisterAsset;
}

impl TryFrom<pb::MsgRegisterAsset> for MsgRegisterAsset {
    type Error = anyhow::Error;

    fn try_from(value: pb::MsgRegisterAsset) -> Result<Self, Self::Error> {
        Ok(MsgRegisterAsset {
            asset_id: value
                .asset_id
                .ok_or_else(|| anyhow::anyhow!("missing asset_id"))?
                .try_into()?,
            is_regulated: value.is_regulated,
        })
    }
}

impl From<MsgRegisterAsset> for pb::MsgRegisterAsset {
    fn from(value: MsgRegisterAsset) -> pb::MsgRegisterAsset {
        pb::MsgRegisterAsset {
            asset_id: Some(value.asset_id.into()),
            is_regulated: value.is_regulated,
        }
    }
}

impl penumbra_sdk_txhash::EffectingData for MsgRegisterAsset {
    fn effect_hash(&self) -> penumbra_sdk_txhash::EffectHash {
        penumbra_sdk_txhash::EffectHash::from_proto_effecting_data::<pb::MsgRegisterAsset>(
            &self.clone().into(),
        )
    }
}

/// Message to register a user's compliance viewing key for a regulated asset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::MsgRegisterUser", into = "pb::MsgRegisterUser")]
pub struct MsgRegisterUser {
    /// The compliance leaf containing the user's registration information.
    pub leaf: ComplianceLeaf,
    /// Signature authorizing this registration.
    pub signature: Vec<u8>,
}

impl DomainType for MsgRegisterUser {
    type Proto = pb::MsgRegisterUser;
}

impl TryFrom<pb::MsgRegisterUser> for MsgRegisterUser {
    type Error = anyhow::Error;

    fn try_from(value: pb::MsgRegisterUser) -> Result<Self, Self::Error> {
        Ok(MsgRegisterUser {
            leaf: value
                .leaf
                .ok_or_else(|| anyhow::anyhow!("missing leaf"))?
                .try_into()?,
            signature: value.signature,
        })
    }
}

impl From<MsgRegisterUser> for pb::MsgRegisterUser {
    fn from(value: MsgRegisterUser) -> pb::MsgRegisterUser {
        pb::MsgRegisterUser {
            leaf: Some(value.leaf.into()),
            signature: value.signature,
        }
    }
}

impl penumbra_sdk_txhash::EffectingData for MsgRegisterUser {
    fn effect_hash(&self) -> penumbra_sdk_txhash::EffectHash {
        penumbra_sdk_txhash::EffectHash::from_proto_effecting_data::<pb::MsgRegisterUser>(
            &self.clone().into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_sdk_keys::keys::MasterComplianceKey;

    #[test]
    fn test_compliance_leaf_new_with_demo_mck() {
        let mut rng = rand::thread_rng();
        let demo_mck = MasterComplianceKey::demo();

        // Create dummy address and asset
        let address = Address::dummy(&mut rng);
        let asset_id = asset::Id(decaf377::Fq::from(100u64));

        // Create leaf using new() method
        let leaf = ComplianceLeaf::new(&demo_mck, address.clone(), asset_id);

        // Verify fields
        assert_eq!(leaf.address, address);
        assert_eq!(leaf.asset_id, asset_id);

        // Verify ACK was derived correctly (should be deterministic)
        let expected_ack = demo_mck.derive_address_key(address.diversifier());
        assert_eq!(leaf.key, expected_ack);
    }

    #[test]
    fn test_compliance_leaf_different_addresses_different_ack() {
        let mut rng = rand::thread_rng();
        let demo_mck = MasterComplianceKey::demo();
        let asset_id = asset::Id(decaf377::Fq::from(100u64));

        // Create two different addresses
        let address1 = Address::dummy(&mut rng);
        let address2 = Address::dummy(&mut rng);

        // Create leaves
        let leaf1 = ComplianceLeaf::new(&demo_mck, address1, asset_id);
        let leaf2 = ComplianceLeaf::new(&demo_mck, address2, asset_id);

        // ACKs should be different (privacy!)
        assert_ne!(
            leaf1.key, leaf2.key,
            "Different addresses must have different ACKs"
        );
    }

    #[test]
    fn test_compliance_leaf_same_address_different_assets() {
        let mut rng = rand::thread_rng();
        let demo_mck = MasterComplianceKey::demo();
        let address = Address::dummy(&mut rng);

        // Same address, different assets
        let usdc = asset::Id(decaf377::Fq::from(1u64));
        let dai = asset::Id(decaf377::Fq::from(2u64));

        let leaf_usdc = ComplianceLeaf::new(&demo_mck, address.clone(), usdc);
        let leaf_dai = ComplianceLeaf::new(&demo_mck, address.clone(), dai);

        // ACKs should be the same (derived from same address diversifier)
        assert_eq!(
            leaf_usdc.key, leaf_dai.key,
            "Same address should have same ACK across different assets"
        );

        // But asset IDs should differ
        assert_ne!(leaf_usdc.asset_id, leaf_dai.asset_id);
    }

    /// Test proto round-trip for ComplianceLeaf.
    /// This mimics what happens over gRPC: rpc.rs serializes to proto, client parses it back.
    /// This test would catch serialization bugs in the ACK encoding.
    #[test]
    fn test_compliance_leaf_proto_roundtrip() {
        use penumbra_sdk_keys::keys::AddressComplianceKey;

        let mut rng = rand::thread_rng();

        // Create a leaf with a specific ACK
        let wallet = Address::dummy(&mut rng);
        let ack =
            AddressComplianceKey::new(decaf377::Element::GENERATOR * decaf377::Fr::from(12345u64));
        let asset_id = asset::Id(decaf377::Fq::from(999u64));

        let original = ComplianceLeaf {
            address: wallet,
            key: ack,
            asset_id,
        };

        // Convert to proto (what rpc.rs does)
        let proto: pb::ComplianceLeaf = original.clone().into();

        // Verify proto has the ACK bytes (what goes over the wire)
        let key_proto = proto.key.as_ref().expect("key should be present");
        assert_eq!(key_proto.inner.len(), 32, "ACK should be 32 bytes");

        // Convert back from proto (what client_compliance_demo.rs does)
        let recovered: ComplianceLeaf = proto.try_into().expect("should parse");

        // CRITICAL: ACK must match after round-trip
        assert_eq!(
            original.key.inner(),
            recovered.key.inner(),
            "ACK must survive proto round-trip"
        );

        // All fields should match
        assert_eq!(original.address, recovered.address);
        assert_eq!(original.asset_id, recovered.asset_id);

        // Commitment must match (this is what the circuit uses)
        assert_eq!(
            original.commit().0,
            recovered.commit().0,
            "Commitment must match after round-trip"
        );
    }
}

/// A Merkle path in the Quad Merkle Tree (arity 4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(try_from = "pb::MerklePath", into = "pb::MerklePath")]
pub struct MerklePath {
    /// The layers of the Merkle path, from leaf to root.
    pub layers: Vec<MerklePathLayer>,
}

impl MerklePath {
    /// Create a MerklePath from the output of registry auth_path functions.
    ///
    /// Converts `Vec<[StateCommitment; 3]>` into the MerklePath format by
    /// serializing each StateCommitment to bytes.
    pub fn from_auth_path(auth_path: Vec<[StateCommitment; 3]>) -> Self {
        let layers = auth_path
            .into_iter()
            .map(|siblings_array| {
                let siblings = siblings_array
                    .iter()
                    .map(|commitment| commitment.0.to_bytes().to_vec())
                    .collect();
                MerklePathLayer { siblings }
            })
            .collect();
        MerklePath { layers }
    }
}

impl From<Vec<[StateCommitment; 3]>> for MerklePath {
    fn from(auth_path: Vec<[StateCommitment; 3]>) -> Self {
        Self::from_auth_path(auth_path)
    }
}

impl DomainType for MerklePath {
    type Proto = pb::MerklePath;
}

impl TryFrom<pb::MerklePath> for MerklePath {
    type Error = anyhow::Error;

    fn try_from(value: pb::MerklePath) -> Result<Self, Self::Error> {
        let layers = value
            .layers
            .into_iter()
            .map(|l| l.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MerklePath { layers })
    }
}

impl From<MerklePath> for pb::MerklePath {
    fn from(value: MerklePath) -> pb::MerklePath {
        pb::MerklePath {
            layers: value.layers.into_iter().map(|l| l.into()).collect(),
        }
    }
}

/// A single layer in the Quad Merkle Tree path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::MerklePathLayer", into = "pb::MerklePathLayer")]
pub struct MerklePathLayer {
    pub siblings: Vec<Vec<u8>>,
}

impl DomainType for MerklePathLayer {
    type Proto = pb::MerklePathLayer;
}

impl TryFrom<pb::MerklePathLayer> for MerklePathLayer {
    type Error = anyhow::Error;

    fn try_from(value: pb::MerklePathLayer) -> Result<Self, Self::Error> {
        Ok(MerklePathLayer {
            siblings: value.siblings,
        })
    }
}

impl From<MerklePathLayer> for pb::MerklePathLayer {
    fn from(value: MerklePathLayer) -> pb::MerklePathLayer {
        pb::MerklePathLayer {
            siblings: value.siblings,
        }
    }
}

/// Compliance ciphertext for a single party (sender or receiver).
///
/// This structure supports triple-layer encryption:
/// 1. Detection Tag - encrypted with detection key (for scanning)
/// 2. Core Data - encrypted with encryption key (asset ID, amount)
/// 3. Extension Data - encrypted with extension key (counterparty address)
#[derive(Clone, Debug)]
pub struct ComplianceCiphertext {
    /// The ephemeral public key R = r * B_d (diversified generator).
    /// Used to derive shared secrets with the daily public key.
    pub epk: decaf377::Element,

    /// Encrypted "Magic Byte" / Detection Tag (32 bytes: 16 bytes plaintext + 16 bytes AEAD auth tag).
    /// Decryptable by the issuer's daily detection key for scanning.
    pub detection_tag: [u8; 32],

    /// Encrypted core compliance data (AssetID + Amount).
    /// Decryptable by the issuer's daily encryption key.
    pub encrypted_core: Vec<u8>,

    /// Encrypted extension data (counterparty address).
    /// Decryptable by the issuer's daily extension key.
    pub encrypted_extension: Vec<u8>,
}

impl ComplianceCiphertext {
    /// Serialize the ephemeral public key to bytes.
    pub fn epk_bytes(&self) -> [u8; 32] {
        self.epk.vartime_compress().0
    }

    /// Create from an ephemeral public key point and encrypted data.
    pub fn new(
        epk: decaf377::Element,
        detection_tag: [u8; 32],
        encrypted_core: Vec<u8>,
        encrypted_extension: Vec<u8>,
    ) -> Self {
        Self {
            epk,
            detection_tag,
            encrypted_core,
            encrypted_extension,
        }
    }

    /// Serialize the entire compliance ciphertext to bytes.
    ///
    /// Format:
    /// - 32 bytes: ephemeral public key (compressed)
    /// - 32 bytes: detection tag (encrypted magic bytes)
    /// - Variable: encrypted_core (asset_id + amount + auth tag)
    /// - Variable: encrypted_extension (counterparty address + auth tag)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.epk_bytes());
        bytes.extend_from_slice(&self.detection_tag);
        bytes.extend_from_slice(&self.encrypted_core);
        bytes.extend_from_slice(&self.encrypted_extension);
        bytes
    }

    /// Deserialize a compliance ciphertext from bytes.
    ///
    /// Format (matching crypto.rs encryption output with tiered encryption):
    /// - EPK_BYTES (32): ephemeral public key (compressed)
    /// - DETECTION_TAG_BYTES (32): detection tag (encrypted asset_id)
    /// - ENCRYPTED_CORE_BYTES (96): encrypted amount + self_address
    /// - ENCRYPTED_EXTENSION_BYTES (96): encrypted counterparty_address
    ///
    /// Total: TOTAL_WIRE_BYTES (256 bytes)
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        use crate::{
            DETECTION_TAG_BYTES, ENCRYPTED_CORE_BYTES, ENCRYPTED_EXTENSION_BYTES, EPK_BYTES,
            TOTAL_WIRE_BYTES,
        };

        if bytes.len() != TOTAL_WIRE_BYTES {
            anyhow::bail!(
                "invalid ciphertext length: expected {} bytes, got {}",
                TOTAL_WIRE_BYTES,
                bytes.len()
            );
        }

        let mut offset = 0;

        // Parse ephemeral public key (32 bytes)
        let epk_bytes: [u8; EPK_BYTES] = bytes[offset..offset + EPK_BYTES].try_into()?;
        let epk = decaf377::Encoding(epk_bytes)
            .vartime_decompress()
            .map_err(|_| anyhow::anyhow!("failed to decompress ephemeral public key"))?;
        offset += EPK_BYTES;

        // Parse detection tag (32 bytes)
        let detection_tag: [u8; 32] = bytes[offset..offset + DETECTION_TAG_BYTES].try_into()?;
        offset += DETECTION_TAG_BYTES;

        // Parse encrypted core (96 bytes)
        let encrypted_core = bytes[offset..offset + ENCRYPTED_CORE_BYTES].to_vec();
        offset += ENCRYPTED_CORE_BYTES;

        // Parse encrypted extension (96 bytes)
        let encrypted_extension = bytes[offset..offset + ENCRYPTED_EXTENSION_BYTES].to_vec();

        Ok(Self {
            epk,
            detection_tag,
            encrypted_core,
            encrypted_extension,
        })
    }

    /// Convert to circuit public inputs format.
    ///
    /// Returns the EPK and ciphertext packed as field elements for use in ZK circuit public inputs.
    /// The ciphertext bytes (CIPHERTEXT_PAYLOAD_BYTES total: 32 detection_tag + 96 encrypted_core + 96 encrypted_extension)
    /// are packed into exactly NUM_CIPHERTEXT_FQS elements (32 bytes each = 7 Fq) to match the encryption format.
    pub fn to_circuit_public_inputs(&self) -> (decaf377::Element, Vec<decaf377::Fq>) {
        use crate::{CIPHERTEXT_PAYLOAD_BYTES, NUM_CIPHERTEXT_FQS};
        use decaf377::Fq;

        let epk = self.epk;

        // Reconstruct the ciphertext payload in the same order as encryption
        let mut ciphertext_bytes = Vec::with_capacity(CIPHERTEXT_PAYLOAD_BYTES);
        ciphertext_bytes.extend_from_slice(&self.detection_tag); // 32 bytes
        ciphertext_bytes.extend_from_slice(&self.encrypted_core); // 96 bytes
        ciphertext_bytes.extend_from_slice(&self.encrypted_extension); // 96 bytes

        debug_assert_eq!(ciphertext_bytes.len(), CIPHERTEXT_PAYLOAD_BYTES);

        // Convert to Fq elements (matching encryption output)
        let ciphertext_fqs: Vec<Fq> = ciphertext_bytes
            .chunks_exact(32)
            .map(|chunk| {
                let buf: [u8; 32] = chunk.try_into().expect("chunk should be exactly 32 bytes");
                Fq::from_le_bytes_mod_order(&buf)
            })
            .collect();

        debug_assert_eq!(ciphertext_fqs.len(), NUM_CIPHERTEXT_FQS);

        (epk, ciphertext_fqs)
    }
}

/// Complete compliance payload containing both sender and receiver ciphertexts.
///
/// This structure goes into the transaction body and allows the issuer to
/// decrypt both sides of a transaction using their daily master key.
#[derive(Clone, Debug)]
pub struct CompliancePayload {
    /// Compliance ciphertext for the sender's side of the transaction.
    pub sender_compliance: ComplianceCiphertext,

    /// Compliance ciphertext for the receiver's side of the transaction.
    pub receiver_compliance: ComplianceCiphertext,
}

impl CompliancePayload {
    /// Create a new compliance payload from sender and receiver ciphertexts.
    pub fn new(
        sender_compliance: ComplianceCiphertext,
        receiver_compliance: ComplianceCiphertext,
    ) -> Self {
        Self {
            sender_compliance,
            receiver_compliance,
        }
    }
}
