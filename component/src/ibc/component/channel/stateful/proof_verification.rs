use crate::ibc::component::client::StateReadExt;

use super::super::*;
use ibc::clients::ics07_tendermint::client_state::ClientState as TendermintClientState;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics04_channel::context::calculate_block_delay;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
use ibc::core::ics23_commitment::commitment::CommitmentRoot;
use ibc::core::ics23_commitment::merkle::apply_prefix;
use ibc::core::ics23_commitment::merkle::MerkleProof;
use ibc::core::ics23_commitment::specs::ProofSpecs;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::AcksPath;
use ibc::core::ics24_host::path::CommitmentsPath;
use ibc::core::ics24_host::path::ReceiptsPath;
use ibc::core::ics24_host::path::SeqRecvsPath;
use ibc::core::ics24_host::Path;
use ibc::downcast;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use prost::Message;

use ibc::core::ics02_client::client_state::AnyClientState;
use penumbra_chain::StateReadExt as _;
use sha2::{Digest, Sha256};

// NOTE: this is underspecified.
// using the same implementation here as ibc-go:
// https://github.com/cosmos/ibc-go/blob/main/modules/core/04-channel/types/packet.go#L19
// timeout_timestamp + timeout_height.revision_number + timeout_height.revision_height
// + sha256(data)
pub fn commit_packet(packet: &Packet) -> Vec<u8> {
    let mut commit = vec![];
    commit.extend_from_slice(&packet.timeout_timestamp.nanoseconds().to_be_bytes());
    commit.extend_from_slice(
        &packet
            .timeout_height
            .commitment_revision_number()
            .to_be_bytes(),
    );
    commit.extend_from_slice(
        &packet
            .timeout_height
            .commitment_revision_height()
            .to_be_bytes(),
    );
    commit.extend_from_slice(&Sha256::digest(&packet.data)[..]);

    Sha256::digest(&commit).to_vec()
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
        .map_err(|_| anyhow::anyhow!("invalid merkle proof"))?
        .into();

    merkle_proof.verify_non_membership(proof_specs, root.clone().into(), merkle_path)?;

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
        .map_err(|_| anyhow::anyhow!("invalid merkle proof"))?
        .into();

    merkle_proof.verify_membership(proof_specs, root.clone().into(), merkle_path, value, 0)?;

    Ok(())
}

#[async_trait]
pub trait ChannelProofVerifier: StateReadExt {
    async fn verify_channel_proof(
        &self,
        connection: &ConnectionEnd,
        proofs: &ibc::proofs::Proofs,
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
            .get_verified_consensus_state(proofs.height(), connection.client_id().clone())
            .await?;

        let client_def = AnyClient::from_client_type(trusted_client_state.client_type());

        // PROOF VERIFICATION. verify that our counterparty committed expected_channel to its
        // state.
        client_def.verify_channel_state(
            &trusted_client_state,
            proofs.height(),
            connection.counterparty().prefix(),
            proofs.object_proof(),
            trusted_consensus_state.root(),
            port_id,
            channel_id,
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
                &msg.proofs.height(),
                connection,
            )
            .await?;

        let commitment_path = CommitmentsPath {
            port_id: msg.packet.destination_port.clone(),
            channel_id: msg.packet.destination_channel,
            sequence: msg.packet.sequence,
        };

        let commitment_bytes = commit_packet(&msg.packet);

        verify_merkle_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            msg.proofs.object_proof(),
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
                &msg.proofs.height(),
                connection,
            )
            .await?;

        let ack_path = AcksPath {
            port_id: msg.packet.destination_port.clone(),
            channel_id: msg.packet.destination_channel,
            sequence: msg.packet.sequence,
        };

        verify_merkle_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            msg.proofs.object_proof(),
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
                &msg.proofs.height(),
                connection,
            )
            .await?;

        let mut seq_bytes = Vec::new();
        u64::from(msg.next_sequence_recv)
            .encode(&mut seq_bytes)
            .expect("buffer size too small");

        let seq_path = SeqRecvsPath(
            msg.packet.destination_port.clone(),
            msg.packet.destination_channel,
        );

        verify_merkle_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            msg.proofs.object_proof(),
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
                &msg.proofs.height(),
                connection,
            )
            .await?;

        let receipt_path = ReceiptsPath {
            port_id: msg.packet.destination_port.clone(),
            channel_id: msg.packet.destination_channel,
            sequence: msg.packet.sequence,
        };

        verify_merkle_absence_proof(
            &trusted_client_state.proof_specs,
            connection.counterparty().prefix(),
            msg.proofs.object_proof(),
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
            height: &ibc::Height,
            connection: &ConnectionEnd,
        ) -> anyhow::Result<(TendermintClientState, AnyConsensusState)> {
            let trusted_client_state = self.get_client_state(client_id).await?;

            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            let trusted_consensus_state = self
                .get_verified_consensus_state(*height, client_id.clone())
                .await?;

            let tm_client_state = downcast!(trusted_client_state => AnyClientState::Tendermint)
                .ok_or_else(|| anyhow::anyhow!("client state is not tendermint"))?;

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
            let delay_period_blocks = calculate_block_delay(delay_period_time, max_time_per_block);

            TendermintClientState::verify_delay_passed(
                current_timestamp.into(),
                ibc::Height::new(0, current_height)?,
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
