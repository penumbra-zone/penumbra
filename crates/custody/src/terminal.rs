use anyhow::Result;
use penumbra_sdk_governance::ValidatorVoteBody;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_stake::validator::Validator;
use penumbra_sdk_transaction::TransactionPlan;
use serde::de::DeserializeOwned;
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
pub trait Terminal: Sync {
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
    fn explain(&self, msg: &str) -> Result<()>;

    /// Broadcast a message to other users.
    async fn broadcast(&self, data: &str) -> Result<()>;

    /// Try to read a typed message from the terminal, retrying until
    /// the message parses successfully or the user interrupts the program.
    async fn next_response<D>(&self) -> Result<D>
    where
        D: DomainType,
        anyhow::Error: From<<D as TryFrom<<D as DomainType>::Proto>>::Error>,
        <D as DomainType>::Proto: DeserializeOwned,
    {
        loop {
            // Read a line or bubble up an error if we couldn't.
            let line = self.read_line_raw().await?;

            let proto = match serde_json::from_str::<<D as DomainType>::Proto>(&line) {
                Ok(proto) => proto,
                Err(e) => {
                    self.explain(&format!("Error parsing response: {:#}", e))?;
                    self.explain("Please try again:")?;
                    continue;
                }
            };

            let message = match D::try_from(proto) {
                Ok(message) => message,
                Err(e) => {
                    let e: anyhow::Error = e.into();
                    self.explain(&format!("Error parsing response: {:#}", e))?;
                    self.explain("Please try again:")?;
                    continue;
                }
            };

            return Ok(message);
        }
    }

    /// Read a single line from the terminal.
    async fn read_line_raw(&self) -> Result<String>;

    /// Wait for the user to supply a password.
    async fn get_password(&self) -> Result<String>;
}
