use ark_ec::pairing::Pairing;
use ark_serialize::CanonicalDeserialize;
use base64ct::{Base64, Encoding};
use circom_types::{
    traits::{CircomArkworksPairingBridge, CircomArkworksPrimeFieldBridge},
    Witness,
};
use co_circom_types::{
    split_input, CompressedRep3SharedWitness, Compression, Input, ShamirSharedWitness,
};
use co_groth16::Proof;
use crypto_box::{aead::OsRng, PublicKey};
use eyre::Context;
use taceo_proof_api_client::{
    apis::{configuration::Configuration, job_api},
    models::{JobStatus, JobType, ScheduleJobRequest, ScheduleJobResponse},
};
use uuid::Uuid;
pub mod constraint_synthesis;

async fn schedule_job(
    config: &Configuration,
    code: &str,
    blueprint_id: Uuid,
    job_type: JobType,
) -> eyre::Result<ScheduleJobResponse> {
    Ok(job_api::schedule_job(
        config,
        ScheduleJobRequest::new(blueprint_id, code.to_string(), job_type),
    )
    .await?)
}

async fn add_input(
    config: &Configuration,
    res: &ScheduleJobResponse,
    shares: [Vec<u8>; 3],
) -> eyre::Result<()> {
    tracing::debug!("decode pub key 0");
    let pk0 = PublicKey::from_bytes(
        Base64::decode_vec(&res.key_material[0].enc_key)?
            .try_into()
            .expect("correct len"),
    );
    tracing::debug!("decode pub key 0");
    let pk1 = PublicKey::from_bytes(
        Base64::decode_vec(&res.key_material[1].enc_key)?
            .try_into()
            .expect("correct len"),
    );
    tracing::debug!("decode pub key 0");
    let pk2 = PublicKey::from_bytes(
        Base64::decode_vec(&res.key_material[2].enc_key)?
            .try_into()
            .expect("correct len"),
    );
    tracing::debug!("sealing shares...");
    let ct0 = pk0
        .seal(&mut OsRng, &shares[0])
        .context("while sealing share")?;
    let ct1 = pk1
        .seal(&mut OsRng, &shares[1])
        .context("while sealing share")?;
    let ct2 = pk2
        .seal(&mut OsRng, &shares[2])
        .context("while sealing share")?;
    job_api::add_input(config, &res.job_id.to_string(), ct0, ct1, ct2).await?;
    Ok(())
}

/// Schedule a full REP3 job including witness extenesion.
pub async fn schedule_full_job_rep3<P>(
    config: &Configuration,
    code: &str,
    blueprint_id: Uuid,
    input: Input,
    public_inputs: &[String],
) -> eyre::Result<Uuid>
where
    P: Pairing + CircomArkworksPairingBridge,
    P::ScalarField: CircomArkworksPrimeFieldBridge,
    P::BaseField: CircomArkworksPrimeFieldBridge,
{
    tracing::debug!("schedule_job Rep3Full for blueprint_id {blueprint_id}");
    let res = schedule_job(config, code, blueprint_id, JobType::Rep3Full).await?;
    tracing::info!("got job_id {}", res.job_id);
    tracing::debug!("sharing input...");
    let [share0, share1, share2] = split_input::<P::ScalarField>(input, public_inputs)?;
    let shares = [
        bincode::serialize(&share0)?,
        bincode::serialize(&share1)?,
        bincode::serialize(&share2)?,
    ];
    tracing::debug!("adding input for job...");
    add_input(config, &res, shares).await?;
    tracing::debug!("done");
    Ok(res.job_id)
}

/// Schedule a REP3 prove job.
pub async fn schedule_prove_job_rep3<P>(
    config: &Configuration,
    code: &str,
    blueprint_id: Uuid,
    witness: Witness<P::ScalarField>,
    num_pub_inputs: usize,
) -> eyre::Result<Uuid>
where
    P: Pairing + CircomArkworksPairingBridge,
    P::ScalarField: CircomArkworksPrimeFieldBridge,
    P::BaseField: CircomArkworksPrimeFieldBridge,
{
    tracing::debug!("schedule_job Rep3Prove for blueprint_id {blueprint_id}");
    let res = schedule_job(config, code, blueprint_id, JobType::Rep3Prove).await?;
    tracing::info!("got job_id {}", res.job_id);
    let mut rng = rand::thread_rng();
    tracing::debug!("sharing witness...");
    let [share0, share1, share2] = CompressedRep3SharedWitness::<P::ScalarField>::share_rep3(
        witness,
        num_pub_inputs,
        &mut rng,
        Compression::SeededHalfShares,
    );
    let shares = [
        bincode::serialize(&share0)?,
        bincode::serialize(&share1)?,
        bincode::serialize(&share2)?,
    ];
    tracing::debug!("adding input for job...");
    add_input(config, &res, shares).await?;
    tracing::debug!("done");
    Ok(res.job_id)
}

/// Schedule a Shamir prove job.
pub async fn schedule_prove_job_shamir<P>(
    config: &Configuration,
    code: &str,
    blueprint_id: Uuid,
    witness: Witness<P::ScalarField>,
    num_pub_inputs: usize,
) -> eyre::Result<Uuid>
where
    P: Pairing + CircomArkworksPairingBridge,
    P::ScalarField: CircomArkworksPrimeFieldBridge,
    P::BaseField: CircomArkworksPrimeFieldBridge,
{
    tracing::debug!("schedule_job ShamirProve for blueprint_id {blueprint_id}");
    let res = schedule_job(config, code, blueprint_id, JobType::ShamirProve).await?;
    tracing::info!("got job_id {}", res.job_id);
    let mut rng = rand::thread_rng();
    tracing::debug!("sharing witness...");
    let [share0, share1, share2] = ShamirSharedWitness::<P::ScalarField>::share_shamir(
        witness,
        num_pub_inputs,
        1,
        3,
        &mut rng,
    )
    .try_into()
    .expect("correct len");
    let shares = [
        bincode::serialize(&share0)?,
        bincode::serialize(&share1)?,
        bincode::serialize(&share2)?,
    ];
    tracing::debug!("adding input for job...");
    add_input(config, &res, shares).await?;
    tracing::debug!("done");
    Ok(res.job_id)
}

/// The result of a scheduled job, represents either a successful proof, a error, or the status of a still running job
#[derive(Debug, Clone)]
pub enum JobResult<P: Pairing> {
    Ok((Proof<P>, Vec<P::ScalarField>)),
    Running(JobStatus),
    Err(String),
}

pub async fn get_job_result<P: Pairing>(
    config: &Configuration,
    id: Uuid,
) -> eyre::Result<JobResult<P>> {
    let res = job_api::get_result(config, &id.to_string()).await?;
    tracing::debug!("result from api: {res:?}");
    match res.status {
        JobStatus::Success => {
            let proof_res = res.ok.unwrap().unwrap();
            tracing::debug!("deser proof...");
            let proof = Proof::<P>::deserialize_uncompressed_unchecked(
                Base64::decode_vec(&proof_res.proof)?.as_slice(),
            )?;
            tracing::debug!("deser public_inputs...");
            let public_inputs = Vec::<P::ScalarField>::deserialize_uncompressed_unchecked(
                Base64::decode_vec(&proof_res.public_inputs)?.as_slice(),
            )?;
            tracing::debug!("done");
            Ok(JobResult::Ok((proof, public_inputs)))
        }
        JobStatus::Failed => Ok(JobResult::Err(res.error.unwrap().unwrap())),
        status => Ok(JobResult::Running(status)),
    }
}
