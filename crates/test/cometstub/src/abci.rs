//! ABCI- and ABCI++-related facilities.
//!
//! See the [ABCI++ specification][abci-spec] for more information. See ["Methods][abci-methods]
//! for more information on ABCI methods.
//!
//! [abci-spec]: https://github.com/cometbft/cometbft/blob/main/spec/abci/README.md
//! [abci-methods]: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci++_methods.md
//
//  TODO(kate): `tendermint::abci::request` types, stub these out as needed.
//    - apply_snapshot_chunk::ApplySnapshotChunk,
//    - begin_block::BeginBlock,
//    - check_tx::{CheckTx, CheckTxKind},
//    - deliver_tx::DeliverTx,
//    - echo::Echo,
//    - end_block::EndBlock,
//    - extend_vote::ExtendVote,
//    - finalize_block::FinalizeBlock,
//    - info::Info,
//    - init_chain::InitChain,
//    - load_snapshot_chunk::LoadSnapshotChunk,
//    - offer_snapshot::OfferSnapshot,
//    - prepare_proposal::PrepareProposal,
//    - process_proposal::ProcessProposal,
//    - query::Query,
//    - set_option::SetOption,
//    - verify_vote_extension::VerifyVoteExtension,

use tendermint::{
    abci::{request::BeginBlock, types},
    block::Round,
    Hash,
};

#[allow(dead_code)] // XXX(kate)
pub(crate) fn begin_block() -> BeginBlock {
    BeginBlock {
        hash: Hash::None,
        header: crate::header::header(),
        last_commit_info: types::CommitInfo {
            round: Round::default(),
            votes: vec![],
        },
        byzantine_validators: vec![],
    }
}
