//! Facilities for generating tendermint [`Header`]s

use tendermint::{
    account,
    block::{self, Header},
    chain,
    validator::Set,
    AppHash, Hash, Time,
};

pub(crate) fn header() -> Header {
    let validators_hash = Set::new(vec![], None).hash();
    Header {
        version: block::header::Version { block: 0, app: 0 },
        chain_id: chain::Id::try_from("test").unwrap(),
        height: block::Height::default(),
        time: Time::unix_epoch(),
        last_block_id: None,
        last_commit_hash: None,
        data_hash: None,
        validators_hash,
        next_validators_hash: validators_hash,
        consensus_hash: Hash::None,
        app_hash: app_hash(),
        last_results_hash: None,
        evidence_hash: None,
        proposer_address: account::Id::new([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]),
    }
}

// TODO(kate): informalsystems/tendermint-rs#1243
fn app_hash() -> AppHash {
    AppHash::try_from(vec![1, 2, 3]).expect("AppHash::try_from is infallible")
}
