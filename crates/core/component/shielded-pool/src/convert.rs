use anyhow::{anyhow, Result};
use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_relations::r1cs;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use base64::prelude::*;
use decaf377::{Bls12_377, Fq, Fr};
use penumbra_asset::{
    asset::{self, AssetIdVar},
    balance::{self, commitment::BalanceCommitmentVar, BalanceVar},
    Balance, Value, ValueVar, STAKING_TOKEN_ASSET_ID,
};
use penumbra_num::{
    fixpoint::{U128x128, U128x128Var},
    Amount, AmountVar,
};
use penumbra_proof_params::{DummyWitness, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};

/// The public input for a [`ConvertProof`].
#[derive(Clone, Debug)]
pub struct ConvertProofPublic {
    /// The source asset being consumed.
    pub from: asset::Id,
    /// The destination asset being produced.
    pub to: asset::Id,
    /// The exchange rate: how many units of `to` we get for each unit of `from`.
    pub rate: U128x128,
    /// A commitment to the balance of this transaction: what assets were consumed and produced.
    pub balance_commitment: balance::Commitment,
}

/// The private input for a [`ConvertProof`].
#[derive(Clone, Debug)]
pub struct ConvertProofPrivate {
    /// The private amount of the source asset we're converting.
    pub amount: Amount,
    /// The blinding we used to create the public commitment.
    pub balance_blinding: Fr,
}

#[cfg(test)]
fn check_satisfaction(public: &ConvertProofPublic, private: &ConvertProofPrivate) -> Result<()> {
    let consumed = Value {
        amount: private.amount,
        asset_id: public.from,
    };
    let produced = Value {
        amount: public.rate.apply_to_amount(&private.amount)?,
        asset_id: public.to,
    };
    let balance: Balance = Balance::from(produced) - consumed;
    let commitment = balance.commit(private.balance_blinding);
    if commitment != public.balance_commitment {
        anyhow::bail!("balance commitment did not match public input");
    }
    Ok(())
}

#[cfg(test)]
fn check_circuit_satisfaction(
    public: ConvertProofPublic,
    private: ConvertProofPrivate,
) -> Result<()> {
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};

    let cs = ConstraintSystem::new_ref();
    let circuit = ConvertCircuit::new(public, private);
    cs.set_optimization_goal(r1cs::OptimizationGoal::Constraints);
    // For why this is ok, see `generate_test_parameters`.
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints from circuit");
    cs.finalize();
    if !cs.is_satisfied()? {
        anyhow::bail!("constraints are not satisfied");
    }
    Ok(())
}

/// A circuit that converts a private amount of one asset into another, by some rate.
#[derive(Clone, Debug)]
pub struct ConvertCircuit {
    /// The amount of the source token being converted.
    amount: Amount,
    /// A randomizer for the commitment.
    balance_blinding: Fr,
    /// The source asset.
    from: asset::Id,
    /// The target asset
    to: asset::Id,
    /// The conversion rate from source to target.
    rate: U128x128,
    /// A commitment to a balance of `-amount[from] + (rate * amount)[to]`.
    balance_commitment: balance::Commitment,
}

impl ConvertCircuit {
    fn new(
        ConvertProofPublic {
            from,
            to,
            rate,
            balance_commitment,
        }: ConvertProofPublic,
        ConvertProofPrivate {
            amount,
            balance_blinding,
        }: ConvertProofPrivate,
    ) -> Self {
        Self {
            amount,
            balance_blinding,
            balance_commitment,
            from,
            to,
            rate,
        }
    }
}

impl r1cs::ConstraintSynthesizer<Fq> for ConvertCircuit {
    fn generate_constraints(self, cs: r1cs::ConstraintSystemRef<Fq>) -> r1cs::Result<()> {
        use ark_r1cs_std::prelude::*;

        // Witnesses
        let amount_var = AmountVar::new_witness(cs.clone(), || Ok(self.amount))?;
        let balance_blinding_var = {
            let balance_blinding_arr: [u8; 32] = self.balance_blinding.to_bytes();
            UInt8::new_witness_vec(cs.clone(), &balance_blinding_arr)?
        };

        // Public Inputs
        let from = AssetIdVar::new_input(cs.clone(), || Ok(self.from))?;
        let to = AssetIdVar::new_input(cs.clone(), || Ok(self.to))?;
        let rate = U128x128Var::new_input(cs.clone(), || Ok(self.rate))?;
        let balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;

        // Constraints
        let expected_balance = {
            let taken = BalanceVar::from_negative_value_var(ValueVar {
                amount: amount_var.clone(),
                asset_id: from,
            });

            let produced = BalanceVar::from_positive_value_var(ValueVar {
                amount: rate.apply_to_amount(amount_var)?,
                asset_id: to,
            });

            taken + produced
        };
        let expected_commitment = expected_balance.commit(balance_blinding_var)?;
        expected_commitment.enforce_equal(&balance_commitment)?;

        Ok(())
    }
}

impl DummyWitness for ConvertCircuit {
    fn with_dummy_witness() -> Self {
        let amount = Amount::from(1u64);
        let balance_blinding = Fr::from(1u64);
        let from = *STAKING_TOKEN_ASSET_ID;
        let to = *STAKING_TOKEN_ASSET_ID;
        let rate = U128x128::from(1u64);
        let balance = Balance::from(Value {
            asset_id: to,
            amount,
        }) - Balance::from(Value {
            asset_id: from,
            amount,
        });
        let balance_commitment = balance.commit(balance_blinding);
        Self {
            amount,
            balance_blinding,
            from,
            to,
            rate,
            balance_commitment,
        }
    }
}

/// A proof that one asset was correctly converted into another.
///
/// This checks that: `COMMITMENT = COMMIT(-amount[FROM] + (RATE * amount)[TO])`,
/// where `amount` is private, and other variables are public.
#[derive(Clone, Debug)]
pub struct ConvertProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl ConvertProof {
    /// Generate a [`ConvertProof`]
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: ConvertProofPublic,
        private: ConvertProofPrivate,
    ) -> Result<Self> {
        let circuit = ConvertCircuit::new(public, private);
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?BASE64_STANDARD.encode(&self.0), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: ConvertProofPublic,
    ) -> Result<()> {
        let proof = Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(
            public
                .from
                .to_field_elements()
                .ok_or_else(|| anyhow!("could not convert `from` asset ID to field elements"))?,
        );
        public_inputs.extend(
            public
                .to
                .to_field_elements()
                .ok_or_else(|| anyhow!("could not convert `to` asset ID to field elements"))?,
        );
        public_inputs.extend(
            public
                .rate
                .to_field_elements()
                .ok_or_else(|| anyhow!("could not convert exchange rate to field elements"))?,
        );
        public_inputs.extend(
            public
                .balance_commitment
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow!("could not convert balance commitment to field elements"))?,
        );

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow!("undelegate claim proof did not verify"))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for ConvertProof {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        Ok(Self(value.try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_valid_convert_statement(balance_blinding: Fr)(amount in any::<u64>(), from_asset_id64 in any::<u64>(), to_asset_id64 in any::<u64>(), rate in any::<(u64, u128)>()) -> (ConvertProofPublic, ConvertProofPrivate) {
            let rate = U128x128::ratio(u128::from(rate.0), rate.1).expect("the bounds will make this not overflow");
            let from = asset::Id(Fq::from(from_asset_id64));
            let to = asset::Id(Fq::from(to_asset_id64));
            let amount = Amount::from(amount);
            let balance = Balance::from(Value { asset_id: to, amount: rate.apply_to_amount(&amount).expect("the bounds will make this not overflow")}) - Value {asset_id: from, amount};
            let public = ConvertProofPublic { from, to, rate, balance_commitment: balance.commit(balance_blinding) };
            let private = ConvertProofPrivate { amount, balance_blinding };
            (public, private)
        }
    }

    proptest! {
        #[test]
        fn convert_proof_happy_path((public, private) in arb_valid_convert_statement(Fr::from(1u64))) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    fn nonzero_u128() -> impl Strategy<Value = u128> {
        prop::num::u128::ANY.prop_filter("nonzero", |x| *x != 0)
    }

    fn nonzero_u64() -> impl Strategy<Value = u64> {
        prop::num::u64::ANY.prop_filter("nonzero", |x| *x != 0)
    }

    prop_compose! {
        // The circuit should be unsatisfiable if the rate used by the prover is incorrect.
        // We generate a random rate, filtering out non-zero denominators to avoid division by zero.
        // This is the "true" rate.
        // Next, we add a (u64) random value to the true rate, and the prover generates the balance
        // using this incorrect rate.
        fn arb_invalid_convert_statement_wrong_rate(balance_blinding: Fr)(amount in any::<u64>(), from_asset_id64 in any::<u64>(), to_asset_id64 in any::<u64>(), rate_num in nonzero_u64(), rate_denom in nonzero_u128(), random_rate_num in nonzero_u64()) -> (ConvertProofPublic, ConvertProofPrivate) {
            let rate = U128x128::ratio(u128::from(rate_num), rate_denom).expect("the bounds will make this not overflow");
            let incorrect_rate = rate.checked_add(&U128x128::ratio(random_rate_num, 1u64).expect("should not overflow")).expect("should not overflow");
            let from = asset::Id(Fq::from(from_asset_id64));
            let to = asset::Id(Fq::from(to_asset_id64));
            let amount = Amount::from(amount);
            let balance = Balance::from(Value { asset_id: to, amount: incorrect_rate.apply_to_amount(&amount).expect("the bounds will make this not overflow")}) - Value {asset_id: from, amount};
            let public = ConvertProofPublic { from, to, rate, balance_commitment: balance.commit(balance_blinding) };
            let private = ConvertProofPrivate { amount, balance_blinding };
            (public, private)
        }
    }

    proptest! {
        #[test]
        fn convert_proof_invalid_convert_statement_wrong_rate((public, private) in arb_invalid_convert_statement_wrong_rate(Fr::from(1u64))) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }
}
