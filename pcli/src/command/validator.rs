use std::fs::File;

use anyhow::Context;
use anyhow::Result;
use structopt::StructOpt;

use penumbra_stake::IdentityKey;
use penumbra_stake::ValidatorDefinition;
use penumbra_wallet::ClientState;

use crate::state::ClientStateFile;
use crate::Opt;

#[derive(Debug, StructOpt)]
pub enum ValidatorCmd {
    /// Display the validator identity key derived from this wallet's spend seed.
    Identity,
    /// Create a ValidatorDefinition transaction to create or update a validator.
    UploadDefinition {
        /// The JSON file containing the ValidatorDefinition to upload
        #[structopt(long)]
        file: String,
    },
}

impl ValidatorCmd {
    pub fn needs_sync(&self) -> bool {
        match self {
            ValidatorCmd::Identity => false,
            ValidatorCmd::UploadDefinition { .. } => true,
        }
    }

    pub async fn exec(&self, opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
        match self {
            ValidatorCmd::Identity => {
                let ik = IdentityKey(
                    state
                        .wallet()
                        .full_viewing_key()
                        .spend_verification_key()
                        .clone(),
                );

                println!("{}", ik);
            }
            ValidatorCmd::UploadDefinition { file } => {
                let definition_file =
                    File::open(&file).with_context(|| format!("cannot open file {:?}", file))?;
                let definition: ValidatorDefinition = serde_json::from_reader(definition_file)
                    .expect("can parse app_state in genesis file");
                // Construct a new transaction and include the validator definition.
                let transaction = state.build_validator_definition(definition)?;

                opt.submit_transaction(&transaction).await?;
                // Only commit the state if the transaction was submitted
                // successfully, so that we don't store pending notes that will
                // never appear on-chain.
                state.commit()?;
                println!("Uploaded validator definition");
            }
        }

        Ok(())
    }
}
