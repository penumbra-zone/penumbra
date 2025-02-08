use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_sdk_view::Planner;
use rand_core::OsRng;
use serde_json::Value;

use penumbra_sdk_governance::{
    ValidatorVote, ValidatorVoteBody, ValidatorVoteReason, Vote, MAX_VALIDATOR_VOTE_REASON_LENGTH,
};
use penumbra_sdk_proto::{view::v1::GasPricesRequest, DomainType};
use penumbra_sdk_stake::{
    validator,
    validator::{Validator, ValidatorToml},
    FundingStream, FundingStreams, IdentityKey,
};

use crate::App;

use penumbra_sdk_fee::FeeTier;

#[derive(Debug, clap::Subcommand)]
pub enum ValidatorCmd {
    /// Display the validator identity key derived from this wallet's spend seed.
    Identity {
        /// Use Base64 encoding for the identity key, rather than the default of Bech32.
        #[clap(long)]
        base64: bool,
    },
    /// Display the validator's governance subkey derived from this wallet's governance seed.
    GovernanceKey {
        /// Use Base64 encoding for the governance key, rather than the default of Bech32.
        #[clap(long)]
        base64: bool,
    },
    /// Manage your validator's definition.
    #[clap(subcommand)]
    Definition(DefinitionCmd),
    /// Submit and sign votes in your capacity as a validator.
    #[clap(subcommand)]
    Vote(VoteCmd),
}

#[derive(Debug, clap::Subcommand)]
pub enum VoteCmd {
    /// Cast a vote on a proposal in your capacity as a validator (see also: `pcli tx vote` for
    /// delegator voting).
    Cast {
        /// Optional. Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", global = true, display_order = 300)]
        source: u32,
        /// The vote to cast.
        #[clap(subcommand)]
        vote: super::tx::VoteCmd,
        /// A comment or justification of the vote. Limited to 1 KB.
        #[clap(long, default_value = "", global = true, display_order = 400)]
        reason: String,
        /// Use an externally-provided signature to authorize the vote.
        ///
        /// This is useful for offline signing, e.g. in an airgap setup. The signature for the
        /// vote may be generated using the `pcli validator vote sign` command.
        #[clap(long, global = true, display_order = 500)]
        signature: Option<String>,
        /// Vote on behalf of a particular validator.
        ///
        /// This must be specified when the custody backend does not match the validator identity
        /// key, i.e. when using a separate governance key on another wallet.
        #[clap(long, global = true, display_order = 600)]
        validator: Option<IdentityKey>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Sign a vote on a proposal in your capacity as a validator, for submission elsewhere.
    Sign {
        /// The vote to sign.
        #[clap(subcommand)]
        vote: super::tx::VoteCmd,
        /// A comment or justification of the vote. Limited to 1 KB.
        #[clap(long, default_value = "", global = true, display_order = 400)]
        reason: String,
        /// The file to write the signature to [default: stdout].
        #[clap(long, global = true, display_order = 500)]
        signature_file: Option<String>,
        /// Vote on behalf of a particular validator.
        ///
        /// This must be specified when the custody backend does not match the validator identity
        /// key, i.e. when using a separate governance key on another wallet.
        #[clap(long, global = true, display_order = 600)]
        validator: Option<IdentityKey>,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum DefinitionCmd {
    /// Submit a ValidatorDefinition transaction to create or update a validator.
    Upload {
        /// The TOML file containing the ValidatorDefinition to upload.
        #[clap(long)]
        file: String,
        /// Optional. Only spend funds originally received by the given account.
        #[clap(long, default_value = "0")]
        source: u32,
        /// Use an externally-provided signature to authorize the validator definition.
        ///
        /// This is useful for offline signing, e.g. in an airgap setup. The signature for the
        /// definition may be generated using the `pcli validator definition sign` command.
        #[clap(long)]
        signature: Option<String>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Sign a validator definition offline for submission elsewhere.
    Sign {
        /// The TOML file containing the ValidatorDefinition to sign.
        #[clap(long)]
        file: String,
        /// The file to write the signature to [default: stdout].
        #[clap(long)]
        signature_file: Option<String>,
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
    /// Fetches the definition for your validator
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
            ValidatorCmd::GovernanceKey { .. } => true,
            ValidatorCmd::Definition(
                DefinitionCmd::Template { .. } | DefinitionCmd::Sign { .. },
            ) => true,
            ValidatorCmd::Definition(
                DefinitionCmd::Upload { .. } | DefinitionCmd::Fetch { .. },
            ) => false,
            ValidatorCmd::Vote(VoteCmd::Sign { .. }) => true,
            ValidatorCmd::Vote(VoteCmd::Cast { .. }) => false,
        }
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let fvk = app.config.full_viewing_key.clone();

        match self {
            ValidatorCmd::Identity { base64 } => {
                let ik = IdentityKey(fvk.spend_verification_key().clone().into());

                if *base64 {
                    use base64::{display::Base64Display, engine::general_purpose::STANDARD};
                    println!("{}", Base64Display::new(ik.0.as_ref(), &STANDARD));
                } else {
                    println!("{ik}");
                }
            }
            ValidatorCmd::GovernanceKey { base64 } => {
                let gk = app.config.governance_key();

                if *base64 {
                    use base64::{display::Base64Display, engine::general_purpose::STANDARD};
                    println!("{}", Base64Display::new(&gk.0.to_bytes(), &STANDARD));
                } else {
                    println!("{gk}");
                }
            }
            ValidatorCmd::Definition(DefinitionCmd::Sign {
                file,
                signature_file,
            }) => {
                let new_validator = read_validator_toml(file)?;

                let input_file_path = std::fs::canonicalize(file)
                    .with_context(|| format!("invalid path: {file:?}"))?;
                let input_file_name = input_file_path
                    .file_name()
                    .with_context(|| format!("invalid path: {file:?}"))?;

                let signature = app.sign_validator_definition(new_validator.clone()).await?;

                if let Some(output_file) = signature_file {
                    let output_file_path = std::fs::canonicalize(output_file)
                        .with_context(|| format!("invalid path: {output_file:?}"))?;
                    let output_file_name = output_file_path
                        .file_name()
                        .with_context(|| format!("invalid path: {output_file:?}"))?;
                    File::create(output_file)
                        .with_context(|| format!("cannot create file {output_file:?}"))?
                        .write_all(URL_SAFE.encode(signature.encode_to_vec()).as_bytes())
                        .with_context(|| format!("could not write file {output_file:?}"))?;
                    println!(
                        "Signed validator definition #{} for {}\nWrote signature to {output_file_path:?}",
                        new_validator.sequence_number,
                        new_validator.identity_key,
                    );
                    println!(
                        "To upload the definition, use the below command with the exact same definition file:\n\n  $ pcli validator definition upload --file {:?} --signature - < {:?}",
                        input_file_name,
                        output_file_name,
                    );
                } else {
                    println!(
                        "Signed validator definition #{} for {}\nTo upload the definition, use the below command with the exact same definition file:\n\n  $ pcli validator definition upload --file {:?} \\\n      --signature {}",
                        new_validator.sequence_number,
                        new_validator.identity_key,
                        input_file_name,
                        URL_SAFE.encode(signature.encode_to_vec())
                    );
                }
            }
            ValidatorCmd::Definition(DefinitionCmd::Upload {
                file,
                source,
                signature,
                fee_tier,
            }) => {
                let gas_prices = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .gas_prices(GasPricesRequest {})
                    .await?
                    .into_inner()
                    .gas_prices
                    .expect("gas prices must be available")
                    .try_into()?;

                let new_validator = read_validator_toml(file)?;

                // Sign the validator definition with the wallet's spend key, or instead attach the
                // provided signature if present.
                let auth_sig = if let Some(signature) = signature {
                    // The user can specify `-` to read the signature from stdin.
                    let mut signature = signature.clone();
                    if signature == "-" {
                        let mut buf = String::new();
                        std::io::stdin().read_to_string(&mut buf)?;
                        signature = buf;
                    }
                    <Signature<SpendAuth> as penumbra_sdk_proto::DomainType>::decode(
                        &URL_SAFE
                            .decode(signature)
                            .context("unable to decode signature as base64")?[..],
                    )
                    .context("unable to parse decoded signature")?
                } else {
                    app.sign_validator_definition(new_validator.clone()).await?
                };
                let vd = validator::Definition {
                    validator: new_validator,
                    auth_sig,
                };
                // Construct a new transaction and include the validator definition.

                let plan = Planner::new(OsRng)
                    .validator_definition(vd)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .plan(app.view(), source.into())
                    .await?;

                app.build_and_submit_transaction(plan).await?;
                // Only commit the state if the transaction was submitted
                // successfully, so that we don't store pending notes that will
                // never appear on-chain.
                println!("Uploaded validator definition");
            }
            ValidatorCmd::Vote(VoteCmd::Sign {
                vote,
                reason,
                signature_file,
                validator,
            }) => {
                let identity_key = validator
                    .unwrap_or_else(|| IdentityKey(fvk.spend_verification_key().clone().into()));
                let governance_key = app.config.governance_key();

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

                let signature = app.sign_validator_vote(body).await?;

                if let Some(signature_file) = signature_file {
                    File::create(signature_file)
                        .with_context(|| format!("cannot create file {signature_file:?}"))?
                        .write_all(URL_SAFE.encode(signature.encode_to_vec()).as_bytes())
                        .context("could not write file")?;
                    let output_file_path = std::fs::canonicalize(signature_file)
                        .with_context(|| format!("invalid path: {signature_file:?}"))?;
                    println!(
                        "Signed validator vote {vote} on proposal #{proposal} by {identity_key}\nWrote signature to {output_file_path:?}",
                    );
                    println!(
                        "To cast the vote, use the below command:\n\n  $ pcli validator vote cast {vote} --on {proposal} --reason {reason:?} --signature - < {signature_file:?}",
                    );
                } else {
                    println!(
                        "Signed validator vote {vote} on proposal #{proposal} by {identity_key}\nTo cast the vote, use the below command:\n\n  $ pcli validator vote cast {vote} --on {proposal} --reason {reason:?} \\\n      --signature {}",
                        URL_SAFE.encode(signature.encode_to_vec())
                    );
                }
            }
            ValidatorCmd::Vote(VoteCmd::Cast {
                source,
                vote,
                reason,
                signature,
                validator,
                fee_tier,
            }) => {
                let gas_prices = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .gas_prices(GasPricesRequest {})
                    .await?
                    .into_inner()
                    .gas_prices
                    .expect("gas prices must be available")
                    .try_into()?;

                let identity_key = validator
                    .unwrap_or_else(|| IdentityKey(fvk.spend_verification_key().clone().into()));
                let governance_key = app.config.governance_key();

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

                // If the user specified a signature, use it. Otherwise, generate a new signature
                // using local custody
                let auth_sig = if let Some(signature) = signature {
                    // The user can specify `-` to read the signature from stdin.
                    let mut signature = signature.clone();
                    if signature == "-" {
                        let mut buf = String::new();
                        std::io::stdin().read_to_string(&mut buf)?;
                        signature = buf;
                    }
                    <Signature<SpendAuth> as penumbra_sdk_proto::DomainType>::decode(
                        &URL_SAFE
                            .decode(signature)
                            .context("unable to decode signature as base64")?[..],
                    )
                    .context("unable to parse decoded signature")?
                } else {
                    app.sign_validator_vote(body.clone()).await?
                };

                let vote = ValidatorVote { body, auth_sig };

                // Construct a new transaction and include the validator definition.
                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .validator_vote(vote)
                    .plan(app.view(), source.into())
                    .await?;

                app.build_and_submit_transaction(plan).await?;

                println!("Cast validator vote");
            }
            ValidatorCmd::Definition(DefinitionCmd::Template {
                file,
                tendermint_validator_keyfile,
            }) => {
                let (address, _dtk) = fvk.incoming().payment_address(0u32.into());
                let identity_key = IdentityKey(fvk.spend_verification_key().clone().into());
                // By default, the template sets the governance key to the same verification key as
                // the identity key, but a validator can change this if they want to use different
                // key material.
                let governance_key = app.config.governance_key();

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
                let identity_key = IdentityKey(fvk.spend_verification_key().clone().into());
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

/// Parse a validator definition TOML file and return the parsed definition.
fn read_validator_toml(file: &str) -> Result<Validator> {
    let mut definition_file =
        File::open(file).with_context(|| format!("cannot open file {file:?}"))?;
    let mut definition: String = String::new();
    definition_file
        .read_to_string(&mut definition)
        .with_context(|| format!("failed to read file {file:?}"))?;
    let new_validator: ValidatorToml =
        toml::from_str(&definition).context("unable to parse validator definition")?;
    let new_validator: Validator = new_validator
        .try_into()
        .context("unable to parse validator definition")?;
    Ok(new_validator)
}
