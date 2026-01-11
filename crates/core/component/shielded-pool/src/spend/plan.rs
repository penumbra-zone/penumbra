use ark_groth16::ProvingKey;
use decaf377::{Bls12_377, Fq, Fr};
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_sdk_asset::{Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_compliance::MerklePath;
use penumbra_sdk_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{Body, Spend, SpendProof};
use crate::{Backref, Note, Rseed, SpendProofPrivate, SpendProofPublic};

/// A planned [`Spend`](Spend).
///
/// # Compliance Data Architecture Decision
///
/// Compliance-related fields (paths, positions, anchors) are stored directly in the plan
/// rather than in a separate WitnessData-like structure. This differs from how SCT Merkle
/// proofs are handled (via WitnessData at build time), but is intentional because:
///
/// 1. **Compliance data is not original Penumbra transaction data** - we have freedom to
///    design it however makes sense for our use case.
///
/// 2. **Plans should be self-contained and portable** - a serialized plan should be buildable
///    without additional chain queries (important for hardware wallets, offline signing).
///
/// 3. **Consistency** - we already have `compliance_path`, `compliance_anchor`, `asset_anchor`
///    in the plan; adding `asset_path`, `asset_position`, `compliance_position` is consistent.
///
/// 4. **Staleness is manageable** - the compliance registry changes infrequently (user/asset
///    registrations are stable), so stale data causing failures is unlikely.
///
/// 5. **Simpler code path** - no need to pass additional data structures at build time.
///
/// If staleness becomes problematic in the future, we can add historical compliance anchor
/// validation on chain (like SCT anchors) without changing client architecture.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SpendPlan", into = "pb::SpendPlan")]
pub struct SpendPlan {
    pub note: Note,
    pub position: tct::Position,
    pub randomizer: Fr,
    pub value_blinding: Fr,
    pub proof_blinding_r: Fq,
    pub proof_blinding_s: Fq,
    /// Compliance Merkle path for proving user is in the compliance registry
    pub compliance_path: MerklePath,
    /// Target timestamp for compliance key derivation (Unix timestamp in seconds)
    pub target_timestamp: u64,
    /// Precomputed compliance ciphertext (or placeholder if not yet generated)
    #[serde(skip)]
    pub compliance_ciphertext: Vec<u8>,
    /// Compliance leaf for ZK proof
    #[serde(skip)]
    pub compliance_leaf: Option<penumbra_sdk_compliance::ComplianceLeaf>,
    /// Ephemeral secret used in compliance ciphertext encryption (needed by circuit)
    #[serde(skip)]
    pub compliance_ephemeral_secret: Option<Fr>,
    /// Whether the asset is regulated (requires compliance)
    #[serde(skip)]
    pub is_regulated: bool,
    /// Counterparty address (the recipient of this spend)
    #[serde(skip)]
    pub counterparty_address: Option<penumbra_sdk_keys::Address>,
    /// Counterparty compliance leaf (the recipient's leaf, for binding)
    #[serde(skip)]
    pub counterparty_leaf: Option<penumbra_sdk_compliance::ComplianceLeaf>,
    /// Shared transaction blinding nonce (same for spend and output in one transaction)
    #[serde(skip)]
    pub tx_blinding_nonce: Fr,
    /// Compliance anchor (user tree root) for proof generation
    #[serde(skip)]
    pub compliance_anchor: tct::StateCommitment,
    /// Asset anchor (asset tree root) for proof generation
    #[serde(skip)]
    pub asset_anchor: tct::StateCommitment,
    /// Asset Merkle path for proving asset is in the asset registry
    #[serde(skip)]
    pub asset_path: MerklePath,
    /// Position of the asset in the asset registry tree
    #[serde(skip)]
    pub asset_position: u64,
    /// Position of the user's compliance leaf in the compliance tree
    #[serde(skip)]
    pub compliance_position: u64,
}

impl SpendPlan {
    /// Set compliance details for this spend plan.
    ///
    /// This should be called after constructing the plan to properly encrypt
    /// the compliance ciphertext using ACK.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator
    /// * `sender_ack` - The sender's Wallet Compliance Key (from their registry entry)
    /// * `sender_address` - The sender's address
    /// * `is_regulated` - Whether the asset is regulated
    /// * `recipient_address` - The recipient's address (counterparty)
    /// * `recipient_leaf` - The recipient's compliance leaf (from registry)
    pub fn set_compliance_details(
        &mut self,
        rng: &mut (impl rand_core::RngCore + rand_core::CryptoRng),
        sender_ack: &penumbra_sdk_keys::keys::AddressComplianceKey,
        sender_address: &Address,
        is_regulated: bool,
        recipient_address: &Address,
        recipient_leaf: penumbra_sdk_compliance::ComplianceLeaf,
    ) -> anyhow::Result<()> {
        let date = crate::timestamp_to_day_index(self.target_timestamp);

        let compliance_data = crate::generate_compliance_details(
            rng,
            is_regulated,
            sender_ack,
            sender_address,
            date,
            self.note.asset_id(),
            self.note.amount(),
            recipient_address.clone(),
        )?;

        self.compliance_ciphertext = compliance_data.ciphertext;
        self.compliance_leaf = Some(compliance_data.leaf);
        self.compliance_ephemeral_secret = Some(compliance_data.ephemeral_secret);
        self.is_regulated = is_regulated;
        self.counterparty_address = Some(recipient_address.clone());
        self.counterparty_leaf = Some(recipient_leaf);

        Ok(())
    }

    /// Create a new [`SpendPlan`] that spends the given `position`ed `note`.
    ///
    /// Uses the current system time as the target timestamp.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        note: Note,
        position: tct::Position,
    ) -> SpendPlan {
        let target_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_secs();

        // Generate valid compliance data using BLACK_HOLE_ACK for unregulated assets.
        // This ensures circuit constraints are satisfied even without explicit compliance setup.
        let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(
            *penumbra_sdk_compliance::BLACK_HOLE_ACK,
        );
        let address = note.address();
        let date = target_timestamp / 86400; // Convert timestamp to day index

        let (compliance_ciphertext_obj, ephemeral_secret) =
            penumbra_sdk_compliance::crypto::encrypt_compliance_details(
                &mut *rng,
                &black_hole_ack,
                &address,
                date,
                note.asset_id(),
                note.amount(),
                address.clone(), // Use same address as counterparty for default
            )
            .expect("can encrypt compliance details");

        let compliance_ciphertext = compliance_ciphertext_obj.to_bytes();

        let compliance_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: address.clone(),
            key: black_hole_ack.clone(),
            asset_id: note.asset_id(),
        };

        // Create 16-layer dummy Merkle path
        let dummy_path = MerklePath {
            layers: (0..16)
                .map(|_| penumbra_sdk_compliance::structs::MerklePathLayer {
                    siblings: vec![vec![0u8; 32], vec![0u8; 32], vec![0u8; 32]],
                })
                .collect(),
        };

        SpendPlan {
            note,
            position,
            randomizer: Fr::rand(rng),
            value_blinding: Fr::rand(rng),
            proof_blinding_r: Fq::rand(rng),
            proof_blinding_s: Fq::rand(rng),
            compliance_path: dummy_path.clone(),
            target_timestamp,
            compliance_ciphertext,
            compliance_leaf: Some(compliance_leaf.clone()),
            compliance_ephemeral_secret: Some(ephemeral_secret),
            is_regulated: false,
            counterparty_address: None,
            counterparty_leaf: Some(compliance_leaf), // Use same leaf as counterparty for default
            tx_blinding_nonce: Fr::rand(rng),
            compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
            asset_anchor: tct::StateCommitment(Fq::from(0u64)),
            asset_path: dummy_path,
            asset_position: 0,
            compliance_position: 0,
        }
    }

    /// Create a dummy [`SpendPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(rng: &mut R, fvk: &FullViewingKey) -> SpendPlan {
        // A valid address we can spend; since the note is hidden, we can just pick the default.
        let dummy_address = fvk.payment_address(AddressIndex::default()).0;
        let rseed = Rseed::generate(rng);
        let dummy_note = Note::from_parts(
            dummy_address,
            Value {
                amount: 0u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
            rseed,
        )
        .expect("dummy note is valid");

        Self::new(rng, dummy_note, 0u64.into())
    }

    /// Convenience method to construct the [`Spend`] described by this [`SpendPlan`].
    pub fn spend(
        &self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
        anchor: tct::Root,
        pk: &ProvingKey<Bls12_377>,
        compliance_keys: Option<(decaf377::Element, decaf377::Element)>,
    ) -> Result<Spend, crate::ProofError> {
        Ok(Spend {
            body: self.spend_body(fvk, compliance_keys),
            auth_sig,
            proof: self.spend_proof(fvk, auth_path, anchor, pk, compliance_keys)?,
        })
    }

    /// Construct the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_body(
        &self,
        fvk: &FullViewingKey,
        _compliance_keys: Option<(decaf377::Element, decaf377::Element)>,
    ) -> Body {
        // Construct the backreference for this spend.
        let backref = Backref::new(self.note.commit());
        let encrypted_backref = backref.encrypt(&fvk.backref_key(), &self.nullifier(fvk));

        // Compute blinded leaf hashes for privacy-preserving counterparty binding
        let user_leaf = self.compliance_leaf.clone().unwrap_or_else(|| {
            penumbra_sdk_compliance::ComplianceLeaf {
                address: self.note.address().clone(),
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                    *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                ),
                asset_id: self.note.asset_id(),
            }
        });
        let sender_leaf_hash = user_leaf.commit();

        let counterparty_leaf_hash = self
            .counterparty_leaf
            .clone()
            .unwrap_or_else(|| penumbra_sdk_compliance::ComplianceLeaf {
                address: self.note.address().clone(),
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                    *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                ),
                asset_id: self.note.asset_id(),
            })
            .commit();

        let blinded_sender_leaf =
            penumbra_sdk_compliance::blind_sender_leaf(sender_leaf_hash, self.tx_blinding_nonce);
        let blinded_counterparty_leaf = penumbra_sdk_compliance::blind_counterparty_leaf(
            counterparty_leaf_hash,
            self.tx_blinding_nonce,
        );

        // Use the precomputed compliance ciphertext
        // (should be set via set_compliance_details() before calling this method)
        Body {
            balance_commitment: self.balance().commit(self.value_blinding),
            nullifier: self.nullifier(fvk),
            rk: self.rk(fvk),
            encrypted_backref,
            compliance_ciphertext: self.compliance_ciphertext.clone(),
            target_timestamp: self.target_timestamp,
            sender_leaf_hash: blinded_sender_leaf,
            counterparty_leaf_hash: blinded_counterparty_leaf,
            compliance_anchor: self.compliance_anchor,
            asset_anchor: self.asset_anchor,
        }
    }

    /// Construct the randomized verification key associated with this [`SpendPlan`].
    pub fn rk(&self, fvk: &FullViewingKey) -> decaf377_rdsa::VerificationKey<SpendAuth> {
        fvk.spend_verification_key().randomize(&self.randomizer)
    }

    /// Construct the [`Nullifier`] associated with this [`SpendPlan`].
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        let nk = fvk.nullifier_key();
        Nullifier::derive(nk, self.position, &self.note.commit())
    }

    /// Construct the [`SpendProof`] required by the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_proof(
        &self,
        fvk: &FullViewingKey,
        state_commitment_proof: tct::Proof,
        anchor: tct::Root,
        pk: &ProvingKey<Bls12_377>,
        _compliance_keys: Option<(decaf377::Element, decaf377::Element)>,
    ) -> Result<SpendProof, crate::ProofError> {
        // Use the anchors from the plan (set via enrich_with_compliance)
        let asset_anchor = self.asset_anchor;
        let compliance_anchor = self.compliance_anchor;

        // Use the precomputed compliance ciphertext and leaf
        // (should be set via set_compliance_details() before calling this method)
        let user_leaf = self.compliance_leaf.clone().unwrap_or_else(|| {
            // Fallback to BLACK_HOLE for backwards compatibility
            penumbra_sdk_compliance::ComplianceLeaf {
                address: self.note.address().clone(),
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                    *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                ),
                asset_id: self.note.asset_id(),
            }
        });

        // Extract compliance ciphertext using unified method
        use penumbra_sdk_compliance::structs::ComplianceCiphertext;
        let ct = ComplianceCiphertext::from_bytes(&self.compliance_ciphertext)
            .expect("can deserialize ciphertext");
        let (compliance_epk, compliance_ciphertext) = ct.to_circuit_public_inputs();

        // Compute blinded leaf hashes for privacy-preserving counterparty binding
        let sender_leaf_hash = user_leaf.commit();
        let counterparty_leaf_hash = self
            .counterparty_leaf
            .clone()
            .unwrap_or_else(|| {
                // Fallback to BLACK_HOLE for unregulated assets
                penumbra_sdk_compliance::ComplianceLeaf {
                    address: self.note.address().clone(),
                    key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                        *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                    ),
                    asset_id: self.note.asset_id(),
                }
            })
            .commit();

        let blinded_sender_leaf =
            penumbra_sdk_compliance::blind_sender_leaf(sender_leaf_hash, self.tx_blinding_nonce);
        let blinded_counterparty_leaf = penumbra_sdk_compliance::blind_counterparty_leaf(
            counterparty_leaf_hash,
            self.tx_blinding_nonce,
        );

        let public = SpendProofPublic {
            anchor,
            balance_commitment: self.balance().commit(self.value_blinding),
            nullifier: self.nullifier(fvk),
            rk: self.rk(fvk),
            asset_anchor,
            compliance_anchor,
            compliance_epk,
            compliance_ciphertext,
            target_timestamp: self.target_timestamp,
            sender_leaf_hash: blinded_sender_leaf,
            counterparty_leaf_hash: blinded_counterparty_leaf,
        };

        let private = SpendProofPrivate {
            state_commitment_proof,
            note: self.note.clone(),
            v_blinding: self.value_blinding,
            spend_auth_randomizer: self.randomizer,
            ak: *fvk.spend_verification_key(),
            nk: *fvk.nullifier_key(),
            asset_path: self.asset_path.clone(),
            asset_position: self.asset_position,
            is_regulated: self.is_regulated,
            compliance_path: self.compliance_path.clone(),
            compliance_position: self.compliance_position,
            user_leaf,
            compliance_ephemeral_secret: self.compliance_ephemeral_secret.unwrap_or(Fr::from(0u64)),
            counterparty_leaf: self.counterparty_leaf.clone().unwrap_or_else(|| {
                // Fallback to BLACK_HOLE for unregulated assets
                penumbra_sdk_compliance::ComplianceLeaf {
                    address: self.note.address().clone(),
                    key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                        *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                    ),
                    asset_id: self.note.asset_id(),
                }
            }),
            tx_blinding_nonce: self.tx_blinding_nonce,
        };

        SpendProof::prove(
            self.proof_blinding_r,
            self.proof_blinding_s,
            pk,
            public,
            private,
        )
    }

    pub fn balance(&self) -> Balance {
        Value {
            amount: self.note.value().amount,
            asset_id: self.note.value().asset_id,
        }
        .into()
    }
}

impl DomainType for SpendPlan {
    type Proto = pb::SpendPlan;
}

impl From<SpendPlan> for pb::SpendPlan {
    fn from(msg: SpendPlan) -> Self {
        use penumbra_sdk_proto::core::component::compliance::v1 as compliance_pb;

        Self {
            note: Some(msg.note.into()),
            position: u64::from(msg.position),
            randomizer: msg.randomizer.to_bytes().to_vec(),
            value_blinding: msg.value_blinding.to_bytes().to_vec(),
            proof_blinding_r: msg.proof_blinding_r.to_bytes().to_vec(),
            proof_blinding_s: msg.proof_blinding_s.to_bytes().to_vec(),
            target_timestamp: msg.target_timestamp,
            compliance_ciphertext: msg.compliance_ciphertext,
            is_regulated: msg.is_regulated,
            compliance_leaf: msg
                .compliance_leaf
                .map(|leaf| compliance_pb::ComplianceLeaf {
                    address: Some(leaf.address.into()),
                    key: Some(compliance_pb::ComplianceViewingKey {
                        inner: leaf.key.0.vartime_compress().0.to_vec(),
                    }),
                    asset_id: Some(leaf.asset_id.into()),
                }),
            counterparty_leaf: msg
                .counterparty_leaf
                .map(|leaf| compliance_pb::ComplianceLeaf {
                    address: Some(leaf.address.into()),
                    key: Some(compliance_pb::ComplianceViewingKey {
                        inner: leaf.key.0.vartime_compress().0.to_vec(),
                    }),
                    asset_id: Some(leaf.asset_id.into()),
                }),
            compliance_ephemeral_secret: msg
                .compliance_ephemeral_secret
                .map(|s| s.to_bytes().to_vec())
                .unwrap_or_default(),
            counterparty_address: msg.counterparty_address.map(Into::into),
            tx_blinding_nonce: msg.tx_blinding_nonce.to_bytes().to_vec(),
            compliance_anchor: Some(msg.compliance_anchor.into()),
            asset_anchor: Some(msg.asset_anchor.into()),
        }
    }
}

impl TryFrom<pb::SpendPlan> for SpendPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SpendPlan) -> Result<Self, Self::Error> {
        use penumbra_sdk_keys::keys::AddressComplianceKey;

        // Parse compliance leaf if present
        let compliance_leaf = msg
            .compliance_leaf
            .map(
                |leaf| -> anyhow::Result<penumbra_sdk_compliance::ComplianceLeaf> {
                    let address = leaf
                        .address
                        .ok_or_else(|| anyhow::anyhow!("missing address in compliance leaf"))?
                        .try_into()?;
                    let key = leaf
                        .key
                        .ok_or_else(|| anyhow::anyhow!("missing key in compliance leaf"))?;
                    let key_bytes: [u8; 32] = key
                        .inner
                        .as_slice()
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("invalid compliance key length"))?;
                    let key_element = decaf377::Encoding(key_bytes)
                        .vartime_decompress()
                        .map_err(|_| anyhow::anyhow!("invalid compliance key encoding"))?;
                    let asset_id = leaf
                        .asset_id
                        .ok_or_else(|| anyhow::anyhow!("missing asset_id in compliance leaf"))?
                        .try_into()?;
                    Ok(penumbra_sdk_compliance::ComplianceLeaf {
                        address,
                        key: AddressComplianceKey::new(key_element),
                        asset_id,
                    })
                },
            )
            .transpose()?;

        // Parse counterparty leaf if present
        let counterparty_leaf = msg
            .counterparty_leaf
            .map(
                |leaf| -> anyhow::Result<penumbra_sdk_compliance::ComplianceLeaf> {
                    let address = leaf
                        .address
                        .ok_or_else(|| anyhow::anyhow!("missing address in counterparty leaf"))?
                        .try_into()?;
                    let key = leaf
                        .key
                        .ok_or_else(|| anyhow::anyhow!("missing key in counterparty leaf"))?;
                    let key_bytes: [u8; 32] = key
                        .inner
                        .as_slice()
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("invalid counterparty key length"))?;
                    let key_element = decaf377::Encoding(key_bytes)
                        .vartime_decompress()
                        .map_err(|_| anyhow::anyhow!("invalid counterparty key encoding"))?;
                    let asset_id = leaf
                        .asset_id
                        .ok_or_else(|| anyhow::anyhow!("missing asset_id in counterparty leaf"))?
                        .try_into()?;
                    Ok(penumbra_sdk_compliance::ComplianceLeaf {
                        address,
                        key: AddressComplianceKey::new(key_element),
                        asset_id,
                    })
                },
            )
            .transpose()?;

        // Parse ephemeral secret if present (non-empty bytes)
        let compliance_ephemeral_secret = if msg.compliance_ephemeral_secret.is_empty() {
            None
        } else {
            let bytes: [u8; 32] = msg
                .compliance_ephemeral_secret
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid ephemeral secret length"))?;
            Some(Fr::from_bytes_checked(&bytes).expect("ephemeral secret malformed"))
        };

        // Parse tx_blinding_nonce (default to zero if empty for backwards compatibility)
        let tx_blinding_nonce = if msg.tx_blinding_nonce.is_empty() {
            Fr::from(0u64)
        } else {
            let bytes: [u8; 32] = msg
                .tx_blinding_nonce
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid tx_blinding_nonce length"))?;
            Fr::from_bytes_checked(&bytes).expect("tx_blinding_nonce malformed")
        };

        // Parse compliance_anchor if present
        let compliance_anchor = msg
            .compliance_anchor
            .map(|c| c.try_into())
            .transpose()?
            .unwrap_or_else(|| tct::StateCommitment(Fq::from(0u64)));

        // Parse asset_anchor if present
        let asset_anchor = msg
            .asset_anchor
            .map(|c| c.try_into())
            .transpose()?
            .unwrap_or_else(|| tct::StateCommitment(Fq::from(0u64)));

        Ok(Self {
            note: msg
                .note
                .ok_or_else(|| anyhow::anyhow!("missing note"))?
                .try_into()?,
            position: msg.position.into(),
            randomizer: Fr::from_bytes_checked(msg.randomizer.as_slice().try_into()?)
                .expect("randomizer malformed"),
            value_blinding: Fr::from_bytes_checked(msg.value_blinding.as_slice().try_into()?)
                .expect("value_blinding malformed"),
            proof_blinding_r: Fq::from_bytes_checked(msg.proof_blinding_r.as_slice().try_into()?)
                .expect("proof_blinding_r malformed"),
            proof_blinding_s: Fq::from_bytes_checked(msg.proof_blinding_s.as_slice().try_into()?)
                .expect("proof_blinding_s malformed"),
            compliance_path: MerklePath::default(),
            target_timestamp: msg.target_timestamp,
            compliance_ciphertext: msg.compliance_ciphertext,
            compliance_leaf,
            compliance_ephemeral_secret,
            is_regulated: msg.is_regulated,
            counterparty_address: msg.counterparty_address.map(|a| a.try_into()).transpose()?,
            counterparty_leaf,
            tx_blinding_nonce,
            compliance_anchor,
            asset_anchor,
            asset_path: MerklePath::default(), // Set via enrich_with_compliance
            asset_position: 0,                 // Set via enrich_with_compliance
            compliance_position: 0,            // Set via enrich_with_compliance
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::spend::proof::SpendCircuit;
    use crate::test_proof_helpers::proof_test_helpers::*;
    use rand_core::OsRng;

    fn verify_spend_proof_with_asset(asset_id_u64: u64) {
        use crate::test_proof_helpers::proof_test_helpers::generate_test_data;

        let mut rng = OsRng;

        // 1. Generate unified test data
        let is_regulated = asset_id_u64 == REGULATED_ASSET_ID;
        let test_data = generate_test_data(&mut rng, asset_id_u64, 100, is_regulated);

        // 2. Setup circuit keys and SCT
        let (pk, pvk, _blinding_r, _blinding_s) = setup_groth16_keys::<SpendCircuit>();

        let mut sct = penumbra_sdk_tct::Tree::new();
        let note_commitment = test_data.note.commit();
        sct.insert(penumbra_sdk_tct::Witness::Keep, note_commitment)
            .unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        let fvk = test_data.sk.full_viewing_key();

        // 3. Create SpendPlan using test data
        let mut spend_plan = SpendPlan::new(
            &mut rng,
            test_data.note.clone(),
            state_commitment_proof.position(),
        );

        // Create a recipient leaf (using same ACK for simplicity in this test)
        let recipient_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: test_data.address.clone(),
            key: test_data.ack.clone(),
            asset_id: penumbra_sdk_asset::asset::Id(Fq::from(asset_id_u64)),
        };

        spend_plan
            .set_compliance_details(
                &mut rng,
                &test_data.ack,
                &test_data.address,
                is_regulated,
                &test_data.address,
                recipient_leaf,
            )
            .expect("can set compliance details");

        // 4. Generate Proof
        let spend_proof = spend_plan
            .spend_proof(&fvk, state_commitment_proof, anchor, &pk, None)
            .expect("proof generation should succeed");

        // 5. Setup dummy compliance anchors
        let (asset_anchor, compliance_anchor) = dummy_compliance_anchors();

        // Extract ciphertext from spend_plan
        use penumbra_sdk_compliance::structs::ComplianceCiphertext;
        let ct = ComplianceCiphertext::from_bytes(&spend_plan.compliance_ciphertext)
            .expect("can deserialize ciphertext");
        let (compliance_epk, packed_ciphertext) = ct.to_circuit_public_inputs();

        // Compute blinded leaf hashes
        let sender_leaf_hash = spend_plan.compliance_leaf.clone().unwrap().commit();
        let counterparty_leaf_hash = spend_plan.counterparty_leaf.clone().unwrap().commit();

        let blinded_sender = penumbra_sdk_compliance::blind_sender_leaf(
            sender_leaf_hash,
            spend_plan.tx_blinding_nonce,
        );
        let blinded_counterparty = penumbra_sdk_compliance::blind_counterparty_leaf(
            counterparty_leaf_hash,
            spend_plan.tx_blinding_nonce,
        );

        // 6. Verify
        spend_proof
            .verify(
                &pvk,
                SpendProofPublic {
                    anchor,
                    balance_commitment: spend_plan.balance().commit(spend_plan.value_blinding),
                    nullifier: spend_plan.nullifier(&fvk),
                    rk: spend_plan.rk(&fvk),
                    asset_anchor,
                    compliance_anchor,
                    compliance_epk,
                    compliance_ciphertext: packed_ciphertext,
                    target_timestamp: spend_plan.target_timestamp,
                    sender_leaf_hash: blinded_sender,
                    counterparty_leaf_hash: blinded_counterparty,
                },
            )
            .unwrap();
    }

    #[test]
    fn test_regulated_asset_spend_proof() {
        verify_spend_proof_with_asset(REGULATED_ASSET_ID);
    }

    #[test]
    fn test_unregulated_asset_spend_proof() {
        verify_spend_proof_with_asset(UNREGULATED_ASSET_ID);
    }
}
