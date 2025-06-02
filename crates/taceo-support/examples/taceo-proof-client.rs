use std::{fs::File, path::PathBuf, time::Duration};

// use ark_bls12_377::Bls12_377;
// use ark_bls12_381::Bls12_381;
// use ark_bn254::Bn254;
use ark_ec::pairing::Pairing;
use ark_groth16::VerifyingKey;
use ark_serialize::CanonicalDeserialize;
use circom_types::{
    groth16::JsonVerificationKey,
    traits::{CircomArkworksPairingBridge, CircomArkworksPrimeFieldBridge},
    Witness, R1CS,
};
use clap::{ArgGroup, Parser, ValueEnum};
use co_groth16::CoGroth16;
use taceo_proof_api_client::apis::configuration::Configuration;
use taceo_support::{
    constraint_synthesis::{generate_circuit_constraints, generate_valid_spend_inputs},
    JobResult,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum JobType {
    Rep3Full,
    Rep3Prove,
    ShamirProve,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Curve {
    Bn254,
    Bls381,
    Bls377,
}

#[derive(Parser, Debug)]
#[clap(group(
    ArgGroup::new("exclusive_args")
        .args(&["r1cs", "num_inputs"])
        .required(true)
))]
struct Args {
    /// The job type
    pub job: JobType,

    /// The API endpoint URL
    #[clap(long, env = "PROOF_API_URL", default_value = "http://localhost:1234")]
    pub api_url: String,

    /// The curve
    #[clap(long, env = "PROOF_CURVE")]
    pub curve: Curve,

    /// The path to the job input
    #[clap(long, env = "PROOF_INPUT")]
    pub input: PathBuf,

    /// The ppd-network code
    #[clap(long, env = "PROOF_CODE")]
    pub code: String,

    /// The job blueprint
    #[clap(long, env = "PROOF_BLUEPRINT")]
    pub blueprint: Uuid,

    /// The path to the r1cs file
    #[clap(long, env = "PROOF_R1CS")]
    pub r1cs: Option<PathBuf>,

    /// The number of inputs for the circuit
    #[clap(long, env = "PROOF_NUM_INPUTS")]
    pub num_inputs: Option<usize>,

    /// The public inputs for witness extension
    #[clap(long, env = "PROOF_PUBLIC_INPUTS", required_if_eq("job", "rep3-full"))]
    pub public_inputs: Option<Vec<String>>,

    /// The path to verifying key
    #[clap(long, env = "PROOF_VERIFYING_KEY")]
    pub vk: Option<PathBuf>,
}

async fn run<P>(config: &Configuration, args: Args) -> eyre::Result<()>
where
    P: Pairing + CircomArkworksPairingBridge,
    P::ScalarField: CircomArkworksPrimeFieldBridge,
    P::BaseField: CircomArkworksPrimeFieldBridge,
{
    let num_inputs = if let Some(r1cs) = args.r1cs {
        let r1cs = R1CS::<P>::from_reader(File::open(r1cs)?)?;
        r1cs.num_inputs
    } else {
        args.num_inputs.expect("must be present if r1cs is not")
    };

    // schedule job
    tracing::info!("scheduling job...");
    let job_id = match args.job {
        JobType::Rep3Full => {
            let input = serde_json::from_reader(File::open(args.input)?)?;
            taceo_support::schedule_full_job_rep3::<P>(
                config,
                &args.code,
                args.blueprint,
                input,
                &args
                    .public_inputs
                    .expect("must be present if job is Rep3Full"),
            )
            .await?
        }
        JobType::Rep3Prove => {
            let witness = Witness::from_reader(File::open(args.input)?)?;
            taceo_support::schedule_prove_job_rep3::<P>(
                config,
                &args.code,
                args.blueprint,
                witness,
                num_inputs,
            )
            .await?
        }
        JobType::ShamirProve => {
            let witness = Witness::from_reader(File::open(args.input)?)?;
            taceo_support::schedule_prove_job_shamir::<P>(
                config,
                &args.code,
                args.blueprint,
                witness,
                num_inputs,
            )
            .await?
        }
    };

    // poll job status to get result
    tracing::info!("waiting for result...");
    loop {
        match taceo_support::get_job_result(config, job_id).await? {
            JobResult::Ok((proof, public_inputs)) => {
                tracing::info!("got proof");
                if let Some(vk) = args.vk {
                    let vk = if vk.extension().is_some_and(|ext| ext == "json") {
                        JsonVerificationKey::from_reader(File::open(vk)?)?.into()
                    } else {
                        VerifyingKey::<P>::deserialize_uncompressed_unchecked(File::open(vk)?)?
                    };
                    tracing::info!("verifying proof...");
                    CoGroth16::verify(&vk, &proof, &public_inputs)?;
                }
                break;
            }
            JobResult::Err(err) => {
                tracing::error!("got error: {err}");
                break;
            }
            JobResult::Running(_) => {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }

    Ok(())
}

fn install_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false).with_line_number(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let (public, private) = generate_valid_spend_inputs();
    println!("public: {:?}", public);
    println!("private: {:?}", private);

    generate_circuit_constraints(public, private);

    // install_tracing();
    // let args = Args::parse();
    // let config = Configuration {
    //     base_path: args.api_url.clone(),
    //     ..Default::default()
    // };

    // match args.curve {
    //     Curve::Bn254 => run::<Bn254>(&config, args).await?,
    //     Curve::Bls381 => run::<Bls12_381>(&config, args).await?,
    //     Curve::Bls377 => run::<Bls12_377>(&config, args).await?,
    // };

    Ok(())
}
