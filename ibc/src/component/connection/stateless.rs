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
