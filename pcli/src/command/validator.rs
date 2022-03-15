use std::fs::File;

use anyhow::Context;
use anyhow::Result;
use penumbra_stake::Validator;
use penumbra_stake::ValidatorDefinition;
use rand_core::OsRng;
use structopt::StructOpt;

use penumbra_proto::{stake::Validator as ProtoValidator, Message};
use penumbra_stake::IdentityKey;

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
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        source: Option<u64>,
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
            ValidatorCmd::UploadDefinition { file, fee, source } => {
                // The definitions are stored in a JSON document,
                // however for ease of use it's best for us to generate
                // the signature here based on the configured wallet.
                //
                // TODO: eventually we'll probably want to support defining the
                // identity key in the JSON file.
                //
                // We could also support defining multiple validators in a single
                // file.
                let definition_file =
                    File::open(&file).with_context(|| format!("cannot open file {:?}", file))?;
                let new_validator: Validator = serde_json::from_reader(definition_file)
                    .map_err(|_| anyhow::anyhow!("Unable to parse validator definition"))?;

                // Sign the validator definition with the wallet's spend key.
                let protobuf_serialized: ProtoValidator = new_validator.clone().into();
                let v_bytes = protobuf_serialized.encode_to_vec();
                let signing_key = state.wallet().spend_key().spend_auth_key().clone();
                let auth_sig = signing_key.sign(&mut OsRng, &v_bytes);
                let vd = ValidatorDefinition {
                    validator: new_validator,
                    auth_sig,
                };
                // Construct a new transaction and include the validator definition.
                // TODO: is it possible to get rid of this clone? It's only used because we can't
                // borrow state mutably & immutably.
                let transaction =
                    state.build_validator_definition(&mut OsRng, vd, *fee, *source)?;

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
