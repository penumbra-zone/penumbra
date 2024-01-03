use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::{Context, Result};
use rand_core::OsRng;
use serde_json::Value;

use penumbra_fee::Fee;
use penumbra_governance::{
    ValidatorVote, ValidatorVoteBody, ValidatorVoteReason, Vote, MAX_VALIDATOR_VOTE_REASON_LENGTH,
};
use penumbra_keys::keys::AddressIndex;
use penumbra_proto::{
    core::component::stake::v1alpha1::Validator as ProtoValidator, DomainType, Message,
};
use penumbra_stake::{
    validator,
    validator::{Validator, ValidatorToml},
    FundingStream, FundingStreams, GovernanceKey, IdentityKey,
};
use penumbra_wallet::plan;

use crate::{config::CustodyConfig, App};

#[derive(Debug, clap::Subcommand)]
pub enum ValidatorCmd {
    /// Display the validator identity key derived from this wallet's spend seed.
    Identity {
        /// Use Base64 encoding for the identity key, rather than the default of Bech32.
        #[clap(long)]
        base64: bool,
    },
    /// Manage your validator's definition.
    #[clap(subcommand)]
    Definition(DefinitionCmd),
    /// Cast a vote on a proposal in your capacity as a validator (see also: `pcli tx vote`).
    Vote {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0", global = true, display_order = 200)]
        fee: u64,
        /// Optional. Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", global = true, display_order = 300)]
        source: u32,
        /// The vote to cast.
        #[clap(subcommand)]
        vote: super::tx::VoteCmd,
        /// A comment or justification of the vote. Limited to 1 KB.
        #[clap(long, default_value = "", global = true, display_order = 400)]
        reason: String,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum DefinitionCmd {
    /// Create a ValidatorDefinition transaction to create or update a validator.
    Upload {
        /// The TOML file containing the ValidatorDefinition to upload.
        #[clap(long)]
        file: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
    },
    /// Generates a template validator definition for editing.
    ///
    /// The validator identity field will be prepopulated with the validator
    /// identity key derived from this wallet's seed phrase.
    Template {
        /// The TOML file to write the template to [default: stdout].
        #[clap(long)]
        file: Option<String>,

        /// The Tendermint JSON file 'priv_validator_key.json', containing
        /// the consensus key for the validator identity. If provided,
        /// the key will be used in the generated validator template.
        /// If not provided, a random key will be inserted.
        #[clap(short = 'k', long)]
        tendermint_validator_keyfile: Option<camino::Utf8PathBuf>,
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
            ValidatorCmd::Identity { .. } => true,
            ValidatorCmd::Definition(DefinitionCmd::Upload { .. }) => false,
            ValidatorCmd::Definition(
                DefinitionCmd::Template { .. } | DefinitionCmd::Fetch { .. },
            ) => true,
            ValidatorCmd::Vote { .. } => false,
        }
    }

    // TODO: move use of sk into custody service
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let sk = match &app.config.custody {
            CustodyConfig::SoftKms(config) => config.spend_key.clone(),
            _ => {
                anyhow::bail!("Validator commands require SoftKMS backend");
            }
        };
        let fvk = app.config.full_viewing_key.clone();

        match self {
            ValidatorCmd::Identity { base64 } => {
                let ik = IdentityKey(fvk.spend_verification_key().clone());

                if *base64 {
                    use base64::{display::Base64Display, engine::general_purpose::STANDARD};
                    println!("{}", Base64Display::new(&ik.0.to_bytes(), &STANDARD));
                } else {
                    println!("{ik}");
                }
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
                let mut definition_file =
                    File::open(file).with_context(|| format!("cannot open file {file:?}"))?;
                let mut definition: String = String::new();
                definition_file
                    .read_to_string(&mut definition)
                    .with_context(|| format!("failed to read file {file:?}"))?;
                let new_validator: ValidatorToml =
                    toml::from_str(&definition).context("Unable to parse validator definition")?;
                let new_validator: Validator = new_validator
                    .try_into()
                    .context("Unable to parse validator definition")?;
                let fee = Fee::from_staking_token_amount((*fee).into());

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
                    app.view
                        .as_mut()
                        .context("view service must be initialized")?,
                    OsRng,
                    vd,
                    fee,
                    AddressIndex::new(*source),
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
                source,
                vote,
                reason,
            } => {
                // TODO: support submitting a separate governance key.
                let identity_key = IdentityKey(*sk.full_viewing_key().spend_verification_key());
                // Currently this is always just copied from the identity key
                let governance_key = GovernanceKey(identity_key.0);

                let (proposal, vote): (u64, Vote) = (*vote).into();

                if reason.len() > MAX_VALIDATOR_VOTE_REASON_LENGTH {
                    anyhow::bail!("validator vote reason is too long, max 1024 bytes");
                }

                // Construct the vote body
                let body = ValidatorVoteBody {
                    proposal,
                    vote,
                    identity_key,
                    governance_key,
                    reason: ValidatorVoteReason(reason.clone()),
                };

                // TODO: support signing with a separate governance key
                let governance_auth_key = sk.spend_auth_key();

                // Generate an authorizing signature with the governance key for the vote body
                let body_bytes = body.encode_to_vec();
                let auth_sig = governance_auth_key.sign(OsRng, &body_bytes);

                let vote = ValidatorVote { body, auth_sig };

                // Construct a new transaction and include the validator definition.
                let fee = Fee::from_staking_token_amount((*fee).into());

                let plan = plan::validator_vote(
                    app.view
                        .as_mut()
                        .context("view service must be initialized")?,
                    OsRng,
                    vote,
                    fee,
                    AddressIndex::new(*source),
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;

                println!("Cast validator vote");
            }
            ValidatorCmd::Definition(DefinitionCmd::Template {
                file,
                tendermint_validator_keyfile,
            }) => {
                let (address, _dtk) = fvk.incoming().payment_address(0u32.into());
                let identity_key = IdentityKey(fvk.spend_verification_key().clone());
                // By default, the template sets the governance key to the same verification key as
                // the identity key, but a validator can change this if they want to use different
                // key material.
                let governance_key = GovernanceKey(identity_key.0);

                // Honor the filepath to `priv_validator_key.json`, if set. Otherwise, generate
                // a random pubkey and emit a warning about it.
                let consensus_key: tendermint::PublicKey = match tendermint_validator_keyfile {
                    Some(f) => {
                        tracing::debug!(?f, "Reading tendermint validator pubkey from file");
                        let tm_key_config: Value =
                            serde_json::from_str(&std::fs::read_to_string(f)?).context(format!(
                                "Could not parse file as Tendermint validator config: {f}"
                            ))?;
                        serde_json::value::from_value::<tendermint::PublicKey>(
                            tm_key_config["pub_key"].clone(),
                        )
                        .context(format!("Tendermint JSON file malformed: {f}"))?
                    }
                    None => {
                        tracing::warn!("Generating a random consensus pubkey for Tendermint; consider using the '--tendermint-validator-keyfile' flag");
                        generate_new_tendermint_keypair()?.public_key()
                    }
                };

                // Customize the human-readable comment text in the definition.
                let generated_key_notice: String = match tendermint_validator_keyfile {
                    Some(_s) => String::from(""),
                    None => {
                        "\n# The consensus_key field is random, and needs to be replaced with your
# tendermint instance's public key, which can be found in `priv_validator_key.json`.
#"
                        .to_string()
                    }
                };

                let template: ValidatorToml = Validator {
                    identity_key,
                    governance_key,
                    consensus_key,
                    name: String::new(),
                    website: String::new(),
                    description: String::new(),
                    // Default enabled to "false" so operators are required to manually
                    // enable their validators when ready.
                    enabled: false,
                    funding_streams: FundingStreams::try_from(vec![
                        FundingStream::ToAddress {
                            address,
                            rate_bps: 100,
                        },
                        FundingStream::ToCommunityPool { rate_bps: 100 },
                    ])?,
                    sequence_number: 0,
                }
                .into();

                let template_str = format!(
                    "# This is a template for a validator definition.
#
# The identity_key and governance_key fields are auto-filled with values derived
# from this wallet's account.
# {}
# You should fill in the name, website, and description fields.
#
# By default, validators are disabled, and cannot be delegated to. To change
# this, set `enabled = true`.
#
# Every time you upload a new validator config, you'll need to increment the
# `sequence_number`.

{}
",
                    generated_key_notice,
                    toml::to_string_pretty(&template)?
                );

                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {file:?}"))?
                        .write_all(template_str.as_bytes())
                        .context("could not write file")?;
                } else {
                    println!("{}", &template_str);
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

/// Generate a new ED25519 keypair for use with Tendermint.
fn generate_new_tendermint_keypair() -> anyhow::Result<tendermint::PrivateKey> {
    let signing_key = ed25519_consensus::SigningKey::new(OsRng);
    let slice_signing_key = signing_key.as_bytes().as_slice();
    let priv_consensus_key = tendermint::PrivateKey::Ed25519(slice_signing_key.try_into()?);
    Ok(priv_consensus_key)
}
