use decaf377::Bls12_377;

use ark_groth16::{PreparedVerifyingKey, ProvingKey};
use base64::prelude::*;
use penumbra_proto::{core::component::stake::v1 as pb, DomainType};

use decaf377::{Fq, Fr};
use penumbra_asset::{asset, balance, STAKING_TOKEN_ASSET_ID};
use penumbra_num::Amount;
use penumbra_proof_params::VerifyingKeyExt;
use penumbra_shielded_pool::{ConvertProof, ConvertProofPrivate, ConvertProofPublic};

use crate::Penalty;

/// The public inputs to an [`UndelegateClaimProof`].
#[derive(Clone, Debug)]
pub struct UndelegateClaimProofPublic {
    pub balance_commitment: balance::Commitment,
    pub unbonding_id: asset::Id,
    pub penalty: Penalty,
}

impl From<UndelegateClaimProofPublic> for ConvertProofPublic {
    fn from(value: UndelegateClaimProofPublic) -> Self {
        Self {
            from: value.unbonding_id,
            to: *STAKING_TOKEN_ASSET_ID,
            rate: value.penalty.kept_rate(),
            balance_commitment: value.balance_commitment,
        }
    }
}

/// The private inputs to an [`UndelegateClaimProof`].
#[derive(Clone, Debug)]
pub struct UndelegateClaimProofPrivate {
    pub unbonding_amount: Amount,
    pub balance_blinding: Fr,
}

impl From<UndelegateClaimProofPrivate> for ConvertProofPrivate {
    fn from(value: UndelegateClaimProofPrivate) -> Self {
        Self {
            amount: value.unbonding_amount,
            balance_blinding: value.balance_blinding,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UndelegateClaimProof(ConvertProof);

impl UndelegateClaimProof {
    #![allow(clippy::too_many_arguments)]
    /// Generate an `UndelegateClaimProof` given the proving key, public inputs,
    /// witness data, and two random elements `blinding_r` and `blinding_s`.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: UndelegateClaimProofPublic,
        private: UndelegateClaimProofPrivate,
    ) -> anyhow::Result<Self> {
        let proof = ConvertProof::prove(blinding_r, blinding_s, pk, public.into(), private.into())?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?BASE64_STANDARD.encode(self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: UndelegateClaimProofPublic,
    ) -> anyhow::Result<()> {
        self.0.verify(vk, public.into())
    }
}

impl DomainType for UndelegateClaimProof {
    type Proto = pb::ZkUndelegateClaimProof;
}

impl From<UndelegateClaimProof> for pb::ZkUndelegateClaimProof {
    fn from(proof: UndelegateClaimProof) -> Self {
        pb::ZkUndelegateClaimProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkUndelegateClaimProof> for UndelegateClaimProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkUndelegateClaimProof) -> Result<Self, Self::Error> {
        Ok(UndelegateClaimProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decaf377::{Fq, Fr};
    use decaf377_rdsa as rdsa;
    use penumbra_num::Amount;
    use penumbra_proof_params::generate_prepared_test_parameters;
    use proptest::prelude::*;
    use rand_core::OsRng;

    use crate::{IdentityKey, Penalty, UnbondingToken};
    use penumbra_shielded_pool::ConvertCircuit;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn undelegate_claim_proof_happy_path(validator_randomness in fr_strategy(), balance_blinding in fr_strategy(), value1_amount in 2..200u64, penalty_amount in 0..100u64) {
            let mut rng = OsRng;
            let (pk, vk) = generate_prepared_test_parameters::<ConvertCircuit>(&mut rng);

            let sk = rdsa::SigningKey::new_from_field(validator_randomness);
            let validator_identity = IdentityKey((&sk).into());
            let unbonding_amount = Amount::from(value1_amount);

            let start_epoch_index = 1;
            let unbonding_token = UnbondingToken::new(validator_identity, start_epoch_index);
            let unbonding_id = unbonding_token.id();
            let penalty = Penalty::from_percent(penalty_amount);
            let balance = penalty.balance_for_claim(unbonding_id, unbonding_amount);
            let balance_commitment = balance.commit(balance_blinding);

            let public = UndelegateClaimProofPublic { balance_commitment, unbonding_id, penalty };
            let private = UndelegateClaimProofPrivate { unbonding_amount, balance_blinding };

            let blinding_r = Fq::rand(&mut rng);
            let blinding_s = Fq::rand(&mut rng);
            let proof = UndelegateClaimProof::prove(
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
