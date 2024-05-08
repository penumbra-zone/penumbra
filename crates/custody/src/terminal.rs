use anyhow::Result;
use penumbra_governance::ValidatorVoteBody;
use penumbra_stake::validator::Validator;
use penumbra_transaction::TransactionPlan;
use tonic::async_trait;

#[derive(Debug, Clone)]
pub enum SigningRequest {
    TransactionPlan(TransactionPlan),
    ValidatorDefinition(Validator),
    ValidatorVote(ValidatorVoteBody),
}
/// A trait abstracting over the kind of terminal interface we expect.
///
/// This is mainly used to accommodate the kind of interaction we have with the CLI
/// interface, but it can also be plugged in with more general backends.
#[async_trait]
pub trait Terminal {
    /// Have a user confirm that they want to sign this transaction or other data (e.g. validator
    /// definition, validator vote)
    ///
    /// In an actual terminal, this should display the data to be signed in a human readable
    /// form, and then get feedback from the user.
    async fn confirm_request(&self, request: &SigningRequest) -> Result<bool>;

    /// Push an explanatory message to the terminal.
    ///
    /// This message has no relation to the actual protocol, it just allows explaining
    /// what subsequent data means, and what the user needs to do.
    ///
    /// Backends can replace this with a no-op.
    async fn explain(&self, msg: &str) -> Result<()>;

    /// Broadcast a message to other users.
    async fn broadcast(&self, data: &str) -> Result<()>;

    /// Wait for a response from *some* other user, it doesn't matter which.
    ///
    /// This function should not return None spuriously, when it does,
    /// it should continue to return None until a message is broadcast.
    async fn next_response(&self) -> Result<Option<String>>;

    /// Wait for the user to supply a password.
    async fn get_password(&self) -> Result<String>;
}
