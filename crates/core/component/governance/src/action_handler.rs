use cnidarium_component::ActionHandler;

pub mod delegator_vote;
pub mod deposit_claim;
pub mod validator_vote;
pub mod withdraw;

// Note: The ProposalSubmit action handler is defined in `penumbra-app`
// due to it requiring knowledge of all other actions and the `TransactionPlan`,
// located in `penumbra-transaction`.
