use anyhow::Result;
use penumbra_stake::IdentityKey;
use penumbra_wallet::ClientState;
use structopt::StructOpt;

use crate::Opt;

#[derive(Debug, StructOpt)]
pub enum ValidatorCmd {
    /// Display the validator identity key derived from this wallet's spend seed.
    Identity,
}

impl ValidatorCmd {
    pub fn needs_sync(&self) -> bool {
        match self {
            ValidatorCmd::Identity => false,
        }
    }

    pub async fn exec(&self, _opt: &Opt, state: &ClientState) -> Result<()> {
        match self {
            ValidatorCmd::Identity => {
                let ik = IdentityKey(
                    state
                        .wallet()
                        .full_viewing_key()
                        .spend_verification_key()
                        .clone(),
                );

                println!("{}", ik);
            }
        }

        Ok(())
    }
}
