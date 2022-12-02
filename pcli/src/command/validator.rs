use std::{fs::File, io::Write};

use anyhow::{Context, Result};
use penumbra_component::stake::{validator, validator::Validator, FundingStream, FundingStreams};
use penumbra_crypto::{transaction::Fee, GovernanceKey, IdentityKey};
use penumbra_proto::{core::stake::v1alpha1::Validator as ProtoValidator, Message, Protobuf};
use penumbra_transaction::action::{ValidatorVote, ValidatorVoteBody, Vote};
use penumbra_wallet::plan;
use rand_core::OsRng;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum ValidatorCmd {
    /// Display the validator identity key derived from this wallet's spend seed.
    Identity,
    /// Manage your validator's definition.
    #[clap(subcommand)]
    Definition(DefinitionCmd),
    /// Cast a vote on a proposal in your capacity as a validator.
    ///
    /// This is distinct from casting a vote as a delegator, which can be done using `pcli tx
    /// proposal vote`.
    Vote {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// The proposal id to vote on.
        #[clap(long = "on")]
        proposal_id: u64,
        /// The vote to cast.
        vote: Vote,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum DefinitionCmd {
    /// Create a ValidatorDefinition transaction to create or update a validator.
    Upload {
        /// The JSON file containing the ValidatorDefinition to upload.
        #[clap(long)]
        file: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
    /// Generates a template validator definition for editing.
    ///
    /// The validator identity field will be prepopulated with the validator
    /// identity key derived from this wallet's seed phrase.
    Template {
        /// The JSON file to write the template to [default: stdout].
        #[clap(long)]
        file: Option<String>,
    },
    /// Fetches the definition for your validator.
    Fetch {
        /// The JSON file to write the definition to [default: stdout].
        #[clap(long)]
        file: Option<String>,
    },
}

impl ValidatorCmd {
    pub fn offline(&self) -> bool {
        match self {
            ValidatorCmd::Identity => true,
            ValidatorCmd::Definition(DefinitionCmd::Upload { .. }) => false,
            ValidatorCmd::Definition(
                DefinitionCmd::Template { .. } | DefinitionCmd::Fetch { .. },
            ) => true,
            ValidatorCmd::Vote { .. } => false,
        }
    }

    // TODO: move use of sk into custody service
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let sk = app.wallet.spend_key.clone();
        let fvk = sk.full_viewing_key().clone();
        match self {
            ValidatorCmd::Identity => {
                let ik = IdentityKey(fvk.spend_verification_key().clone());

                println!("{}", ik);
            }
            ValidatorCmd::Definition(DefinitionCmd::Upload { file, fee, source }) => {
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
                    File::open(file).with_context(|| format!("cannot open file {:?}", file))?;
                let new_validator: Validator = serde_json::from_reader(definition_file)
                    .map_err(|_| anyhow::anyhow!("Unable to parse validator definition"))?;
                let fee = Fee::from_staking_token_amount((*fee as u64).into());

                // Sign the validator definition with the wallet's spend key.
                let protobuf_serialized: ProtoValidator = new_validator.clone().into();
                let v_bytes = protobuf_serialized.encode_to_vec();
                let auth_sig = sk.spend_auth_key().sign(OsRng, &v_bytes);
                let vd = validator::Definition {
                    validator: new_validator,
                    auth_sig,
                };
                // Construct a new transaction and include the validator definition.
                let plan = plan::validator_definition(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    vd,
                    fee,
                    *source,
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;
                // Only commit the state if the transaction was submitted
                // successfully, so that we don't store pending notes that will
                // never appear on-chain.
                println!("Uploaded validator definition");
            }
            ValidatorCmd::Vote {
                fee,
                proposal_id,
                source,
                vote,
            } => {
                // TODO: support submitting a separate governance key.
                let identity_key = IdentityKey(*sk.full_viewing_key().spend_verification_key());
                // Currently this is always just copied from the identity key
                let governance_key = GovernanceKey(identity_key.0);

                // Construct the vote body
                let body = ValidatorVoteBody {
                    proposal: *proposal_id,
                    vote: *vote,
                    identity_key,
                    governance_key,
                };

                // TODO: support signing with a separate governance key
                let governance_auth_key = sk.spend_auth_key();

                // Generate an authorizing signature with the governance key for the vote body
                let body_bytes = body.encode_to_vec();
                let auth_sig = governance_auth_key.sign(OsRng, &body_bytes);

                let vote = ValidatorVote { body, auth_sig };

                // Construct a new transaction and include the validator definition.
                let fee = Fee::from_staking_token_amount((*fee as u64).into());
                let plan = plan::validator_vote(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    vote,
                    fee,
                    *source,
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;

                println!("Cast validator vote");
            }
            ValidatorCmd::Definition(DefinitionCmd::Template { file }) => {
                let (address, _dtk) = fvk.incoming().payment_address(0u64.into());
                let identity_key = IdentityKey(fvk.spend_verification_key().clone());
                // By default, the template sets the governance key to the same verification key as
                // the identity key, but a validator can change this if they want to use different
                // key material.
                let governance_key = GovernanceKey(identity_key.0);
                // Generate a random consensus key.
                // TODO: not great because the private key is discarded here and this isn't obvious to the user
                let consensus_key = ed25519_consensus::SigningKey::new(OsRng);

                /* MAKESHIFT RAFT ZONE */
                let signing_key_bytes = consensus_key.as_bytes().as_slice();
                let verification_key_bytes = consensus_key.verification_key();
                let verification_key_bytes = verification_key_bytes.as_bytes().as_slice();
                // TODO(erwan): surely there's a better way to do this?
                let mut keypair = [verification_key_bytes, signing_key_bytes].concat();
                let keypair = keypair.as_slice();

                let consensus_key = ed25519_dalek::Keypair::from_bytes(keypair.as_ref()).unwrap();
                /* END */

                let consensus_key = tendermint::PrivateKey::Ed25519(consensus_key).public_key();

                let template = Validator {
                    identity_key,
                    governance_key,
                    consensus_key,
                    name: String::new(),
                    website: String::new(),
                    description: String::new(),
                    // Default enabled to "false" so operators are required to manually
                    // enable their validators when ready.
                    enabled: false,
                    funding_streams: FundingStreams::try_from(vec![FundingStream {
                        address,
                        rate_bps: 100,
                    }])?,
                    sequence_number: 0,
                };

                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {:?}", file))?
                        .write_all(&serde_json::to_vec_pretty(&template)?)
                        .context("could not write file")?;
                } else {
                    println!("{}", serde_json::to_string_pretty(&template)?);
                }
            }
            ValidatorCmd::Definition(DefinitionCmd::Fetch { file }) => {
                let identity_key = IdentityKey(fvk.spend_verification_key().clone());
                super::query::ValidatorCmd::Definition {
                    file: file.clone(),
                    identity_key: identity_key.to_string(),
                }
                .exec(app)
                .await?;
            }
        }

        Ok(())
    }
}
