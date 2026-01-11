//! R1CS gadgets for compliance verification.
//!
//! This module implements zero-knowledge constraint gadgets for proving compliance
//! properties without revealing sensitive transaction details.

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::r1cs::{ElementVar, FqVar};
use decaf377::{Fq, Fr};
use once_cell::sync::Lazy;

use penumbra_sdk_keys::keys::AddressComplianceKey;

use crate::structs::{ComplianceLeaf, MerklePath};
use crate::tree::DEFAULT_DEPTH;
use crate::BLACK_HOLE_ACK;

/// Domain separator for detection key derivation.
static DETECTION_DOMAIN: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(
        blake2b_simd::blake2b(b"penumbra.compliance.keytype.detection").as_bytes(),
    )
});

/// Domain separator for core key derivation.
static CORE_DOMAIN: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(
        blake2b_simd::blake2b(b"penumbra.compliance.keytype.core").as_bytes(),
    )
});

/// Domain separator for extension key derivation.
static EXTENSION_DOMAIN: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(
        blake2b_simd::blake2b(b"penumbra.compliance.keytype.extension").as_bytes(),
    )
});

/// Verify a QuadTree Merkle authentication path in R1CS.
pub fn verify_quad_path(
    cs: ConstraintSystemRef<Fq>,
    leaf_var: FqVar,
    path: &MerklePath,
    position_var: FqVar,
) -> ark_relations::r1cs::Result<FqVar> {
    let mut current_hash = leaf_var;

    // Domain separator for hashing (consistent with QuadTree::hash_children)
    let zero_domain_sep = FqVar::new_constant(cs.clone(), Fq::from(0u64))?;

    // Convert position to little-endian bits
    let pos_bits = position_var.to_bits_le()?;

    // Process exactly DEFAULT_DEPTH layers to maintain fixed circuit size
    // (circuits must have fixed constraint count)
    for layer_idx in 0..(DEFAULT_DEPTH as usize) {
        let siblings = if layer_idx < path.layers.len() {
            // Use real siblings from witness path
            let layer = &path.layers[layer_idx];
            let mut vars = Vec::with_capacity(3);
            for sibling_bytes in &layer.siblings {
                let mut bytes_arr = [0u8; 32];
                bytes_arr.copy_from_slice(sibling_bytes);
                let sibling_fq = Fq::from_le_bytes_mod_order(&bytes_arr);
                vars.push(FqVar::new_witness(cs.clone(), || Ok(sibling_fq))?);
            }
            vars
        } else {
            // Use dummy zero siblings if path is shorter than FIXED_DEPTH
            vec![
                FqVar::new_witness(cs.clone(), || Ok(Fq::from(0u64)))?,
                FqVar::new_witness(cs.clone(), || Ok(Fq::from(0u64)))?,
                FqVar::new_witness(cs.clone(), || Ok(Fq::from(0u64)))?,
            ]
        };

        // Extract 2 bits for quad-tree navigation at this layer
        let bit_offset = layer_idx * 2;
        let bit_0 = if bit_offset < pos_bits.len() {
            pos_bits[bit_offset].clone()
        } else {
            Boolean::FALSE
        };
        let bit_1 = if bit_offset + 1 < pos_bits.len() {
            pos_bits[bit_offset + 1].clone()
        } else {
            Boolean::FALSE
        };

        // Construct the 4 children for quad-tree hashing based on position bits
        // Position bits determine which child contains the current hash
        let child_0 = bit_0
            .not()
            .and(&bit_1.not())?
            .select(&current_hash, &siblings[0])?;

        let child_1 = bit_0.and(&bit_1.not())?.select(
            &current_hash,
            &bit_0
                .not()
                .and(&bit_1.not())?
                .select(&siblings[0], &siblings[1])?,
        )?;

        let child_2 = bit_0.not().and(&bit_1)?.select(
            &current_hash,
            &bit_1.not().select(&siblings[1], &siblings[2])?,
        )?;

        let child_3 = bit_0.and(&bit_1)?.select(&current_hash, &siblings[2])?;

        let parent_hash = poseidon377::r1cs::hash_4(
            cs.clone(),
            &zero_domain_sep,
            (child_0, child_1, child_2, child_3),
        )?;

        current_hash = parent_hash;
    }

    Ok(current_hash)
}

// ============================================================================
// Compliance Key R1CS Gadgets
// ============================================================================

/// R1CS variable representing a Address Compliance Key (ACK).
///
/// The ACK is a public curve point `ACK = msk * B_d` stored in the compliance registry.
/// In the circuit, we work with its variable representation to prove key derivation.
#[derive(Clone)]
pub struct AddressComplianceKeyVar {
    /// The elliptic curve point representing the address compliance key.
    pub inner: ElementVar,
}

impl AddressComplianceKeyVar {
    /// Create a new AddressComplianceKeyVar from an ElementVar.
    pub fn new(inner: ElementVar) -> Self {
        Self { inner }
    }

    /// Derive the daily public key for a specific key type and date in-circuit.
    ///
    /// # Algorithm (Circuit Version)
    ///
    /// Given:
    /// - `self.inner`: ACK (point) = msk * B_d
    /// - `date`: Unix day index (field element)
    /// - `diversified_generator`: B_d (point)
    /// - `domain`: Key type domain separator (detection, core, or extension)
    ///
    /// Compute:
    /// 1. `T = Poseidon(domain, (date, 0))`
    /// 2. `Q = T * B_d` (scalar multiplication)
    /// 3. `PK_day = ACK + Q` (point addition)
    ///
    /// Returns: `PK_day` as ElementVar
    pub fn derive_daily_public_key_with_domain(
        &self,
        cs: ConstraintSystemRef<Fq>,
        date: &FqVar,
        diversified_generator: &ElementVar,
        domain: &Fq,
    ) -> Result<ElementVar, SynthesisError> {
        let domain_sep = FqVar::new_constant(cs.clone(), *domain)?;
        let zero = FqVar::new_constant(cs.clone(), Fq::from(0u64))?;
        let tweak_fq = poseidon377::r1cs::hash_2(cs.clone(), &domain_sep, (date.clone(), zero))?;

        let q_point = diversified_generator.scalar_mul_le(tweak_fq.to_bits_le()?.iter())?;
        let pk_day = self.inner.clone() + q_point;

        Ok(pk_day)
    }

    /// Derive all three daily public keys (detection, core, extension) for a date.
    ///
    /// Returns: (pk_detection, pk_core, pk_extension)
    pub fn derive_all_daily_public_keys(
        &self,
        cs: ConstraintSystemRef<Fq>,
        date: &FqVar,
        diversified_generator: &ElementVar,
    ) -> Result<(ElementVar, ElementVar, ElementVar), SynthesisError> {
        let pk_detection = self.derive_daily_public_key_with_domain(
            cs.clone(),
            date,
            diversified_generator,
            &DETECTION_DOMAIN,
        )?;
        let pk_core = self.derive_daily_public_key_with_domain(
            cs.clone(),
            date,
            diversified_generator,
            &CORE_DOMAIN,
        )?;
        let pk_extension = self.derive_daily_public_key_with_domain(
            cs,
            date,
            diversified_generator,
            &EXTENSION_DOMAIN,
        )?;

        Ok((pk_detection, pk_core, pk_extension))
    }

    /// Get the inner ElementVar.
    pub fn inner(&self) -> &ElementVar {
        &self.inner
    }
}

impl AllocVar<AddressComplianceKey, Fq> for AddressComplianceKeyVar {
    fn new_variable<T: std::borrow::Borrow<AddressComplianceKey>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let ack = f()?;
        let ack_point = ack.borrow().inner();

        // Allocate the curve point as an ElementVar
        //  Need to explicitly specify type parameter for Element
        let inner = ElementVar::new_variable(cs, || Ok(*ack_point), mode)?;

        Ok(Self { inner })
    }
}

// ============================================================================
// Compliance Leaf R1CS Gadgets
// ============================================================================

/// R1CS variable representing a Compliance Leaf.
///
/// This structure must match `structs::ComplianceLeaf` exactly so that
/// the commitment computed in-circuit matches the commitment in the registry.
pub struct ComplianceLeafVar {
    /// The wallet address (contains diversifier and transmission key).
    pub address: penumbra_sdk_keys::AddressVar,
    /// The address compliance key (public curve point).
    pub key: AddressComplianceKeyVar,
    /// The asset ID being regulated.
    pub asset_id: FqVar,
}

impl ComplianceLeafVar {
    /// Compute the Poseidon commitment of this leaf.
    ///
    /// This MUST match the logic in `structs::ComplianceLeaf::commit()` exactly!
    ///
    /// # Algorithm
    ///
    /// 1. Extract field elements from the address (diversified generator, transmission key, clue key)
    /// 2. Compress ACK curve point to field element
    /// 3. Hash all fields: `Hash(0, (ack_field, div_gen_x, div_gen_y, tx_key, clue_key, asset_id))`
    ///
    /// Returns: FqVar representing the leaf commitment
    pub fn commit(&self, cs: ConstraintSystemRef<Fq>) -> Result<FqVar, SynthesisError> {
        // Domain separator (must match structs::ComplianceLeaf::commit and COMPLIANCE_LEAF_DOMAIN_SEP)
        let domain_sep = FqVar::new_constant(
            cs.clone(),
            Fq::from_le_bytes_mod_order(
                blake2b_simd::blake2b(b"penumbra.compliance.leaf").as_bytes(),
            ),
        )?;

        // 1. Get diversified generator from address and compress to field element
        let div_gen = self.address.diversified_generator();
        let div_gen_compressed = div_gen.compress_to_field()?;

        // 2. Get transmission key from address and compress to field element
        let transmission_key = self.address.transmission_key();
        let transmission_key_s = transmission_key.compress_to_field()?;

        // 3. Compress ACK point to field element
        let ack_field = self.key.inner().compress_to_field()?;

        // 4. Hash: Poseidon_4(domain, (div_gen, tx_key, ack, asset_id))
        //    This MUST match the native structs::ComplianceLeaf::commit() logic exactly!
        //    Order: diversified_generator, transmission_key_s, ack_field, asset_id_field
        let commitment = poseidon377::r1cs::hash_4(
            cs,
            &domain_sep,
            (
                div_gen_compressed,
                transmission_key_s,
                ack_field,
                self.asset_id.clone(),
            ),
        )?;

        Ok(commitment)
    }
}

impl AllocVar<ComplianceLeaf, Fq> for ComplianceLeafVar {
    fn new_variable<T: std::borrow::Borrow<ComplianceLeaf>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let leaf = f()?;
        let leaf_ref = leaf.borrow();

        // Allocate each field
        let address = penumbra_sdk_keys::AddressVar::new_variable(
            cs.clone(),
            || Ok(&leaf_ref.address),
            mode,
        )?;
        let key = AddressComplianceKeyVar::new_variable(cs.clone(), || Ok(&leaf_ref.key), mode)?;

        // Convert asset::Id to Fq
        let asset_id_fq = leaf_ref.asset_id.0;
        let asset_id = FqVar::new_variable(cs, || Ok(asset_id_fq), mode)?;

        Ok(Self {
            address,
            key,
            asset_id,
        })
    }
}

// ============================================================================
// Compliance Encryption Verification Gadgets
// ============================================================================

/// Verify that compliance ciphertext was correctly generated using ECDH.
///
/// This gadget proves the following in zero-knowledge:
/// 1. The ephemeral public key `R` was correctly computed: `R = r * B_d`
/// 2. The shared secret `S` was correctly computed: `S = r * PK_day`
///
/// # Inputs
///
/// - `ephemeral_secret` (r): Private witness - the random scalar used for ECDH
/// - `pk_day`: Public/witness - the daily public key derived from ACK
/// - `diversified_generator` (B_d): Public input - from the recipient's address
/// - `published_epk` (R): Public input - from the ciphertext (ephemeral public key)
/// - `published_shared_secret` (S): Witness - the ECDH shared secret
///
/// # Constraints
///
/// 1. `published_epk == ephemeral_secret * diversified_generator`
/// 2. `published_shared_secret == ephemeral_secret * pk_day`
///
/// # Returns
///
/// `Ok(())` if all constraints are satisfied, error otherwise.
///
/// # Security
///
/// By proving these constraints in a SNARK, the prover demonstrates that:
/// - They know the ephemeral secret `r` (witness)
/// - The ciphertext was correctly encrypted to the derived daily key
/// - The encryption follows the "All-Seeing" key hierarchy
///
/// The verifier learns nothing about `r` or the shared secret, only that
/// the encryption was performed correctly.
pub fn verify_compliance_encryption(
    _cs: ConstraintSystemRef<Fq>,
    ephemeral_secret: &FqVar,             // r (witness)
    pk_day: &ElementVar,                  // PK_day = ACK + (T * B_d) (witness/public)
    diversified_generator: &ElementVar,   // B_d (public from address)
    published_epk: &ElementVar,           // R (public from ciphertext)
    published_shared_secret: &ElementVar, // S (witness)
) -> Result<(), SynthesisError> {
    // Convert ephemeral_secret to bits for scalar multiplication
    let r_bits = ephemeral_secret.to_bits_le()?;

    // Constraint 1: Verify R = r * B_d
    //
    // This ensures the published ephemeral public key was correctly computed
    // from the secret scalar and the recipient's diversified generator.
    let computed_epk = diversified_generator.scalar_mul_le(r_bits.iter())?;
    computed_epk.enforce_equal(published_epk)?;

    // Constraint 2: Verify S = r * PK_day
    //
    // This ensures the shared secret was correctly computed using ECDH.
    // The issuer can compute S' = dmk_t * R and decrypt the ciphertext.
    let computed_shared_secret = pk_day.scalar_mul_le(r_bits.iter())?;
    computed_shared_secret.enforce_equal(published_shared_secret)?;

    Ok(())
}

// ============================================================================
// Compliance Witness Structure
// ============================================================================

/// Witness data for compliance verification.
///
/// This struct groups all the private witness data needed to prove compliance
/// without revealing sensitive information. It's used by the `verify_compliance_integrity`
/// function to verify both asset registry and user registry inclusion.
#[derive(Clone)]
pub struct ComplianceWitness {
    /// Whether the asset is regulated (requires compliance checks).
    pub is_regulated: bool,
    /// Merkle path proving asset's regulatory status in the Asset Registry.
    pub asset_path: MerklePath,
    /// Position of the asset in the Asset Registry tree.
    pub asset_position: u64,
    /// Merkle path proving user authorization in the Compliance Registry.
    pub compliance_path: MerklePath,
    /// Position of the user in the Compliance Registry tree.
    pub compliance_position: u64,
    /// The compliance leaf containing address, ACK, and asset_id.
    pub user_leaf: ComplianceLeaf,
}

/// Circuit variable representation of compliance plaintext.
///
/// This struct wraps the circuit variables that represent the plaintext data
/// encrypted in the compliance ciphertext. It provides methods to serialize
/// the data to bits in a way that exactly matches the native byte serialization
/// used by the encryption function.
pub struct CompliancePlaintextVar {
    /// Amount being transacted (u128 value stored in FqVar).
    pub amount: FqVar,
    /// Asset ID being transacted.
    pub asset_id: FqVar,
    /// Self diversified generator (party who can decrypt this ciphertext).
    pub self_diversified_generator: ElementVar,
    /// Self transmission key.
    pub self_transmission_key: ElementVar,
    /// Counterparty diversified generator (the other party in the transaction).
    pub counterparty_diversified_generator: ElementVar,
    /// Counterparty transmission key.
    pub counterparty_transmission_key: ElementVar,
}

impl CompliancePlaintextVar {
    /// Get detection plaintext: asset_id (1 Fq element).
    ///
    /// This is encrypted directly as a single Fq - no chunking needed since
    /// asset_id is already an Fq field element.
    pub fn detection_plaintext(&self) -> FqVar {
        self.asset_id.clone()
    }

    /// Get core plaintext as Fq elements: amount + self address (3 Fq elements).
    ///
    /// Packs: amount (16 bytes) + self_div_gen (32 bytes) + self_trans_key (32 bytes)
    /// Total: 80 bytes → ceil(80/31) = 3 Fq elements using 31-byte chunks
    pub fn core_plaintext_fqs(&self) -> Result<Vec<FqVar>, SynthesisError> {
        use crate::structs::{AMOUNT_BYTES, GENERATOR_BYTES, KEY_BYTES};

        // Build core bytes: amount (16) + self_div_gen (32) + self_trans_key (32) = 80 bytes
        let total_bits = (AMOUNT_BYTES + GENERATOR_BYTES + KEY_BYTES) * 8; // 640 bits
        let mut bits = Vec::with_capacity(total_bits);

        // Amount: 128 bits (16 bytes)
        let mut amount_bits = self.amount.to_bits_le()?;
        amount_bits.truncate(AMOUNT_BYTES * 8);
        bits.append(&mut amount_bits);

        // Self Diversified Generator: 256 bits (32 bytes)
        let mut self_div_gen_bits = self
            .self_diversified_generator
            .compress_to_field()?
            .to_bits_le()?;
        self_div_gen_bits.resize(GENERATOR_BYTES * 8, Boolean::FALSE);
        bits.append(&mut self_div_gen_bits);

        // Self Transmission Key: 256 bits (32 bytes)
        let mut self_trans_key_bits = self
            .self_transmission_key
            .compress_to_field()?
            .to_bits_le()?;
        self_trans_key_bits.resize(KEY_BYTES * 8, Boolean::FALSE);
        bits.append(&mut self_trans_key_bits);

        // Pack into Fq elements using 31-byte (248-bit) chunks
        let chunk_size_bits = 31 * 8;
        let mut fqs = Vec::new();
        for chunk in bits.chunks(chunk_size_bits) {
            let mut padded_chunk = chunk.to_vec();
            padded_chunk.resize(256, Boolean::FALSE);
            let fq_var = Boolean::le_bits_to_fp_var(&padded_chunk)?;
            fqs.push(fq_var);
        }

        Ok(fqs)
    }

    /// Get extension plaintext as Fq elements: counterparty address (3 Fq elements).
    ///
    /// Packs: counterparty_div_gen (32 bytes) + counterparty_trans_key (32 bytes)
    /// Total: 64 bytes → ceil(64/31) = 3 Fq elements using 31-byte chunks
    pub fn extension_plaintext_fqs(&self) -> Result<Vec<FqVar>, SynthesisError> {
        use crate::structs::{GENERATOR_BYTES, KEY_BYTES};

        // Build extension bytes: counterparty_div_gen (32) + counterparty_trans_key (32) = 64 bytes
        let total_bits = (GENERATOR_BYTES + KEY_BYTES) * 8; // 512 bits
        let mut bits = Vec::with_capacity(total_bits);

        // Counterparty Diversified Generator: 256 bits (32 bytes)
        let mut counterparty_div_gen_bits = self
            .counterparty_diversified_generator
            .compress_to_field()?
            .to_bits_le()?;
        counterparty_div_gen_bits.resize(GENERATOR_BYTES * 8, Boolean::FALSE);
        bits.append(&mut counterparty_div_gen_bits);

        // Counterparty Transmission Key: 256 bits (32 bytes)
        let mut counterparty_trans_key_bits = self
            .counterparty_transmission_key
            .compress_to_field()?
            .to_bits_le()?;
        counterparty_trans_key_bits.resize(KEY_BYTES * 8, Boolean::FALSE);
        bits.append(&mut counterparty_trans_key_bits);

        // Pack into Fq elements using 31-byte (248-bit) chunks
        let chunk_size_bits = 31 * 8;
        let mut fqs = Vec::new();
        for chunk in bits.chunks(chunk_size_bits) {
            let mut padded_chunk = chunk.to_vec();
            padded_chunk.resize(256, Boolean::FALSE);
            let fq_var = Boolean::le_bits_to_fp_var(&padded_chunk)?;
            fqs.push(fq_var);
        }

        Ok(fqs)
    }
}

// ============================================================================
// Unified Compliance Integrity Verification
// ============================================================================

/// Verify compliance integrity for a transaction action.
///
/// This function consolidates all compliance verification logic into a single
/// reusable gadget that can be called from different proof circuits. It verifies:
/// 1. Asset registry inclusion (proving regulatory status)
/// 2. User registry inclusion (proving authorization)
/// 3. Ciphertext integrity (proving correct encryption to ACK-derived key)
///
/// # Unified Path Logic
///
/// For **Regulated** assets:
/// - Verifies user exists in compliance registry
/// - Encrypts compliance data to user's ACK-derived daily key
/// - Issuer can decrypt with daily master key
///
/// For **Unregulated** assets:
/// - Accepts dummy user data (conditional enforcement skipped)
/// - Encrypts to BLACK_HOLE_ACK (nothing-up-my-sleeve key)
/// - Makes transaction unlinkable to specific users
///
/// Both paths use identical constraint counts for perfect indistinguishability.
///
/// # Arguments
///
/// * `cs` - Constraint system reference
/// * `asset_anchor` - Asset registry Merkle root (public input)
/// * `compliance_anchor` - User registry Merkle root (public input)
/// * `target_date` - Unix day index for key derivation (public input)
/// * `compliance_epk` - Ephemeral public key from compliance ciphertext (public input)
/// * `compliance_ciphertext` - Encrypted compliance data as field elements (public input)
/// * `note_address` - Address variable from the note (binds to user_leaf)
/// * `note_asset_id` - Asset ID field variable from the note
/// * `note_amount` - Amount field variable from the note
/// * `note_diversified_generator` - Diversified generator from note address (self address)
/// * `note_transmission_key` - Transmission key from note address (self address)
/// * `counterparty_diversified_generator` - Diversified generator from counterparty address
/// * `counterparty_transmission_key` - Transmission key from counterparty address
/// * `compliance_ephemeral_secret` - Ephemeral secret for compliance encryption (witness)
/// * `witness` - Grouped compliance witness data
///
/// # Returns
///
/// `Ok(())` if all constraints are satisfied, error otherwise.
pub fn verify_compliance_integrity(
    cs: ConstraintSystemRef<Fq>,
    // Public Inputs
    asset_anchor: FqVar,
    compliance_anchor: FqVar,
    target_date: FqVar,
    compliance_epk: ElementVar,
    compliance_ciphertext: Vec<FqVar>,
    // Note data (to ensure binding)
    note_asset_id: FqVar,
    note_amount: FqVar,
    note_diversified_generator: ElementVar,
    note_transmission_key: ElementVar,
    counterparty_diversified_generator: ElementVar,
    counterparty_transmission_key: ElementVar,
    // Witness data
    compliance_ephemeral_secret: Fr,
    witness: ComplianceWitness,
) -> Result<(), SynthesisError> {
    let (is_regulated, asset_position, compliance_position, user_leaf_var) =
        allocate_compliance_witnesses(cs.clone(), &witness)?;

    verify_asset_registry_path(
        cs.clone(),
        note_asset_id.clone(),
        &is_regulated,
        &witness.asset_path,
        &asset_position,
        &asset_anchor,
    )?;

    verify_compliance_registry_path(
        cs.clone(),
        &user_leaf_var,
        &is_regulated,
        &witness.compliance_path,
        &compliance_position,
        &compliance_anchor,
    )?;

    let target_ack = select_compliance_key(cs.clone(), &is_regulated, &user_leaf_var)?;

    // Derive all 3 daily public keys (detection, core, extension)
    let (pk_detection, pk_core, pk_extension) = target_ack.derive_all_daily_public_keys(
        cs.clone(),
        &target_date,
        &note_diversified_generator,
    )?;

    // Derive all 3 shared secrets using the same ephemeral secret
    let (ss_detection, ss_core, ss_extension) = derive_all_shared_secrets(
        cs.clone(),
        compliance_ephemeral_secret,
        &pk_detection,
        &pk_core,
        &pk_extension,
        &note_diversified_generator,
        &compliance_epk,
    )?;

    verify_tiered_poseidon_encryption(
        cs.clone(),
        &ss_detection,
        &ss_core,
        &ss_extension,
        &compliance_epk,
        note_amount,
        note_asset_id,
        note_diversified_generator,
        note_transmission_key,
        counterparty_diversified_generator,
        counterparty_transmission_key,
        &compliance_ciphertext,
    )?;

    Ok(())
}

/// Allocate witness variables for compliance verification.
fn allocate_compliance_witnesses(
    cs: ConstraintSystemRef<Fq>,
    witness: &ComplianceWitness,
) -> Result<(Boolean<Fq>, FqVar, FqVar, ComplianceLeafVar), SynthesisError> {
    let is_regulated = Boolean::new_witness(cs.clone(), || Ok(witness.is_regulated))?;
    let asset_position = FqVar::new_witness(cs.clone(), || Ok(Fq::from(witness.asset_position)))?;
    let compliance_position =
        FqVar::new_witness(cs.clone(), || Ok(Fq::from(witness.compliance_position)))?;
    let user_leaf_var =
        ComplianceLeafVar::new_variable(cs, || Ok(&witness.user_leaf), AllocationMode::Witness)?;

    Ok((
        is_regulated,
        asset_position,
        compliance_position,
        user_leaf_var,
    ))
}

/// Verify asset registry Merkle path.
fn verify_asset_registry_path(
    cs: ConstraintSystemRef<Fq>,
    note_asset_id: FqVar,
    is_regulated: &Boolean<Fq>,
    asset_path: &MerklePath,
    asset_position: &FqVar,
    asset_anchor: &FqVar,
) -> Result<(), SynthesisError> {
    let zero_domain_sep = FqVar::new_constant(cs.clone(), Fq::from(0u64))?;
    let status_fq = is_regulated.select(&FqVar::one(), &FqVar::zero())?;

    let asset_leaf =
        poseidon377::r1cs::hash_2(cs.clone(), &zero_domain_sep, (note_asset_id, status_fq))?;

    let calculated_asset_root =
        verify_quad_path(cs.clone(), asset_leaf, asset_path, asset_position.clone())?;

    // Conditionally enforce: only if anchor is non-zero
    let is_real_anchor = asset_anchor.is_neq(&FqVar::zero())?;
    calculated_asset_root.conditional_enforce_equal(asset_anchor, &is_real_anchor)?;

    Ok(())
}

/// Verify compliance registry Merkle path.
fn verify_compliance_registry_path(
    cs: ConstraintSystemRef<Fq>,
    user_leaf_var: &ComplianceLeafVar,
    is_regulated: &Boolean<Fq>,
    compliance_path: &MerklePath,
    compliance_position: &FqVar,
    compliance_anchor: &FqVar,
) -> Result<(), SynthesisError> {
    let user_leaf_commitment = user_leaf_var.commit(cs.clone())?;

    let calculated_user_root = verify_quad_path(
        cs.clone(),
        user_leaf_commitment,
        compliance_path,
        compliance_position.clone(),
    )?;

    // Conditional enforcement: only if regulated AND anchor is real
    let is_dummy_anchor = compliance_anchor.is_eq(&FqVar::zero())?;
    let should_enforce = is_regulated.and(&is_dummy_anchor.not())?;

    calculated_user_root.conditional_enforce_equal(compliance_anchor, &should_enforce)?;

    Ok(())
}

/// Select compliance key based on regulation status.
fn select_compliance_key(
    cs: ConstraintSystemRef<Fq>,
    is_regulated: &Boolean<Fq>,
    user_leaf_var: &ComplianceLeafVar,
) -> Result<AddressComplianceKeyVar, SynthesisError> {
    let black_hole_ack_var = ElementVar::new_constant(cs, *BLACK_HOLE_ACK)?;
    let black_hole_ack_wrapped = AddressComplianceKeyVar::new(black_hole_ack_var);

    // Select: regulated -> user's ACK, unregulated -> BLACK_HOLE
    let target_ack_inner =
        is_regulated.select(user_leaf_var.key.inner(), black_hole_ack_wrapped.inner())?;

    Ok(AddressComplianceKeyVar::new(target_ack_inner))
}

/// Derive all 3 shared secrets from the ephemeral secret and 3 daily public keys.
///
/// Also verifies the EPK constraint: EPK = r * B_d
fn derive_all_shared_secrets(
    cs: ConstraintSystemRef<Fq>,
    compliance_ephemeral_secret: Fr,
    pk_detection: &ElementVar,
    pk_core: &ElementVar,
    pk_extension: &ElementVar,
    note_diversified_generator: &ElementVar,
    compliance_epk: &ElementVar,
) -> Result<(ElementVar, ElementVar, ElementVar), SynthesisError> {
    use ark_ff::{BigInteger, PrimeField};

    // Convert ephemeral secret to bits for scalar multiplication
    let esk_bigint = compliance_ephemeral_secret.into_bigint();
    let esk_bits: Vec<bool> = (0..Fr::MODULUS_BIT_SIZE)
        .map(|i| esk_bigint.get_bit(i as usize))
        .collect();
    let esk_bits_vars: Vec<Boolean<Fq>> = esk_bits
        .iter()
        .map(|bit| Boolean::new_witness(cs.clone(), || Ok(*bit)))
        .collect::<Result<Vec<_>, _>>()?;

    // Verify EPK = r * B_d (only need to verify this once)
    let computed_epk = note_diversified_generator.scalar_mul_le(esk_bits_vars.iter())?;
    computed_epk.enforce_equal(compliance_epk)?;

    // Compute all 3 shared secrets: S_type = r * PK_type
    let ss_detection = pk_detection.scalar_mul_le(esk_bits_vars.iter())?;
    let ss_core = pk_core.scalar_mul_le(esk_bits_vars.iter())?;
    let ss_extension = pk_extension.scalar_mul_le(esk_bits_vars.iter())?;

    Ok((ss_detection, ss_core, ss_extension))
}

/// Verify tiered Poseidon stream cipher encryption with 3 different keys.
///
/// Each ciphertext segment is encrypted with its own shared secret:
/// - detection_tag: encrypted with ss_detection (1 Fq)
/// - encrypted_core: encrypted with ss_core (3 Fqs)
/// - encrypted_extension: encrypted with ss_extension (3 Fqs)
///
/// Total: 7 Fq elements in compliance_ciphertext
fn verify_tiered_poseidon_encryption(
    cs: ConstraintSystemRef<Fq>,
    ss_detection: &ElementVar,
    ss_core: &ElementVar,
    ss_extension: &ElementVar,
    compliance_epk: &ElementVar,
    note_amount: FqVar,
    note_asset_id: FqVar,
    self_diversified_generator: ElementVar,
    self_transmission_key: ElementVar,
    counterparty_diversified_generator: ElementVar,
    counterparty_transmission_key: ElementVar,
    compliance_ciphertext: &[FqVar],
) -> Result<(), SynthesisError> {
    // Expected ciphertext layout: [detection: 1] [core: 3] [extension: 3] = 7 Fqs
    if compliance_ciphertext.len() != 7 {
        return Err(SynthesisError::Unsatisfiable);
    }

    let epk_fq = compliance_epk.compress_to_field()?;
    let domain_sep =
        FqVar::new_constant(cs.clone(), *crate::crypto::COMPLIANCE_STREAM_CIPHER_DOMAIN)?;

    // Build plaintext structure
    let plaintext_var = CompliancePlaintextVar {
        amount: note_amount,
        asset_id: note_asset_id,
        self_diversified_generator,
        self_transmission_key,
        counterparty_diversified_generator,
        counterparty_transmission_key,
    };

    // === DETECTION: 1 Fq element (asset_id) ===
    let seed_detection = poseidon377::r1cs::hash_2(
        cs.clone(),
        &domain_sep,
        (ss_detection.compress_to_field()?, epk_fq.clone()),
    )?;

    let detection_plaintext = plaintext_var.detection_plaintext();
    let detection_counter = FqVar::new_constant(cs.clone(), Fq::from(0u64))?;
    let detection_keystream = poseidon377::r1cs::hash_2(
        cs.clone(),
        &seed_detection,
        (detection_counter, seed_detection.clone()),
    )?;
    let computed_detection = &detection_plaintext + &detection_keystream;
    computed_detection.enforce_equal(&compliance_ciphertext[0])?;

    // === CORE: 3 Fq elements (amount + self address) ===
    let seed_core = poseidon377::r1cs::hash_2(
        cs.clone(),
        &domain_sep,
        (ss_core.compress_to_field()?, epk_fq.clone()),
    )?;

    let core_plaintexts = plaintext_var.core_plaintext_fqs()?;
    for (i, plain_var) in core_plaintexts.iter().enumerate() {
        let counter = FqVar::new_constant(cs.clone(), Fq::from(i as u64))?;
        let keystream =
            poseidon377::r1cs::hash_2(cs.clone(), &seed_core, (counter, seed_core.clone()))?;
        let computed_cipher = plain_var + &keystream;
        computed_cipher.enforce_equal(&compliance_ciphertext[1 + i])?;
    }

    // === EXTENSION: 3 Fq elements (counterparty address) ===
    let seed_extension = poseidon377::r1cs::hash_2(
        cs.clone(),
        &domain_sep,
        (ss_extension.compress_to_field()?, epk_fq),
    )?;

    let extension_plaintexts = plaintext_var.extension_plaintext_fqs()?;
    for (i, plain_var) in extension_plaintexts.iter().enumerate() {
        let counter = FqVar::new_constant(cs.clone(), Fq::from(i as u64))?;
        let keystream = poseidon377::r1cs::hash_2(
            cs.clone(),
            &seed_extension,
            (counter, seed_extension.clone()),
        )?;
        let computed_cipher = plain_var + &keystream;
        computed_cipher.enforce_equal(&compliance_ciphertext[4 + i])?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;
    use decaf377::{Element, Fr};
    use penumbra_sdk_asset::asset;
    use penumbra_sdk_keys::keys::{Diversifier, MasterComplianceKey};
    use penumbra_sdk_keys::Address;
    use rand_core::OsRng;

    /// Test: AddressComplianceKeyVar allocation and daily key derivation
    #[test]
    fn test_wallet_compliance_key_var_derivation() {
        let mut rng = OsRng;
        let cs = ConstraintSystem::<Fq>::new_ref();

        // Create a master key and derive a wallet key
        let msk = Fr::rand(&mut rng);
        let master_key = MasterComplianceKey::new(msk);
        let diversifier = Diversifier([7u8; 16]);
        let ack = master_key.derive_address_key(&diversifier);

        // Allocate ACK in the circuit
        let ack_var =
            AddressComplianceKeyVar::new_variable(cs.clone(), || Ok(&ack), AllocationMode::Witness)
                .expect("allocation should succeed");

        // Allocate date
        let date = 19000u64;
        let date_var = FqVar::new_witness(cs.clone(), || Ok(Fq::from(date)))
            .expect("date allocation should succeed");

        // Get diversified generator
        let div_gen = diversifier.diversified_generator();
        let div_gen_var = ElementVar::new_witness(cs.clone(), || Ok(div_gen))
            .expect("div gen allocation should succeed");

        // Derive daily public key in circuit using Detection domain
        let pk_day_var = ack_var
            .derive_daily_public_key_with_domain(
                cs.clone(),
                &date_var,
                &div_gen_var,
                &DETECTION_DOMAIN,
            )
            .expect("derivation should succeed");

        // Verify the circuit is satisfied
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");

        // Compute the expected value natively using Detection key type
        let pk_day_native = ack.derive_daily_public_key(
            penumbra_sdk_keys::keys::KeyType::Detection,
            date,
            &diversifier,
        );

        // Get the value from the circuit variable
        let pk_day_circuit = pk_day_var.value().expect("should have value");

        // They should match!
        assert_eq!(
            pk_day_circuit, pk_day_native,
            "Circuit and native derivation should match"
        );
    }

    /// Test: ComplianceLeafVar commitment matches native implementation
    #[test]
    fn test_compliance_leaf_var_commit() {
        let mut rng = OsRng;
        let cs = ConstraintSystem::<Fq>::new_ref();

        // Create a compliance leaf
        let msk = Fr::rand(&mut rng);
        let master_key = MasterComplianceKey::new(msk);
        let diversifier = Diversifier([1u8; 16]);
        let ack = master_key.derive_address_key(&diversifier);

        // Create a test address
        let scalar = Fr::rand(&mut rng);
        let point = Element::GENERATOR * scalar;
        let pk_d = decaf377_ka::Public(point.vartime_compress().0);
        let mut ck_d_bytes = [0u8; 32];
        use rand_core::RngCore;
        rng.fill_bytes(&mut ck_d_bytes);
        let ck_d = decaf377_fmd::ClueKey(ck_d_bytes);
        let address = Address::from_components(diversifier, pk_d, ck_d).expect("valid address");

        let leaf = ComplianceLeaf {
            address: address,
            key: ack,
            asset_id: asset::Id(Fq::from(42u64)),
        };

        // Compute native commitment
        let native_commitment = leaf.commit();

        // Allocate leaf in circuit
        let leaf_var =
            ComplianceLeafVar::new_variable(cs.clone(), || Ok(&leaf), AllocationMode::Witness)
                .expect("leaf allocation should succeed");

        // Compute commitment in circuit
        let circuit_commitment = leaf_var
            .commit(cs.clone())
            .expect("commitment should succeed");

        // Verify circuit is satisfied
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");

        // Get the value from the circuit
        let circuit_commitment_value = circuit_commitment.value().expect("should have value");

        // They should match!
        assert_eq!(
            circuit_commitment_value, native_commitment.0,
            "Circuit and native commitments should match"
        );
    }

    /// Test: verify_compliance_encryption gadget
    #[test]
    fn test_verify_compliance_encryption() {
        let mut rng = OsRng;
        let cs = ConstraintSystem::<Fq>::new_ref();

        // Setup: Create keys and derive daily public key
        let msk = Fr::rand(&mut rng);
        let master_key = MasterComplianceKey::new(msk);
        let diversifier = Diversifier([3u8; 16]);
        let ack = master_key.derive_address_key(&diversifier);

        let date = 19000u64;
        // Note: Using Detection key type for test - circuit will be updated to support all three
        let pk_day = ack.derive_daily_public_key(
            penumbra_sdk_keys::keys::KeyType::Detection,
            date,
            &diversifier,
        );

        // Generate ephemeral secret and compute ECDH
        let ephemeral_secret = Fr::rand(&mut rng);
        let div_gen = diversifier.diversified_generator();
        let epk = div_gen * ephemeral_secret; // R = r * B_d
        let shared_secret = pk_day * ephemeral_secret; // S = r * PK_day

        // Allocate everything in the circuit
        let ephemeral_secret_var = FqVar::new_witness(cs.clone(), || {
            Ok(Fq::from_le_bytes_mod_order(&ephemeral_secret.to_bytes()))
        })
        .expect("ephemeral secret allocation");

        let pk_day_var =
            ElementVar::new_input(cs.clone(), || Ok(pk_day)).expect("pk_day allocation");

        let div_gen_var =
            ElementVar::new_input(cs.clone(), || Ok(div_gen)).expect("div gen allocation");

        let epk_var = ElementVar::new_input(cs.clone(), || Ok(epk)).expect("epk allocation");

        let shared_secret_var = ElementVar::new_witness(cs.clone(), || Ok(shared_secret))
            .expect("shared secret allocation");

        // Run the verification gadget
        verify_compliance_encryption(
            cs.clone(),
            &ephemeral_secret_var,
            &pk_day_var,
            &div_gen_var,
            &epk_var,
            &shared_secret_var,
        )
        .expect("verification should succeed");

        // Verify the circuit is satisfied
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");
    }
}
