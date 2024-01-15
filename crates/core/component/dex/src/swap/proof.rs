use anyhow::Context;
use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use decaf377::{Bls12_377, FieldExt};
use decaf377::{Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;
use penumbra_fee::Fee;
use penumbra_proto::{core::component::dex::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use penumbra_tct::r1cs::StateCommitmentVar;

use penumbra_asset::{
    asset,
    balance::{self, commitment::BalanceCommitmentVar, BalanceVar},
    Value,
};
use penumbra_keys::{keys::Diversifier, Address};
use penumbra_shielded_pool::Rseed;

use crate::{
    swap::{SwapPlaintext, SwapPlaintextVar},
    TradingPair,
};

use penumbra_proof_params::{DummyWitness, GROTH16_PROOF_LENGTH_BYTES};

/// The public inputs to a [`SwapProof`].
#[derive(Clone, Debug)]
pub struct SwapProofPublic {
    /// A commitment to the balance of this transaction.
    pub balance_commitment: balance::Commitment,
    /// A commitment to the swap.
    pub swap_commitment: tct::StateCommitment,
    /// A commitment to the fee that was paid.
    pub fee_commitment: balance::Commitment,
}

/// The private inputs to a [`SwapProof`].
#[derive(Clone, Debug)]
pub struct SwapProofPrivate {
    /// A randomizer to make the commitment to the fee hiding.
    pub fee_blinding: Fr,
    /// All information about the swap.
    pub swap_plaintext: SwapPlaintext,
}

pub struct SwapCircuit {
    public: SwapProofPublic,
    private: SwapProofPrivate,
}

impl ConstraintSynthesizer<Fq> for SwapCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let swap_plaintext_var =
            SwapPlaintextVar::new_witness(cs.clone(), || Ok(self.private.swap_plaintext.clone()))?;
        let fee_blinding_var =
            UInt8::new_witness_vec(cs.clone(), &self.private.fee_blinding.to_bytes())?;

        // Inputs
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.public.balance_commitment))?;
        let claimed_swap_commitment =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.swap_commitment))?;
        let claimed_fee_commitment =
            BalanceCommitmentVar::new_input(cs, || Ok(self.public.fee_commitment))?;

        // Swap commitment integrity check
        let swap_commitment = swap_plaintext_var.commit()?;
        claimed_swap_commitment.enforce_equal(&swap_commitment)?;

        // Fee commitment integrity check
        let fee_balance = BalanceVar::from_negative_value_var(swap_plaintext_var.claim_fee.clone());
        let fee_commitment = fee_balance.commit(fee_blinding_var)?;
        claimed_fee_commitment.enforce_equal(&fee_commitment)?;

        // Reconstruct swap action balance commitment
        let transparent_blinding_var = UInt8::constant_vec(&[0u8; 32]);
        let balance_1 = BalanceVar::from_negative_value_var(swap_plaintext_var.delta_1_value());
        let balance_2 = BalanceVar::from_negative_value_var(swap_plaintext_var.delta_2_value());
        let balance_1_commit = balance_1.commit(transparent_blinding_var.clone())?;
        let balance_2_commit = balance_2.commit(transparent_blinding_var)?;
        let transparent_balance_commitment = balance_1_commit + balance_2_commit;
        let total_balance_commitment = transparent_balance_commitment + fee_commitment;

        // Balance commitment integrity check
        claimed_balance_commitment.enforce_equal(&total_balance_commitment)?;

        Ok(())
    }
}

impl DummyWitness for SwapCircuit {
    fn with_dummy_witness() -> Self {
        let a = asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .expect("upenumbra asset exists");
        let b = asset::Cache::with_known_assets()
            .get_unit("nala")
            .expect("nala asset exists");
        let trading_pair = TradingPair::new(a.id(), b.id());
        let diversifier_bytes = [1u8; 16];
        let pk_d_bytes = decaf377::basepoint().vartime_compress().0;
        let clue_key_bytes = [1; 32];
        let diversifier = Diversifier(diversifier_bytes);
        let address = Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        )
        .expect("generated 1 address");
        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i: 100000u64.into(),
            delta_2_i: 1u64.into(),
            claim_fee: Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .expect("upenumbra asset exists")
                    .id(),
            }),
            claim_address: address,
            rseed: Rseed([1u8; 32]),
        };

        Self {
            private: SwapProofPrivate {
                swap_plaintext: swap_plaintext.clone(),
                fee_blinding: Fr::from(1),
            },
            public: SwapProofPublic {
                swap_commitment: swap_plaintext.swap_commitment(),
                fee_commitment: balance::Commitment(decaf377::basepoint()),
                balance_commitment: balance::Commitment(decaf377::basepoint()),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct SwapProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl SwapProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: SwapProofPublic,
        private: SwapProofPrivate,
    ) -> anyhow::Result<Self> {
        let circuit = SwapCircuit { public, private };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * balance commitment,
    /// * swap commitment,
    /// * fee commimtment,
    ///
    // Commented out, but this may be useful when debugging proof verification failures,
    // to check that the proof data and verification keys are consistent.
    //#[tracing::instrument(skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    #[tracing::instrument(skip(self, vk))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: SwapProofPublic,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(
            public
                .balance_commitment
                .0
                .to_field_elements()
                .context("balance_commitment should be a Bls12-377 field member")?,
        );
        public_inputs.extend(
            public
                .swap_commitment
                .0
                .to_field_elements()
                .context("swap_commitment should be a Bls12-377 field member")?,
        );
        public_inputs.extend(
            public
                .fee_commitment
                .0
                .to_field_elements()
                .context("fee_commitment should be a Bls12-377 field member")?,
        );

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("a swap proof did not verify"))
    }
}

impl DomainType for SwapProof {
    type Proto = pb::ZkSwapProof;
}

impl From<SwapProof> for pb::ZkSwapProof {
    fn from(proof: SwapProof) -> Self {
        pb::ZkSwapProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkSwapProof> for SwapProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkSwapProof) -> Result<Self, Self::Error> {
        Ok(SwapProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{PrimeField, UniformRand};
    use penumbra_asset::{Balance, Value};
    use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
    use penumbra_num::Amount;
    use penumbra_proof_params::generate_prepared_test_parameters;
    use proptest::prelude::*;
    use rand_core::OsRng;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn swap_proof_happy_path(fee_blinding in fr_strategy(), value1_amount in 2..200u64) {
        let mut rng = OsRng;
        let (pk, vk) = generate_prepared_test_parameters::<SwapCircuit>(&mut rng);


        let seed_phrase = SeedPhrase::generate(rng);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1 = Amount::from(value1_amount);
        let delta_2 = Amount::from(0u64);
        let fee = Fee::default();

        let swap_plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);
        let fee_commitment = swap_plaintext.claim_fee.commit(fee_blinding);
        let swap_commitment = swap_plaintext.swap_commitment();

        let value_1 = Value {
            amount: swap_plaintext.delta_1_i,
            asset_id: swap_plaintext.trading_pair.asset_1(),
        };
        let value_2 = Value {
            amount: swap_plaintext.delta_2_i,
            asset_id:  swap_plaintext.trading_pair.asset_2(),
        };
        let value_fee = Value {
            amount: swap_plaintext.claim_fee.amount(),
            asset_id: swap_plaintext.claim_fee.asset_id(),
        };
        let mut balance = Balance::default();
        balance -= value_1;
        balance -= value_2;
        balance -= value_fee;
        let balance_commitment = balance.commit(fee_blinding);

        let public = SwapProofPublic { balance_commitment, swap_commitment, fee_commitment };
        let private = SwapProofPrivate { fee_blinding, swap_plaintext };

        let blinding_r = Fq::rand(&mut rng);
        let blinding_s = Fq::rand(&mut rng);

        let proof = SwapProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            public.clone(),
            private
        )
        .expect("can create proof");

        let proof_result = proof.verify(&vk, public);

        assert!(proof_result.is_ok());
        }
    }
}
