pub mod connection_open_init {
    use super::super::*;

    pub fn version_is_supported(msg: &MsgConnectionOpenInit) -> anyhow::Result<()> {
        // check if the version is supported (we use the same versions as the cosmos SDK)
        // TODO: should we be storing the compatible versions in our state instead?
        if !SUPPORTED_VERSIONS.contains(
            msg.version
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("invalid version"))?,
        ) {
            return Err(anyhow::anyhow!(
                "unsupported version: in ConnectionOpenInit",
            ));
        } else {
            Ok(())
        }
    }
}

pub mod connection_open_ack {
    use super::super::*;

    pub fn has_client_proof(msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
        if msg.proofs.client_proof().is_none() {
            return Err(anyhow::anyhow!("missing client_proof"));
        } else {
            Ok(())
        }
    }

    pub fn has_client_state(msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
        if msg.client_state.is_none() {
            return Err(anyhow::anyhow!("missing client_state"));
        } else {
            Ok(())
        }
    }

    pub fn has_consensus_proof(msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
        if msg.proofs.consensus_proof().is_none() {
            return Err(anyhow::anyhow!("missing consensus_proof"));
        } else {
            Ok(())
        }
    }
}

pub mod connection_open_try {
    use super::super::*;

    pub fn has_client_proof(msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
        if msg.proofs.client_proof().is_none() {
            return Err(anyhow::anyhow!("missing client_proof"));
        } else {
            Ok(())
        }
    }

    pub fn has_client_state(msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
        if msg.client_state.is_none() {
            return Err(anyhow::anyhow!("missing client_state"));
        } else {
            Ok(())
        }
    }

    pub fn has_consensus_proof(msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
        if msg.proofs.consensus_proof().is_none() {
            return Err(anyhow::anyhow!("missing consensus_proof"));
        } else {
            Ok(())
        }
    }
}
