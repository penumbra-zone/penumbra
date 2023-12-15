use ark_relations::r1cs;
use decaf377::{FieldExt, Fq, Fr};
use penumbra_asset::{
    asset::{self, AssetIdVar},
    balance::{self, commitment::BalanceCommitmentVar, BalanceVar},
    Balance, Value, ValueVar, STAKING_TOKEN_ASSET_ID,
};
use penumbra_num::{
    fixpoint::{U128x128, U128x128Var},
    Amount, AmountVar,
};
use penumbra_proof_params::DummyWitness;

/// A circuit that converts a private amount of one asset into another, by some rate.
#[derive(Clone, Debug)]
pub struct ConvertCircuit {
    /// The amount of the source token being converted.
    amount: Amount,
    /// A randomizer for the commitment.
    balance_blinding: Fr,
    /// The source asset.
    pub from: asset::Id,
    /// The target asset
    pub to: asset::Id,
    /// The conversion rate from source to target.
    pub rate: U128x128,
    /// A commitment to a balance of `-amount[from] + (rate * amount)[to]`.
    pub balance_commitment: balance::Commitment,
}

impl ConvertCircuit {
    pub fn new(
        amount: Amount,
        balance_blinding: Fr,
        balance_commitment: balance::Commitment,
        from: asset::Id,
        to: asset::Id,
        rate: U128x128,
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
        let to = AssetIdVar::new_input(cs.clone(), || Ok(self.from))?;
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
        let balance_blinding = Fr::from(1);
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
