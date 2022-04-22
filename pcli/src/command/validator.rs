use std::{fs::File, io::Write};

use anyhow::{Context, Result};
use futures::TryStreamExt;
use penumbra_proto::{stake::Validator as ProtoValidator, Message};
use penumbra_stake::{
    FundingStream, FundingStreams, IdentityKey, Validator, ValidatorDefinition, ValidatorInfo,
};
use rand_core::OsRng;
use structopt::StructOpt;

use crate::{state::ClientStateFile, Opt};

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
    /// Generates a template validator definition for editing.
    ///
    /// The validator identity field will be prepopulated with the validator
    /// identity key derived from this wallet's seed phrase.
    TemplateDefinition {
        /// The JSON file to write the template to.
        #[structopt(long)]
        file: String,
    },
    /// Fetches a validator's current definition and saves it to a file.
    FetchDefinition {
        /// The JSON file to write the template to.
        #[structopt(long)]
        file: String,
        /// The identity key of the validator to fetch.
        identity_key: String,
    },
}

impl ValidatorCmd {
    pub fn needs_sync(&self) -> bool {
        match self {
            ValidatorCmd::Identity => false,
            ValidatorCmd::UploadDefinition { .. } => true,
            ValidatorCmd::TemplateDefinition { .. } => false,
            ValidatorCmd::FetchDefinition { .. } => false,
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
                let transaction =
                    state.build_validator_definition(&mut OsRng, vd, *fee, *source)?;

                opt.submit_transaction(&transaction).await?;
                // Only commit the state if the transaction was submitted
                // successfully, so that we don't store pending notes that will
                // never appear on-chain.
                state.commit()?;
                println!("Uploaded validator definition");
            }
            ValidatorCmd::TemplateDefinition { file } => {
                let (_label, address) = state.wallet().address_by_index(0)?;
                let identity_key = IdentityKey(
                    state
                        .wallet()
                        .full_viewing_key()
                        .spend_verification_key()
                        .clone(),
                );
                // Generate a random consensus key.
                // TODO: not great because the private key is discarded here and this isn't obvious to the user
                let consensus_key =
                    tendermint::PrivateKey::Ed25519(ed25519_consensus::SigningKey::new(OsRng))
                        .public_key();

                let template = Validator {
                    identity_key,
                    consensus_key,
                    name: String::new(),
                    website: String::new(),
                    description: String::new(),
                    funding_streams: FundingStreams::try_from(vec![FundingStream {
                        address,
                        rate_bps: 100,
                    }])?,
                    sequence_number: 0,
                };

                File::create(file)
                    .with_context(|| format!("cannot create file {:?}", file))?
                    .write_all(&serde_json::to_vec_pretty(&template)?)
                    .context("could not write file")?;
            }
            ValidatorCmd::FetchDefinition { file, identity_key } => {
                let identity_key = identity_key.parse::<IdentityKey>()?;

                /*
                use penumbra_proto::client::specific::ValidatorStatusRequest;

                let mut client = opt.specific_client().await?;
                let status: ValidatorStatus = client
                    .validator_status(ValidatorStatusRequest {
                        chain_id: "".to_string(), // TODO: fill in
                        identity_key: Some(identity_key.into()),
                    })
                    .await?
                    .into_inner()
                    .try_into()?;

                // why isn't the validator definition part of the status?
                // why do we have all these different validator messages?
                // do we need them?
                status.state.
                */

                // Intsead just download everything
                let mut client = opt.oblivious_client().await?;

                use penumbra_proto::client::oblivious::ValidatorInfoRequest;
                let validators = client
                    .validator_info(ValidatorInfoRequest {
                        show_inactive: true,
                        chain_id: state.chain_id().unwrap_or_default(),
                    })
                    .await?
                    .into_inner()
                    .try_collect::<Vec<_>>()
                    .await?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<ValidatorInfo>, _>>()?;

                let validator = validators
                    .iter()
                    .map(|info| &info.validator)
                    .find(|v| v.identity_key == identity_key)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Could not find validator {}", identity_key))?;

                File::create(file)
                    .with_context(|| format!("cannot create file {:?}", file))?
                    .write_all(&serde_json::to_vec_pretty(&validator)?)
                    .context("could not write file")?;
            }
        }

        Ok(())
    }
}
