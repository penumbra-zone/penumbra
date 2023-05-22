use ark_ff::UniformRand;
use penumbra_crypto::{
    ka,
    keys::{IncomingViewingKey, OutgoingViewingKey},
    proofs::groth16::OutputProof,
    symmetric::WrappedMemoKey,
    Address, FieldExt, Fr, Note, PayloadKey, Rseed, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType, TypeUrl};
use rand_core::{CryptoRng, OsRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{Body, Output};

/// A planned [`Output`](Output).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::OutputPlan", into = "pb::OutputPlan")]
pub struct OutputPlan {
    pub value: Value,
    pub dest_address: Address,
    pub rseed: Rseed,
    pub value_blinding: Fr,
}

impl OutputPlan {
    /// Create a new [`OutputPlan`] that sends `value` to `dest_address`.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        dest_address: Address,
    ) -> OutputPlan {
        let rseed = Rseed::generate(rng);
        let value_blinding = Fr::rand(rng);
        Self {
            value,
            dest_address,
            rseed,
            value_blinding,
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

    /// Convenience method to construct the [`Output`] described by this
    /// [`OutputPlan`].
    #[cfg_attr(docsrs, doc(cfg(feature = "proving-keys")))]
    #[cfg(feature = "proving-keys")]
    pub fn output(&self, ovk: &OutgoingViewingKey, memo_key: &PayloadKey) -> Output {
        Output {
            body: self.output_body(ovk, memo_key),
            proof: self.output_proof(),
        }
    }

    pub fn output_note(&self) -> Note {
        Note::from_parts(self.dest_address, self.value, self.rseed)
            .expect("transmission key in address is always valid")
    }

    /// Construct the [`OutputProof`] required by the [`output::Body`] described
    /// by this plan.
    #[cfg_attr(docsrs, doc(cfg(feature = "proving-keys")))]
    #[cfg(feature = "proving-keys")]
    pub fn output_proof(&self) -> OutputProof {
        let note = self.output_note();
        let balance_commitment = self.balance().commit(self.value_blinding);
        let note_commitment = note.commit();
        OutputProof::prove(
            &mut OsRng,
            &penumbra_proof_params::OUTPUT_PROOF_PROVING_KEY,
            note.clone(),
            self.value_blinding,
            balance_commitment,
            note_commitment,
        )
        .expect("can generate ZKOutputProof")
    }

    /// Construct the [`output::Body`] described by this plan.
    pub fn output_body(&self, ovk: &OutgoingViewingKey, memo_key: &PayloadKey) -> Body {
        // Prepare the output note and commitment.
        let note = self.output_note();
        let balance_commitment = self.balance().commit(self.value_blinding);

        // Encrypt the note to the recipient...
        let esk: ka::Secret = note.ephemeral_secret_key();
        // ... and wrap the encryption key to ourselves.
        let ovk_wrapped_key = note.encrypt_key(ovk, balance_commitment);

        let wrapped_memo_key = WrappedMemoKey::encrypt(
            memo_key,
            esk,
            note.transmission_key(),
            &note.diversified_generator(),
        );

        Body {
            note_payload: note.payload(),
            balance_commitment,
            ovk_wrapped_key,
            wrapped_memo_key,
        }
    }

    /// Checks whether this plan's output is viewed by the given IVK.
    pub fn is_viewed_by(&self, ivk: &IncomingViewingKey) -> bool {
        ivk.views_address(&self.dest_address)
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        -penumbra_crypto::Balance::from(self.value)
    }
}

impl TypeUrl for OutputPlan {
    const TYPE_URL: &'static str = "/penumbra.core.transaction.v1alpha1.OutputPlan";
}

impl DomainType for OutputPlan {
    type Proto = pb::OutputPlan;
}

impl From<OutputPlan> for pb::OutputPlan {
    fn from(msg: OutputPlan) -> Self {
        Self {
            value: Some(msg.value.into()),
            dest_address: Some(msg.dest_address.into()),
            rseed: msg.rseed.to_bytes().to_vec().into(),
            value_blinding: msg.value_blinding.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::OutputPlan> for OutputPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::OutputPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            value: msg
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value"))?
                .try_into()?,
            dest_address: msg
                .dest_address
                .ok_or_else(|| anyhow::anyhow!("missing address"))?
                .try_into()?,
            rseed: Rseed(msg.rseed.as_ref().try_into()?),
            value_blinding: Fr::from_bytes(msg.value_blinding.as_ref().try_into()?)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::OutputPlan;
    use penumbra_crypto::keys::{SeedPhrase, SpendKey};
    use penumbra_crypto::{PayloadKey, Value};
    use penumbra_proof_params::OUTPUT_PROOF_VERIFICATION_KEY;
    use rand_core::OsRng;

    #[test]
    /// Check that a valid output proof passes the `penumbra_crypto` integrity checks successfully.
    /// This test serves to anchor how an `OutputPlan` prepares its `OutputProof`, in particular
    /// the balance and note commitments.
    fn check_output_proof_verification() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let ovk = sk.full_viewing_key().outgoing();
        let dummy_memo_key: PayloadKey = [0; 32].into();

        let value: Value = "1234.02penumbra".parse().unwrap();
        let dest_address = "penumbrav2t1f5h060qspaga3vvwf2mwak2dj6ugymxd2et5h6l3n0u2y57lcv4t7j2m8n75nm7qmhg4v3csexl5slm6tm5hg5wyw39fv2q0jnpwdjn3llduzgmg5d3efuqq6ymn76t0hvgage".parse().unwrap();

        let output_plan = OutputPlan::new(&mut rng, value, dest_address);
        let blinding_factor = output_plan.value_blinding;

        let _body = output_plan.output_body(ovk, &dummy_memo_key);

        let balance_commitment = output_plan.balance().commit(blinding_factor);
        let note_commitment = output_plan.output_note().commit();
        let output_proof = output_plan.output_proof();

        output_proof
            .verify(
                &OUTPUT_PROOF_VERIFICATION_KEY,
                balance_commitment,
                note_commitment,
            )
            .unwrap();
    }
}
