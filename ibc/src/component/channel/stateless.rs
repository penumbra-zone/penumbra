pub mod channel_open_init {
    use super::super::*;

    pub fn connection_hops_eq_1(msg: &MsgChannelOpenInit) -> Result<(), anyhow::Error> {
        if msg.channel.connection_hops.len() != 1 {
            return Err(anyhow::anyhow!(
                "currently only channels with one connection hop are supported"
            ));
        }
        Ok(())
    }
}

pub mod channel_open_try {
    use super::super::*;

    pub fn connection_hops_eq_1(msg: &MsgChannelOpenTry) -> anyhow::Result<()> {
        if msg.channel.connection_hops.len() != 1 {
            return Err(anyhow::anyhow!(
                "currently only channels with one connection hop are supported"
            ));
        }
        Ok(())
    }
}

pub mod channel_open_ack {}

pub mod channel_open_confirm {}
