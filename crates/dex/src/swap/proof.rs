use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use decaf377::{Bls12_377, FieldExt};
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;
use rand_core::{CryptoRngCore, OsRng};

use penumbra_crypto::{
    asset,
    balance::{self, commitment::BalanceCommitmentVar, BalanceVar},
    fmd, ka,
    keys::Diversifier,
    note::StateCommitmentVar,
    Address, Fee, Fq, Fr, Rseed, Value,
};

use crate::{
    swap::{SwapPlaintext, SwapPlaintextVar},
    TradingPair,
};

use penumbra_crypto::proofs::groth16::{ParameterSetup, GROTH16_PROOF_LENGTH_BYTES};

pub struct SwapCircuit {
    /// The swap plaintext.
    swap_plaintext: SwapPlaintext,
    /// The blinding factor for the fee commitment.
    fee_blinding: Fr,
    /// Balance commitment of the swap.
    pub balance_commitment: balance::Commitment,
    /// Swap commitment.
    pub swap_commitment: tct::Commitment,
    /// Balance commitment of the fee.
    pub fee_commitment: balance::Commitment,
}

impl SwapCircuit {
    pub fn new(
        swap_plaintext: SwapPlaintext,
        fee_blinding: Fr,
        balance_commitment: balance::Commitment,
        swap_commitment: tct::Commitment,
        fee_commitment: balance::Commitment,
    ) -> Self {
        Self {
            swap_plaintext,
            fee_blinding,
            balance_commitment,
            swap_commitment,
            fee_commitment,
        }
    }
}

impl ConstraintSynthesizer<Fq> for SwapCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let swap_plaintext_var =
            SwapPlaintextVar::new_witness(cs.clone(), || Ok(self.swap_plaintext.clone()))?;
        let fee_blinding_var = UInt8::new_witness_vec(cs.clone(), &self.fee_blinding.to_bytes())?;

        // Inputs
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;
        let claimed_swap_commitment =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.swap_commitment))?;
        let claimed_fee_commitment =
            BalanceCommitmentVar::new_input(cs, || Ok(self.fee_commitment))?;

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

impl ParameterSetup for SwapCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let a = asset::REGISTRY.parse_unit("upenumbra");
        let b: asset::Unit = asset::REGISTRY.parse_unit("nala");
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
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            }),
            claim_address: address,
            rseed: Rseed([1u8; 32]),
        };

        let circuit = SwapCircuit {
            swap_plaintext: swap_plaintext.clone(),
            fee_blinding: Fr::from(1),
            swap_commitment: swap_plaintext.swap_commitment(),
            fee_commitment: balance::Commitment(decaf377::basepoint()),
            balance_commitment: balance::Commitment(decaf377::basepoint()),
        };
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct SwapProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl SwapProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRngCore>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        swap_plaintext: SwapPlaintext,
        fee_blinding: Fr,
        balance_commitment: balance::Commitment,
        swap_commitment: tct::Commitment,
        fee_commitment: balance::Commitment,
    ) -> anyhow::Result<Self> {
        let circuit = SwapCircuit {
            swap_plaintext,
            fee_blinding,
            balance_commitment,
            swap_commitment,
            fee_commitment,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(pk, circuit, rng)
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
        balance_commitment: balance::Commitment,
        swap_commitment: tct::Commitment,
        fee_commitment: balance::Commitment,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(swap_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(fee_commitment.0.to_field_elements().unwrap());

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
            .ok_or_else(|| anyhow::anyhow!("proof did not verify"))
    }
}

impl TypeUrl for SwapProof {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.ZKSwapProof";
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
    use ark_ff::PrimeField;
    use penumbra_crypto::{
        keys::{SeedPhrase, SpendKey},
        Amount, Balance, Value,
    };
    use proptest::prelude::*;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn swap_proof_happy_path(fee_blinding in fr_strategy(), value1_amount in 2..200u64) {
        let (pk, vk) = SwapCircuit::generate_prepared_test_parameters();

        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
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

            let proof = SwapProof::prove(
                &mut rng,
                &pk,
                swap_plaintext,
                fee_blinding,
                balance_commitment,
                swap_commitment,
                fee_commitment
            )
            .expect("can create proof");

            let proof_result = proof.verify(&vk, balance_commitment, swap_commitment, fee_commitment);

            assert!(proof_result.is_ok());
        }
    }
}
