pub mod connection_open_init {
    use super::super::*;

    pub fn version_is_supported(msg: &MsgConnectionOpenInit) -> anyhow::Result<()> {
        // check if the version is supported (we use the same versions as the cosmos SDK)
        // TODO: should we be storing the compatible versions in our state instead?

        // NOTE: version can be nil in MsgConnectionOpenInit
        if msg.version.is_none() {
            return Ok(());
        }

        if !SUPPORTED_VERSIONS.contains(
            msg.version
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("invalid version"))?,
        ) {
            Err(anyhow::anyhow!(
                "unsupported version: in ConnectionOpenInit",
            ))
        } else {
            Ok(())
        }
    }
}

pub mod connection_open_ack {
    use super::super::*;

    pub fn has_client_proof(_msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
        // TODO(erwan): remove now?
        Ok(())
    }

    pub fn has_client_state(_msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
        // TODO(erwan): `Any` can't be empty?
        Ok(())
    }

    pub fn has_consensus_proof(_msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
        // TODO(erwan): remove now?
        Ok(())
    }
}

pub mod connection_open_try {
    use super::super::*;

    pub fn has_client_proof(_msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
        // TODO(erwan): remove now?
        Ok(())
    }

    pub fn has_client_state(_msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
        // TODO(erwan): remove now?
        Ok(())
    }

    pub fn has_consensus_proof(_msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
        // TODO(erwan): remove now?
        Ok(())
    }
}
