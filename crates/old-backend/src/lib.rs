#[cfg(test)]
mod tests {
    use penumbra_proof_params::{
        DummyWitness, SPEND_PROOF_PROVING_KEY, generate_constraint_matrices, generate_test_parameters
    };
    use penumbra_shielded_pool::{SpendCircuit, OutputCircuit, ConvertCircuit, NullifierDerivationCircuit};
    use penumbra_dex::{swap::proof::SwapCircuit, swap_claim::proof::SwapClaimCircuit};
    use rand_core::OsRng;
    use rand_core::CryptoRngCore;

    #[test]
    fn test_generate_parameters() {
        generate_constraint_matrices::<SpendCircuit>();
        generate_constraint_matrices::<OutputCircuit>();
        generate_constraint_matrices::<SwapCircuit>();
        generate_constraint_matrices::<SwapClaimCircuit>();
        generate_constraint_matrices::<ConvertCircuit>();
        generate_constraint_matrices::<NullifierDerivationCircuit>();
    }

    #[test]
    fn test_serialization_comparison() {
        let mut rng = OsRng;
        let (pk, vk) = generate_test_parameters::<SpendCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<OutputCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<SwapCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<SwapClaimCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<ConvertCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<NullifierDerivationCircuit>(&mut rng);
    }
}