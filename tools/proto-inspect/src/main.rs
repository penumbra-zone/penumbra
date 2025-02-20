use anyhow::anyhow;
use penumbra_sdk_proto::{DomainType as _, Message as _};
use penumbra_sdk_transaction::Transaction;
use std::io::{self, BufRead};

fn main() -> anyhow::Result<()> {
    let input = io::stdin()
        .lock()
        .lines()
        .next()
        .ok_or(anyhow!("you need to input a base64 string"))??;
    let data = hex::decode(input)?;
    let tx_result = tendermint_proto::v0_37::abci::TxResult::decode(data.as_slice())?;
    let tx = Transaction::decode(tx_result.tx.to_vec().as_slice())?;
    println!(
        "(height, index): ({}, {})",
        tx_result.height, tx_result.index
    );
    println!("hash: {}", hex::encode_upper(tx.id().0));
    println!("tx: {:?}", &tx);
    Ok(())
}
