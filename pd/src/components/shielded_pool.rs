use std::collections::BTreeSet;

use anyhow::{anyhow, Context, Result};
use ark_ff::PrimeField;
use async_trait::async_trait;
use decaf377::{Fq, Fr};
use penumbra_chain::{sync::CompactBlock, NoteSource};
use penumbra_crypto::{
    asset,
    asset::Denom,
    ka,
    merkle::{self, Frontier, NoteCommitmentTree, TreeExt},
    Address, Note, Nullifier, One, Value,
};
use penumbra_transaction::{action::output, Action, Transaction};
use tendermint::abci;
use tracing::instrument;

use super::{Component, Overlay};
use crate::{genesis, WriteOverlayExt};

// Stub component
pub struct ShieldedPool {
    overlay: Overlay,
    note_commitment_tree: NoteCommitmentTree,
    /// The in-progress CompactBlock representation of the ShieldedPool changes
    compact_block: CompactBlock,
}

#[async_trait]
impl Component for ShieldedPool {
    async fn new(overlay: Overlay) -> Result<Self> {
        let note_commitment_tree = Self::get_nct(&overlay).await?;

        Ok(Self {
            overlay,
            note_commitment_tree,
            compact_block: Default::default(),
        })
    }

    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            if allocation.amount == 0 {
                return Err(anyhow!(
                    "Genesis allocations contain empty note: {:?}",
                    allocation
                ));
            }

            let denom = asset::REGISTRY
                .parse_denom(&allocation.denom)
                .ok_or_else(|| {
                    anyhow!(
                        "Genesis denomination {} is not a base denom",
                        allocation.denom
                    )
                })?;

            self.overlay.register_denom(&denom).await;
            self.mint_note(
                Value {
                    amount: allocation.amount,
                    asset_id: denom.id(),
                },
                &allocation.address,
                NoteSource::Genesis,
            )
            .await?;
        }

        self.compact_block.height = 0;
        self.write_block().await?;

        Ok(())
    }

    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) -> Result<()> {
        Ok(())
    }

    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
        let sighash = tx.transaction_body().sighash();

        // 1. Check binding signature.
        tx.binding_verification_key()
            .verify(&sighash, tx.binding_sig())
            .context("binding signature failed to verify")?;

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();

        for action in tx.transaction_body().actions {
            match action {
                Action::Output(output) => {
                    if output
                        .proof
                        .verify(
                            output.value_commitment,
                            output.body.note_commitment,
                            output.body.ephemeral_key,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("An output proof did not verify"));
                    }
                }
                Action::Spend(spend) => {
                    spend
                        .body
                        .rk
                        .verify(&sighash, &spend.auth_sig)
                        .context("spend auth signature failed to verify")?;

                    if spend
                        .body
                        .proof
                        .verify(
                            tx.transaction_body().merkle_root,
                            spend.body.value_commitment,
                            spend.body.nullifier.clone(),
                            spend.body.rk,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("A spend proof did not verify"));
                    }

                    // Check nullifier has not been revealed already in this transaction.
                    if spent_nullifiers.contains(&spend.body.nullifier.clone()) {
                        return Err(anyhow::anyhow!("Double spend"));
                    }

                    spent_nullifiers.insert(spend.body.nullifier.clone());
                }
                Action::Delegate(_delegate) => {
                    // Handled in the `Staking` component.
                }
                Action::Undelegate(_undelegate) => {
                    // Handled in the `Staking` component.
                }
                Action::ValidatorDefinition(_validator) => {
                    // Handled in the `Staking` component.
                }
                #[allow(unreachable_patterns)]
                _ => {
                    return Err(anyhow::anyhow!("unsupported action"));
                }
            }
        }

        Ok(())
    }

    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        // TODO: rename transaction_body.merkle_root now that we have 2 merkle trees
        self.check_claimed_anchor(&tx.transaction_body.merkle_root)
            .await?;

        for spent_nullifier in tx.spent_nullifiers() {
            self.check_nullifier_unspent(spent_nullifier).await?;
        }

        // TODO: handle quarantine
        Ok(())
    }

    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        let should_quarantine = tx
            .transaction_body
            .actions
            .iter()
            .any(|action| matches!(action, Action::Undelegate { .. }));

        let source = NoteSource::Transaction { id: tx.id() };

        if should_quarantine {
            tracing::warn!("skipping processing, TODO: implement");
        } else {
            for compact_output in tx.output_bodies() {
                self.add_note(compact_output, source).await;
            }
            for spent_nullifier in tx.spent_nullifiers() {
                self.spend_nullifier(spent_nullifier, source).await;
            }
        }

        Ok(())
    }

    async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Result<()> {
        self.compact_block.height = end_block.height as u64;
        self.write_block().await?;
        Ok(())
    }
}

impl ShieldedPool {
    #[instrument(skip(self))]
    async fn mint_note(
        &mut self,
        value: Value,
        address: &Address,
        source: NoteSource,
    ) -> Result<()> {
        // These notes are public, so we don't need a blinding factor for privacy,
        // but since the note commitments are determined by the note contents, we
        // need to have unique (deterministic) blinding factors for each note, so they
        // cannot collide.
        //
        // Hashing the current NCT root is sufficient, since it will change every time
        // we insert a new note.
        let blinding_factor = Fq::from_le_bytes_mod_order(
            blake2b_simd::Params::default()
                .personal(b"PenumbraMint")
                .to_state()
                .update(&self.note_commitment_tree.root2().to_bytes())
                .finalize()
                .as_bytes(),
        );

        let note = Note::from_parts(
            *address.diversifier(),
            *address.transmission_key(),
            value,
            blinding_factor,
        )?;
        let note_commitment = note.commit();

        tracing::debug!(?note_commitment, "minted tokens");

        // Scanning assumes that notes are encrypted, so we need to create
        // note ciphertexts, even if the plaintexts are known.  Use the key
        // "1" to ensure we have contributory behaviour in note encryption.
        let esk = ka::Secret::new_from_field(Fr::one());
        let ephemeral_key = esk.diversified_public(&note.diversified_generator());
        let encrypted_note = note.encrypt(&esk);

        // Now record the note and update the total supply:
        self.overlay
            .update_token_supply(&value.asset_id, value.amount as i64)
            .await?;
        self.add_note(
            output::Body {
                note_commitment,
                ephemeral_key,
                encrypted_note,
            },
            source,
        )
        .await;

        Ok(())
    }

    #[instrument(skip(self, source, output_body))]
    async fn add_note(&mut self, output_body: output::Body, source: NoteSource) {
        tracing::debug!(commitment = ?output_body.note_commitment, "appending to NCT in component");
        // 1. Insert it into the NCT
        self.note_commitment_tree
            .append(&output_body.note_commitment);
        // 2. Record its source in the JMT
        self.overlay.lock().await.put(
            format!("shielded_pool/note_source/{}", &output_body.note_commitment).into(),
            source.to_bytes().to_vec(),
        );
        // 3. Finally, record it in the pending compact block.
        self.compact_block.outputs.push(output_body);
    }

    #[instrument(skip(self))]
    async fn write_block(&mut self) -> Result<()> {
        // Write the CompactBlock:
        self.overlay
            .put_domain(
                format!("shielded_pool/compact_block/{}", self.compact_block.height).into(),
                self.compact_block.clone(),
            )
            .await;
        // and the note commitment tree data and anchor:
        self.put_nct_anchor().await;
        self.put_nct().await?;

        // TODO: write out the updated supply and denoms ?

        Ok(())
    }

    async fn put_nct(&mut self) -> Result<()> {
        let nct_data = bincode::serialize(&self.note_commitment_tree)?;
        self.overlay
            .lock()
            .await
            .put(b"shielded_pool/nct_data".into(), nct_data);
        Ok(())
    }

    /// This is an associated function rather than a method,
    /// so that we can call it in the constructor to get the NCT.
    async fn get_nct(overlay: &Overlay) -> Result<NoteCommitmentTree> {
        if let Ok(Some(bytes)) = overlay
            .lock()
            .await
            .get(b"shielded_pool/nct_data".into())
            .await
        {
            bincode::deserialize(&bytes).map_err(Into::into)
        } else {
            Ok(NoteCommitmentTree::new(0))
        }
    }

    async fn put_nct_anchor(&mut self) {
        let height = self.compact_block.height;
        let nct_anchor = self.note_commitment_tree.root2();
        tracing::debug!(anchor = %nct_anchor, "writing anchor to tree");
        // TODO: should we save a list of historical anchors?
        // Write the NCT anchor both as a value, so we can look it up,
        self.overlay.lock().await.put(
            format!("shielded_pool/nct_anchor/{}", height).into(),
            nct_anchor.to_bytes().to_vec(),
        );
        // and as a key, so we can query for it.
        self.overlay.lock().await.put(
            format!("shielded_pool/valid_anchors/{}", nct_anchor).into(),
            // We don't use the value for validity checks, but writing the height
            // here lets us find out what height the anchor was for.
            height.to_le_bytes().to_vec(),
        );
    }

    /// Checks whether a claimed NCT anchor is a previous valid state root.
    async fn check_claimed_anchor(&self, anchor: &merkle::Root) -> Result<()> {
        if self
            .overlay
            .lock()
            .await
            .get(format!("shielded_pool/valid_anchors/{}", anchor).into())
            .await?
            .is_some()
        {
            Ok(())
        } else {
            Err(anyhow!(
                "provided anchor {} is not a valid NCT root",
                anchor
            ))
        }
    }

    #[instrument(skip(self))]
    async fn spend_nullifier(&mut self, nullifier: Nullifier, source: NoteSource) {
        self.overlay.lock().await.put(
            format!("shielded_pool/spent_nullifiers/{}", nullifier).into(),
            // We don't use the value for validity checks, but writing the source
            // here lets us find out what transaction spent the nullifier.
            source.to_bytes().to_vec(),
        );
    }

    #[instrument(skip(self))]
    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(source_bytes) = self
            .overlay
            .lock()
            .await
            .get(format!("shielded_pool/spent_nullifiers/{}", nullifier).into())
            .await?
        {
            let source_bytes: [u8; 32] = source_bytes.try_into().unwrap();
            let source = NoteSource::try_from(source_bytes).expect("source is validly encoded");
            Err(anyhow!(
                "Nullifier {} was already spent in {:?}",
                nullifier,
                source
            ))
        } else {
            Ok(())
        }
    }
}

/// Extension trait providing read/write access to shielded pool data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: WriteOverlayExt {
    async fn token_supply(&self, asset_id: &asset::Id) -> Result<Option<u64>> {
        self.get_proto(format!("shielded_pool/assets/{}/token_supply", asset_id).into())
            .await
    }

    #[instrument(skip(self))]
    async fn update_token_supply(&self, asset_id: &asset::Id, change: i64) -> Result<()> {
        let key = format!("shielded_pool/assets/{}/token_supply", asset_id).into();
        let current_supply = match self.get_proto(key).await {
            Ok(Some(value)) => value,
            Ok(None) => 0u64,
            // We want to handle the MissingRootError specially here, so that we can
            // use 0 as a default value if the tree is empty, without ignoring all errors.
            Err(e) if e.downcast_ref::<jmt::MissingRootError>().is_some() => 0u64,
            Err(e) => return Err(e),
        };

        // TODO: replace with a single checked_add_signed call when mixed_integer_ops lands in stable
        let new_supply = if change < 0 {
            current_supply
                .checked_sub(change.abs() as u64)
                .ok_or_else(|| {
                    anyhow!(
                        "underflow updating token supply {} with delta {}",
                        current_supply,
                        change
                    )
                })?
        } else {
            current_supply.checked_add(change as u64).ok_or_else(|| {
                anyhow!(
                    "overflow updating token supply {} with delta {}",
                    current_supply,
                    change
                )
            })?
        };
        tracing::debug!(?current_supply, ?new_supply);

        self.put_proto(key, new_supply).await;
        Ok(())
    }

    async fn denom_by_asset(&self, asset_id: &asset::Id) -> Result<Option<Denom>> {
        Ok(self
            .get_proto(format!("shielded_pool/assets/{}/denom", asset_id).into())
            .await?
            .map(|denom: String| {
                asset::REGISTRY
                    .parse_denom(&denom)
                    .expect("tree only records valid denoms")
            }))
    }

    #[instrument(skip(self))]
    async fn register_denom(&self, denom: &Denom) {
        let asset_id = denom.id();
        tracing::debug!(?asset_id);
        self.put_proto(
            format!("shielded_pool/assets/{}/denom", asset_id).into(),
            denom.to_string(),
        )
        .await
    }
}

impl<T: WriteOverlayExt> View for T {}
