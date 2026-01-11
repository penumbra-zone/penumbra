use ark_groth16::ProvingKey;
use decaf377::{Bls12_377, Fq, Fr};
use decaf377_ka as ka;
use penumbra_sdk_asset::{Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_compliance::MerklePath;
use penumbra_sdk_keys::{
    keys::{IncomingViewingKey, OutgoingViewingKey},
    symmetric::WrappedMemoKey,
    Address, PayloadKey,
};
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{Body, Output, OutputProof, OutputProofPrivate, OutputProofPublic};
use crate::{Note, Rseed};

/// A planned [`Output`](Output).
///
/// # Compliance Data Architecture Decision
///
/// See [`SpendPlan`](crate::SpendPlan) for the rationale on storing compliance data
/// directly in plans rather than in a separate WitnessData-like structure.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::OutputPlan", into = "pb::OutputPlan")]
pub struct OutputPlan {
    pub value: Value,
    pub dest_address: Address,
    pub rseed: Rseed,
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
    /// Counterparty address (the sender of this output)
    #[serde(skip)]
    pub counterparty_address: Option<Address>,
    /// Counterparty compliance leaf (the sender's leaf, for binding)
    #[serde(skip)]
    pub counterparty_leaf: Option<penumbra_sdk_compliance::ComplianceLeaf>,
    /// Shared transaction blinding nonce (same for spend and output in one transaction)
    #[serde(skip)]
    pub tx_blinding_nonce: Fr,
    /// Compliance anchor (user tree root) for proof generation
    #[serde(skip)]
    pub compliance_anchor: penumbra_sdk_tct::StateCommitment,
    /// Asset anchor (asset tree root) for proof generation
    #[serde(skip)]
    pub asset_anchor: penumbra_sdk_tct::StateCommitment,
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

impl OutputPlan {
    /// Set compliance details for this output plan.
    ///
    /// This should be called after constructing the plan to properly encrypt
    /// the compliance ciphertext using ACK.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator
    /// * `recipient_ack` - The recipient's Wallet Compliance Key (from their registry entry)
    /// * `is_regulated` - Whether the asset is regulated
    /// * `sender_address` - The sender's address (counterparty)
    /// * `sender_leaf` - The sender's compliance leaf (from registry)
    /// * `tx_blinding_nonce` - Shared transaction blinding nonce (from spend)
    pub fn set_compliance_details(
        &mut self,
        rng: &mut (impl rand_core::RngCore + rand_core::CryptoRng),
        recipient_ack: &penumbra_sdk_keys::keys::AddressComplianceKey,
        is_regulated: bool,
        sender_address: &Address,
        sender_leaf: penumbra_sdk_compliance::ComplianceLeaf,
        tx_blinding_nonce: Fr,
    ) -> anyhow::Result<()> {
        let date = crate::timestamp_to_day_index(self.target_timestamp);
        let note = self.output_note();

        let compliance_data = crate::generate_compliance_details(
            rng,
            is_regulated,
            recipient_ack,
            &self.dest_address,
            date,
            note.asset_id(),
            note.amount(),
            sender_address.clone(),
        )?;

        self.compliance_ciphertext = compliance_data.ciphertext;
        self.compliance_leaf = Some(compliance_data.leaf);
        self.compliance_ephemeral_secret = Some(compliance_data.ephemeral_secret);
        self.is_regulated = is_regulated;
        self.counterparty_address = Some(sender_address.clone());
        self.counterparty_leaf = Some(sender_leaf);
        self.tx_blinding_nonce = tx_blinding_nonce;

        Ok(())
    }

    /// Create a new [`OutputPlan`] that sends `value` to `dest_address`.
    ///
    /// Uses the current system time as the target timestamp.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        dest_address: Address,
    ) -> OutputPlan {
        let rseed = Rseed::generate(rng);
        let value_blinding = Fr::rand(rng);
        let target_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_secs();

        // Generate valid compliance data using BLACK_HOLE_ACK for unregulated assets.
        // This ensures circuit constraints are satisfied even without explicit compliance setup.
        let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(
            *penumbra_sdk_compliance::BLACK_HOLE_ACK,
        );
        let date = target_timestamp / 86400; // Convert timestamp to day index

        let (compliance_ciphertext_obj, ephemeral_secret) =
            penumbra_sdk_compliance::crypto::encrypt_compliance_details(
                &mut *rng,
                &black_hole_ack,
                &dest_address,
                date,
                value.asset_id,
                value.amount,
                dest_address.clone(), // Use same address as counterparty for default
            )
            .expect("can encrypt compliance details");

        let compliance_ciphertext = compliance_ciphertext_obj.to_bytes();

        let compliance_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: dest_address.clone(),
            key: black_hole_ack.clone(),
            asset_id: value.asset_id,
        };

        // Create 16-layer dummy Merkle path
        let dummy_path = MerklePath {
            layers: (0..16)
                .map(|_| penumbra_sdk_compliance::structs::MerklePathLayer {
                    siblings: vec![vec![0u8; 32], vec![0u8; 32], vec![0u8; 32]],
                })
                .collect(),
        };

        let tx_blinding_nonce = Fr::rand(rng);

        Self {
            value,
            dest_address,
            rseed,
            value_blinding,
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
            tx_blinding_nonce,
            compliance_anchor: penumbra_sdk_tct::StateCommitment(Fq::from(0u64)),
            asset_anchor: penumbra_sdk_tct::StateCommitment(Fq::from(0u64)),
            asset_path: dummy_path,
            asset_position: 0,
            compliance_position: 0,
        }
    }

    /// Create a dummy [`OutputPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(rng: &mut R) -> OutputPlan {
        let dummy_address = Address::dummy(rng);
        Self::new(
            rng,
            Value {
                amount: 0u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
            dummy_address,
        )
    }

    /// Convenience method to construct the [`Output`] described by this plan.
    ///
    /// `compliance_keys` is an optional tuple of (Issuer PK, Clue Key).
    /// If provided, the output will be encrypted for compliance.
    /// If None, it will use the "Black Hole" key (unregulated).
    pub fn output(
        &self,
        ovk: &OutgoingViewingKey,
        memo_key: &PayloadKey,
        pk: &ProvingKey<Bls12_377>,
        compliance_keys: Option<(decaf377::Element, decaf377::Element)>,
    ) -> Result<Output, crate::ProofError> {
        Ok(Output {
            body: self.output_body(ovk, memo_key, compliance_keys),
            proof: self.output_proof(pk, compliance_keys)?,
        })
    }

    pub fn output_note(&self) -> Note {
        Note::from_parts(self.dest_address.clone(), self.value, self.rseed)
            .expect("transmission key in address is always valid")
    }

    /// Construct the [`OutputProof`].
    pub fn output_proof(
        &self,
        pk: &ProvingKey<Bls12_377>,
        _compliance_keys: Option<(decaf377::Element, decaf377::Element)>,
    ) -> Result<OutputProof, crate::ProofError> {
        let note = self.output_note();
        let balance_commitment = self.balance().commit(self.value_blinding);
        let note_commitment = note.commit();

        // Use the anchors from the plan (set via enrich_with_compliance)
        let asset_anchor = self.asset_anchor;
        let compliance_anchor = self.compliance_anchor;

        // Use the precomputed compliance ciphertext and leaf
        // (should be set via set_compliance_details() before calling this method)
        let user_leaf = self.compliance_leaf.clone().unwrap_or_else(|| {
            // Fallback to BLACK_HOLE for backwards compatibility
            penumbra_sdk_compliance::ComplianceLeaf {
                address: note.address().clone(),
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                    *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                ),
                asset_id: note.asset_id(),
            }
        });

        // Extract compliance ciphertext using unified method
        use penumbra_sdk_compliance::structs::ComplianceCiphertext;
        let ct = ComplianceCiphertext::from_bytes(&self.compliance_ciphertext)
            .expect("can deserialize ciphertext");
        let (compliance_epk, compliance_ciphertext) = ct.to_circuit_public_inputs();

        // Compute blinded leaf hashes for privacy-preserving counterparty binding
        let receiver_leaf_hash = user_leaf.commit();
        let counterparty_leaf_hash = self
            .counterparty_leaf
            .clone()
            .unwrap_or_else(|| {
                // Fallback to BLACK_HOLE for unregulated assets
                penumbra_sdk_compliance::ComplianceLeaf {
                    address: note.address().clone(),
                    key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                        *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                    ),
                    asset_id: note.asset_id(),
                }
            })
            .commit();

        let blinded_receiver_leaf = penumbra_sdk_compliance::blind_counterparty_leaf(
            receiver_leaf_hash,
            self.tx_blinding_nonce,
        );
        let blinded_counterparty_leaf = penumbra_sdk_compliance::blind_sender_leaf(
            counterparty_leaf_hash,
            self.tx_blinding_nonce,
        );

        OutputProof::prove(
            self.proof_blinding_r,
            self.proof_blinding_s,
            pk,
            OutputProofPublic {
                balance_commitment,
                note_commitment,
                compliance_epk,
                compliance_ciphertext,
                asset_anchor,
                compliance_anchor,
                target_timestamp: self.target_timestamp,
                receiver_leaf_hash: blinded_receiver_leaf,
                counterparty_leaf_hash: blinded_counterparty_leaf,
            },
            OutputProofPrivate {
                note: note.clone(),
                balance_blinding: self.value_blinding,
                asset_path: self.asset_path.clone(),
                asset_position: self.asset_position,
                is_regulated: self.is_regulated,
                compliance_path: self.compliance_path.clone(),
                compliance_position: self.compliance_position,
                user_leaf,
                compliance_ephemeral_secret: self
                    .compliance_ephemeral_secret
                    .unwrap_or(Fr::from(0u64)),
                counterparty_leaf: self.counterparty_leaf.clone().unwrap_or_else(|| {
                    // Fallback to BLACK_HOLE for unregulated assets
                    penumbra_sdk_compliance::ComplianceLeaf {
                        address: note.address().clone(),
                        key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                            *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                        ),
                        asset_id: note.asset_id(),
                    }
                }),
                tx_blinding_nonce: self.tx_blinding_nonce,
            },
        )
    }

    pub fn output_body(
        &self,
        ovk: &OutgoingViewingKey,
        memo_key: &PayloadKey,
        _compliance_keys: Option<(decaf377::Element, decaf377::Element)>,
    ) -> Body {
        let note = self.output_note();
        let balance_commitment = self.balance().commit(self.value_blinding);

        let esk: ka::Secret = note.ephemeral_secret_key();
        let ovk_wrapped_key = note.encrypt_key(ovk, balance_commitment);

        let wrapped_memo_key = WrappedMemoKey::encrypt(
            memo_key,
            esk,
            note.transmission_key(),
            &note.diversified_generator(),
        );

        // Compute blinded leaf hashes for privacy-preserving counterparty binding
        let user_leaf = self.compliance_leaf.clone().unwrap_or_else(|| {
            penumbra_sdk_compliance::ComplianceLeaf {
                address: note.address().clone(),
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                    *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                ),
                asset_id: note.asset_id(),
            }
        });
        let receiver_leaf_hash = user_leaf.commit();

        let counterparty_leaf_hash = self
            .counterparty_leaf
            .clone()
            .unwrap_or_else(|| penumbra_sdk_compliance::ComplianceLeaf {
                address: note.address().clone(),
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(
                    *penumbra_sdk_compliance::BLACK_HOLE_ACK,
                ),
                asset_id: note.asset_id(),
            })
            .commit();

        let blinded_receiver_leaf = penumbra_sdk_compliance::blind_counterparty_leaf(
            receiver_leaf_hash,
            self.tx_blinding_nonce,
        );
        let blinded_counterparty_leaf = penumbra_sdk_compliance::blind_sender_leaf(
            counterparty_leaf_hash,
            self.tx_blinding_nonce,
        );

        // Use the precomputed compliance ciphertext
        // (should be set via set_compliance_details() before calling this method)
        Body {
            note_payload: note.payload(),
            balance_commitment,
            ovk_wrapped_key,
            wrapped_memo_key,
            compliance_ciphertext: self.compliance_ciphertext.clone(),
            target_timestamp: self.target_timestamp,
            receiver_leaf_hash: blinded_receiver_leaf,
            counterparty_leaf_hash: blinded_counterparty_leaf,
            compliance_anchor: self.compliance_anchor,
            asset_anchor: self.asset_anchor,
        }
    }

    pub fn is_viewed_by(&self, ivk: &IncomingViewingKey) -> bool {
        ivk.views_address(&self.dest_address)
    }

    pub fn balance(&self) -> Balance {
        -Balance::from(self.value)
    }
}

impl DomainType for OutputPlan {
    type Proto = pb::OutputPlan;
}

impl From<OutputPlan> for pb::OutputPlan {
    fn from(msg: OutputPlan) -> Self {
        use penumbra_sdk_proto::core::component::compliance::v1 as compliance_pb;

        Self {
            value: Some(msg.value.into()),
            dest_address: Some(msg.dest_address.into()),
            rseed: msg.rseed.to_bytes().to_vec(),
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

impl TryFrom<pb::OutputPlan> for OutputPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::OutputPlan) -> Result<Self, Self::Error> {
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
            .unwrap_or_else(|| penumbra_sdk_tct::StateCommitment(Fq::from(0u64)));

        // Parse asset_anchor if present
        let asset_anchor = msg
            .asset_anchor
            .map(|c| c.try_into())
            .transpose()?
            .unwrap_or_else(|| penumbra_sdk_tct::StateCommitment(Fq::from(0u64)));

        Ok(Self {
            value: msg
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value"))?
                .try_into()?,
            dest_address: msg
                .dest_address
                .ok_or_else(|| anyhow::anyhow!("missing address"))?
                .try_into()?,
            rseed: Rseed(msg.rseed.as_slice().try_into()?),
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
    use super::OutputPlan;
    use crate::output::proof::OutputCircuit;
    use crate::output::OutputProofPublic;

    use crate::test_proof_helpers::proof_test_helpers::*;
    use decaf377::Fr;
    use penumbra_sdk_keys::PayloadKey;
    use rand_core::OsRng;

    /// Helper to run the full verification flow for a specific asset ID.
    fn verify_output_proof_with_asset(asset_id_u64: u64) {
        use crate::test_proof_helpers::proof_test_helpers::generate_test_data;

        let mut rng = OsRng;

        // 1. Generate unified test data
        let is_regulated = asset_id_u64 == REGULATED_ASSET_ID;
        let test_data = generate_test_data(&mut rng, asset_id_u64, 100, is_regulated);

        // 2. Setup circuit keys
        let (pk, pvk, _blinding_r, _blinding_s) = setup_groth16_keys::<OutputCircuit>();

        let ovk = test_data.sk.full_viewing_key().outgoing();
        let dummy_memo_key: PayloadKey = [0; 32].into();

        // 3. Create OutputPlan using test data
        let mut output_plan = OutputPlan::new(&mut rng, test_data.value, test_data.address.clone());
        let blinding_factor = output_plan.value_blinding;

        // Create sender leaf (using same data as receiver for simplicity in test)
        let sender_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: test_data.address.clone(),
            key: test_data.ack.clone(),
            asset_id: test_data.note.asset_id(),
        };
        let tx_blinding_nonce = Fr::rand(&mut rng);

        output_plan
            .set_compliance_details(
                &mut rng,
                &test_data.ack,
                is_regulated,
                &test_data.address,
                sender_leaf,
                tx_blinding_nonce,
            )
            .expect("can set compliance details");

        let _body = output_plan.output_body(ovk, &dummy_memo_key, None);

        let balance_commitment = output_plan.balance().commit(blinding_factor);
        let note_commitment = output_plan.output_note().commit();

        // 4. Generate Proof
        let output_proof = output_plan
            .output_proof(&pk, None)
            .expect("proof generation should succeed");

        // 5. Setup dummy compliance anchors
        let (asset_anchor, compliance_anchor) = dummy_compliance_anchors();

        // Extract ciphertext from output_plan
        use penumbra_sdk_compliance::structs::ComplianceCiphertext;
        let ct = ComplianceCiphertext::from_bytes(&output_plan.compliance_ciphertext)
            .expect("can deserialize ciphertext");
        let (compliance_epk, packed_ciphertext) = ct.to_circuit_public_inputs();

        // Compute blinded leaf hashes
        let receiver_leaf_hash = output_plan.compliance_leaf.clone().unwrap().commit();
        let counterparty_leaf_hash = output_plan.counterparty_leaf.clone().unwrap().commit();

        let blinded_receiver = penumbra_sdk_compliance::blind_counterparty_leaf(
            receiver_leaf_hash,
            output_plan.tx_blinding_nonce,
        );
        let blinded_counterparty = penumbra_sdk_compliance::blind_sender_leaf(
            counterparty_leaf_hash,
            output_plan.tx_blinding_nonce,
        );

        // 6. Verify
        output_proof
            .verify(
                &pvk,
                OutputProofPublic {
                    balance_commitment,
                    note_commitment,
                    compliance_epk,
                    compliance_ciphertext: packed_ciphertext,
                    asset_anchor,
                    compliance_anchor,
                    target_timestamp: output_plan.target_timestamp,
                    receiver_leaf_hash: blinded_receiver,
                    counterparty_leaf_hash: blinded_counterparty,
                },
            )
            .unwrap();
    }

    #[test]
    fn test_regulated_asset_output_proof() {
        verify_output_proof_with_asset(REGULATED_ASSET_ID);
    }

    #[test]
    fn test_unregulated_asset_output_proof() {
        verify_output_proof_with_asset(UNREGULATED_ASSET_ID);
    }
}
