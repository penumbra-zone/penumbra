use crate::ibc::component::client::StateReadExt;

// NOTE: where should this code live after the refactor to actionhandlers?

use super::super::*;
use ibc_proto::ibc::core::commitment::v1::{MerklePath, MerkleRoot};
use ibc_types::{
    clients::ics07_tendermint::{
        client_state::ClientState as TendermintClientState,
        consensus_state::ConsensusState as TendermintConsensusState,
    },
    core::{
        ics02_client::client_state::ClientState,
        ics04_channel::context::calculate_block_delay,
        ics23_commitment::{
            commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
            error::CommitmentError,
            merkle::{apply_prefix, MerkleProof},
            specs::ProofSpecs,
        },
        ics24_host::{
            identifier::ClientId,
            path::{AckPath, ChannelEndPath, CommitmentPath, ReceiptPath, SeqRecvPath},
            Path,
        },
    },
    Height,
};

use anyhow::Context;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ics23::NonExistenceProof;
use penumbra_chain::StateReadExt as _;
use prost::Message;
use sha2::{Digest, Sha256};

// NOTE: this is underspecified.
// using the same implementation here as ibc-go:
// https://github.com/cosmos/ibc-go/blob/main/modules/core/04-channel/types/packet.go#L19
// timeout_timestamp + timeout_height.revision_number + timeout_height.revision_height
// + sha256(data)
pub fn commit_packet(packet: &Packet) -> Vec<u8> {
    let mut commit = vec![];
    commit.extend_from_slice(&packet.timeout_timestamp_on_b.nanoseconds().to_be_bytes());
    commit.extend_from_slice(
        &packet
            .timeout_height_on_b
            .commitment_revision_number()
            .to_be_bytes(),
    );
    commit.extend_from_slice(
        &packet
            .timeout_height_on_b
            .commitment_revision_height()
            .to_be_bytes(),
    );
    commit.extend_from_slice(&Sha256::digest(&packet.data)[..]);

    Sha256::digest(&commit).to_vec()
}

// NOTE: this is underspecified.
// https://github.com/cosmos/ibc-go/blob/main/modules/core/04-channel/types/packet.go#L38
pub fn commit_acknowledgement(ack_data: &[u8]) -> Vec<u8> {
    Sha256::digest(ack_data).to_vec()
}

pub fn verify_membership(
    proof: MerkleProof,
    specs: &ProofSpecs,
    root: MerkleRoot,
    keys: MerklePath,
    value: Vec<u8>,
    start_index: usize,
) -> Result<(), CommitmentError> {
    // validate arguments
    if proof.proofs.is_empty() {
        return Err(CommitmentError::EmptyMerkleProof);
    }
    if root.hash.is_empty() {
        return Err(CommitmentError::EmptyMerkleRoot);
    }
    let num = proof.proofs.len();
    let ics23_specs = Vec::<ics23::ProofSpec>::from(specs.clone());
    if ics23_specs.len() != num {
        return Err(CommitmentError::NumberOfSpecsMismatch);
    }
    if keys.key_path.len() != num {
        return Err(CommitmentError::NumberOfKeysMismatch);
    }
    if value.is_empty() {
        return Err(CommitmentError::EmptyVerifiedValue);
    }

    let mut subroot = value.clone();
    let mut value = value;
    // keys are represented from root-to-leaf
    for ((proof, spec), key) in proof
        .proofs
        .iter()
        .zip(ics23_specs.iter())
        .zip(keys.key_path.iter().rev())
        .skip(start_index)
    {
        match &proof.proof {
            Some(ics23::commitment_proof::Proof::Exist(existence_proof)) => {
                subroot =
                    ics23::calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof)
                        .map_err(|_| CommitmentError::InvalidMerkleProof)?;

                if !ics23::verify_membership::<ics23::HostFunctionsManager>(
                    proof,
                    spec,
                    &subroot,
                    key.as_bytes(),
                    &value,
                ) {
                    return Err(CommitmentError::VerificationFailure);
                }
                value = subroot.clone();
            }
            _ => return Err(CommitmentError::InvalidMerkleProof),
        }
    }

    if root.hash != subroot {
        return Err(CommitmentError::VerificationFailure);
    }

    Ok(())
}
// TODO move to ics23
fn calculate_non_existence_root(proof: &NonExistenceProof) -> Result<Vec<u8>, CommitmentError> {
    if let Some(left) = &proof.left {
        ics23::calculate_existence_root::<ics23::HostFunctionsManager>(left)
            .map_err(|_| CommitmentError::InvalidMerkleProof)
    } else if let Some(right) = &proof.right {
        ics23::calculate_existence_root::<ics23::HostFunctionsManager>(right)
            .map_err(|_| CommitmentError::InvalidMerkleProof)
    } else {
        Err(CommitmentError::InvalidMerkleProof)
    }
}

pub fn verify_non_membership(
    nonmembership_proof: MerkleProof,
    specs: &ProofSpecs,
    root: MerkleRoot,
    keys: MerklePath,
) -> Result<(), CommitmentError> {
    // validate arguments
    if nonmembership_proof.proofs.is_empty() {
        return Err(CommitmentError::EmptyMerkleProof);
    }
    if root.hash.is_empty() {
        return Err(CommitmentError::EmptyMerkleRoot);
    }
    let num = nonmembership_proof.proofs.len();
    let ics23_specs = Vec::<ics23::ProofSpec>::from(specs.clone());
    if ics23_specs.len() != num {
        return Err(CommitmentError::NumberOfSpecsMismatch);
    }
    if keys.key_path.len() != num {
        return Err(CommitmentError::NumberOfKeysMismatch);
    }

    // verify the absence of key in lowest subtree
    let proof = nonmembership_proof
        .proofs
        .get(0)
        .ok_or(CommitmentError::InvalidMerkleProof)?;
    let spec = ics23_specs
        .get(0)
        .ok_or(CommitmentError::InvalidMerkleProof)?;
    // keys are represented from root-to-leaf
    let key = keys
        .key_path
        .get(num - 1)
        .ok_or(CommitmentError::InvalidMerkleProof)?;
    match &proof.proof {
        Some(ics23::commitment_proof::Proof::Nonexist(non_existence_proof)) => {
            let subroot = calculate_non_existence_root(non_existence_proof)?;

            if !ics23::verify_non_membership::<ics23::HostFunctionsManager>(
                proof,
                spec,
                &subroot,
                key.as_bytes(),
            ) {
                return Err(CommitmentError::VerificationFailure);
            }

            // verify membership proofs starting from index 1 with value = subroot
            verify_membership(nonmembership_proof, specs, root, keys, subroot, 1)
        }
        _ => Err(CommitmentError::InvalidMerkleProof),
    }
}
fn verify_merkle_absence_proof(
    proof_specs: &ProofSpecs,
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: impl Into<Path>,
) -> anyhow::Result<()> {
    let merkle_path = apply_prefix(prefix, vec![path.into().to_string()]);
    let merkle_proof: MerkleProof = RawMerkleProof::try_from(proof.clone())
        .context("invalid merkle proof")?
        .into();

    verify_non_membership(merkle_proof, proof_specs, root.clone().into(), merkle_path)?;

    Ok(())
}

fn verify_merkle_proof(
    proof_specs: &ProofSpecs,
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: impl Into<Path>,
    value: Vec<u8>,
) -> anyhow::Result<()> {
    let merkle_path = apply_prefix(prefix, vec![path.into().to_string()]);
    let merkle_proof: MerkleProof = RawMerkleProof::try_from(proof.clone())
        .context("invalid merkle proof")?
        .into();

    verify_membership(
        merkle_proof,
        proof_specs,
        root.clone().into(),
        merkle_path,
        value,
        0,
    )?;

    Ok(())
}

#[async_trait]
pub trait ChannelProofVerifier: StateReadExt {
    async fn verify_channel_proof(
        &self,
        connection: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        proof_height: &Height,
        channel_id: &ChannelId,
        port_id: &PortId,
        expected_channel: &ChannelEnd,
    ) -> anyhow::Result<()> {
        // get the stored client state for the counterparty
        let trusted_client_state = self.get_client_state(connection.client_id()).await?;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if trusted_client_state.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        // get the stored consensus state for the counterparty
        let trusted_consensus_state = self
            .get_verified_consensus_state(*proof_height, connection.client_id().clone())
            .await?;

        let client_def = trusted_client_state;

        // PROOF VERIFICATION. verify that our counterparty committed expected_channel to its
        // state.
        client_def.verify_channel_state(
            *proof_height,
            connection.counterparty().prefix(),
            proof,
            trusted_consensus_state.root(),
            &ChannelEndPath::new(port_id, channel_id),
            expected_channel,
        )?;

        Ok(())
    }
}

impl<T: StateRead> ChannelProofVerifier for T {}

#[async_trait]
pub trait PacketProofVerifier: StateReadExt + inner::Inner {
    async fn verify_packet_recv_proof(
        &self,
        connection: &ConnectionEnd,
        msg: &MsgRecvPacket,
    ) -> anyhow::Result<()> {
        let (trusted_client_state, trusted_consensus_state) = self
            .get_trusted_client_and_consensus_state(
                connection.client_id(),
                &msg.proof_height_on_a,
                connection,
            )
            .await?;

        let commitment_path = CommitmentPath {
            port_id: msg.packet.port_on_b.clone(),
            channel_id: msg.packet.chan_on_b.clone(),
            sequence: msg.packet.sequence,
        };

        let commitment_bytes = commit_packet(&msg.packet);

        verify_merkle_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            &msg.proof_commitment_on_a,
            trusted_consensus_state.root(),
            commitment_path,
            commitment_bytes,
        )?;

        Ok(())
    }

    async fn verify_packet_ack_proof(
        &self,
        connection: &ConnectionEnd,
        msg: &MsgAcknowledgement,
    ) -> anyhow::Result<()> {
        let (trusted_client_state, trusted_consensus_state) = self
            .get_trusted_client_and_consensus_state(
                connection.client_id(),
                &msg.proof_height_on_b,
                connection,
            )
            .await?;

        let ack_path = AckPath {
            port_id: msg.packet.port_on_b.clone(),
            channel_id: msg.packet.chan_on_b.clone(),
            sequence: msg.packet.sequence,
        };

        verify_merkle_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            &msg.proof_acked_on_b,
            trusted_consensus_state.root(),
            ack_path,
            msg.acknowledgement.clone().into(),
        )?;

        Ok(())
    }

    async fn verify_packet_timeout_proof(
        &self,
        connection: &ConnectionEnd,
        msg: &MsgTimeout,
    ) -> anyhow::Result<()> {
        let (trusted_client_state, trusted_consensus_state) = self
            .get_trusted_client_and_consensus_state(
                connection.client_id(),
                &msg.proof_height_on_b,
                connection,
            )
            .await?;

        let mut seq_bytes = Vec::new();
        u64::from(msg.next_seq_recv_on_b)
            .encode(&mut seq_bytes)
            .expect("buffer size too small");

        let seq_path = SeqRecvPath(msg.packet.port_on_b.clone(), msg.packet.chan_on_b.clone());

        verify_merkle_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            &msg.proof_unreceived_on_b,
            trusted_consensus_state.root(),
            seq_path,
            seq_bytes,
        )?;

        Ok(())
    }

    async fn verify_packet_timeout_absence_proof(
        &self,
        connection: &ConnectionEnd,
        msg: &MsgTimeout,
    ) -> anyhow::Result<()> {
        let (trusted_client_state, trusted_consensus_state) = self
            .get_trusted_client_and_consensus_state(
                connection.client_id(),
                &msg.proof_height_on_b,
                connection,
            )
            .await?;

        let receipt_path = ReceiptPath {
            port_id: msg.packet.port_on_b.clone(),
            channel_id: msg.packet.chan_on_b.clone(),
            sequence: msg.packet.sequence,
        };

        verify_merkle_absence_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            &msg.proof_unreceived_on_b,
            trusted_consensus_state.root(),
            receipt_path,
        )?;

        Ok(())
    }
}

impl<T: StateRead> PacketProofVerifier for T {}

mod inner {
    use super::*;

    #[async_trait]
    pub trait Inner: StateReadExt {
        async fn get_trusted_client_and_consensus_state(
            &self,
            client_id: &ClientId,
            height: &ibc_types::Height,
            connection: &ConnectionEnd,
        ) -> anyhow::Result<(TendermintClientState, TendermintConsensusState)> {
            let trusted_client_state = self.get_client_state(client_id).await?;

            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            let trusted_consensus_state = self
                .get_verified_consensus_state(*height, client_id.clone())
                .await?;

            let tm_client_state = trusted_client_state;

            tm_client_state.verify_height(*height)?;

            // verify that the delay time has passed (see ICS07 tendermint IBC client spec for
            // more details)
            let current_timestamp = self.get_block_timestamp().await?;
            let current_height = self.get_block_height().await?;
            let processed_height = self.get_client_update_height(client_id, height).await?;
            let processed_time = self.get_client_update_time(client_id, height).await?;

            // NOTE: hardcoded for now, should probably be a chain parameter.
            let max_time_per_block = std::time::Duration::from_secs(20);

            let delay_period_time = connection.delay_period();
            let delay_period_blocks =
                calculate_block_delay(&delay_period_time, &max_time_per_block);

            TendermintClientState::verify_delay_passed(
                current_timestamp.into(),
                ibc_types::Height::new(0, current_height)?,
                processed_time,
                processed_height,
                delay_period_time,
                delay_period_blocks,
            )?;

            Ok((tm_client_state, trusted_consensus_state))
        }
    }

    impl<T: StateReadExt> Inner for T {}
}
