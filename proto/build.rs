use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();

    // Specify which parts of the protos should have their `bytes` fields
    // converted to Rust `Bytes` (= zero-copy view into a shared buffer) rather
    // than `Vec<u8>`.
    //
    // The upside of using the `Bytes` type is that it avoids copies while
    // parsing the protos.
    //
    // The downside is that the underlying buffer is kept alive as long as there
    // is at least one view into it, so holding on to a small amount of data
    // (e.g., a hash) could keep a much larger buffer live for a long time,
    // increasing memory use.
    //
    // Getting this tradeoff perfect isn't essential, but it's useful to keep in mind.
    config.bytes(&[
        // Transactions have a lot of `bytes` fields that need to be converted
        // into fixed-size byte arrays anyways, so there's no point allocating
        // into a temporary vector.
        ".penumbra.transaction",
        // The byte fields in a compact block will also be converted to fixed-size
        // byte arrays and then discarded.
        ".penumbra.crypto.NotePayload",
        ".penumbra.chain.CompactBlock",
    ]);

    for (path, attribute) in TYPE_ATTRIBUTES.iter() {
        config.type_attribute(path, attribute);
    }
    for (path, attribute) in FIELD_ATTRIBUTES.iter() {
        config.field_attribute(path, attribute);
    }

    // NOTE: we need this because the rust module that defines the IBC types is external, and not
    // part of this crate.
    // See https://docs.rs/prost-build/0.5.0/prost_build/struct.Config.html#method.extern_path
    config.extern_path(".ibc", "::ibc_proto::ibc");
    config.extern_path(".ics23", "::ics23");

    config.compile_protos(
        &[
            "proto/crypto.proto",
            "proto/transaction.proto",
            "proto/stake.proto",
            "proto/chain.proto",
            "proto/ibc.proto",
            "proto/dex.proto",
            "proto/governance.proto",
        ],
        &["proto/", "ibc-go-vendor/"],
    )?;

    // These should disappear, eventually.
    config.compile_protos(
        &["proto/transparent_proofs.proto"],
        &["proto/", "ibc-go-vendor/"],
    )?;

    // For the client code, we also want to generate RPC instances, so compile via tonic:
    tonic_build::configure().compile_with_config(
        config,
        &[
            "proto/client/oblivious.proto",
            "proto/client/specific.proto",
            "proto/view.proto",
            "proto/custody.proto",
        ],
        &["proto/", "ibc-go-vendor/"],
    )?;

    Ok(())
}

static SERIALIZE: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;
/// Serializes newtype structs as if the inner field were serialized on its own.
static SERDE_TRANSPARENT: &str = r#"#[serde(transparent)]"#;
static SERDE_FLATTEN: &str = r#"#[serde(flatten)]"#;
static SERDE_TAG_KIND: &str = r#"#[serde(tag = "kind")]"#;
static SERDE_TAG_STATE: &str = r#"#[serde(tag = "state")]"#;
static SERDE_TAG_OUTCOME: &str = r#"#[serde(tag = "outcome")]"#;
static SERDE_SNAKE_CASE: &str = r#"#[serde(rename_all = "snake_case")]"#;
static SERDE_SKIP_NONE: &str = r#"#[serde(skip_serializing_if = "Option::is_none", default)]"#;

static AS_HEX: &str = r#"#[serde(with = "crate::serializers::hexstr")]"#;
static AS_HEX_FOR_BYTES: &str = r#"#[serde(with = "crate::serializers::hexstr_bytes")]"#;
static AS_BASE64: &str = r#"#[serde(with = "crate::serializers::base64str")]"#;
static AS_BASE64_FOR_BYTES: &str = r#"#[serde(with = "crate::serializers::base64str_bytes")]"#;
static AS_BECH32_IDENTITY_KEY: &str =
    r#"#[serde(with = "crate::serializers::bech32str::validator_identity_key")]"#;
static AS_BECH32_GOVERNANCE_KEY: &str =
    r#"#[serde(with = "crate::serializers::bech32str::validator_governance_key")]"#;
static AS_BECH32_ADDRESS: &str = r#"#[serde(with = "crate::serializers::bech32str::address")]"#;
static AS_BECH32_ASSET_ID: &str = r#"#[serde(with = "crate::serializers::bech32str::asset_id")]"#;
static AS_BECH32_SPEND_KEY: &str = r#"#[serde(with = "crate::serializers::bech32str::spend_key")]"#;
static AS_BECH32_FULL_VIEWING_KEY: &str =
    r#"#[serde(with = "crate::serializers::bech32str::full_viewing_key")]"#;
static AS_BECH32_LP_ID: &str = r#"#[serde(with = "crate::serializers::bech32str::lp_id")]"#;
static AS_VOTE: &str = r#"#[serde(with = "crate::serializers::vote")]"#;

static TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".penumbra.stake.Validator", SERIALIZE),
    (".penumbra.stake.FundingStream", SERIALIZE),
    (".penumbra.stake.ValidatorDefinition", SERIALIZE),
    (".penumbra.stake.ValidatorInfo", SERIALIZE),
    (".penumbra.stake.ValidatorList", SERIALIZE),
    (".penumbra.stake.ValidatorState", SERIALIZE),
    (".penumbra.stake.ValidatorStateEnum", SERIALIZE),
    (".penumbra.stake.ValidatorStatus", SERIALIZE),
    (".penumbra.stake.BondingState", SERIALIZE),
    (".penumbra.stake.RateData", SERIALIZE),
    (".penumbra.stake.BaseRateData", SERIALIZE),
    (".penumbra.stake.Delegate", SERIALIZE),
    (".penumbra.stake.Undelegate", SERIALIZE),
    (".penumbra.stake.DelegationChanges", SERIALIZE),
    (".penumbra.stake.CommissionAmount", SERIALIZE),
    (".penumbra.stake.CommissionAmounts", SERIALIZE),
    (".penumbra.stake.Uptime", SERIALIZE),
    (".penumbra.stake.CurrentConsensusKeys", SERIALIZE),
    (".penumbra.crypto.IdentityKey", SERIALIZE),
    (".penumbra.crypto.IdentityKey", SERDE_TRANSPARENT),
    (".penumbra.crypto.GovernanceKey", SERIALIZE),
    (".penumbra.crypto.GovernanceKey", SERDE_TRANSPARENT),
    (".penumbra.crypto.ConsensusKey", SERIALIZE),
    (".penumbra.crypto.ConsensusKey", SERDE_TRANSPARENT),
    (".penumbra.crypto.Address", SERIALIZE),
    (".penumbra.crypto.Address", SERDE_TRANSPARENT),
    (".penumbra.crypto.Note", SERIALIZE),
    (".penumbra.crypto.NoteCommitment", SERIALIZE),
    (".penumbra.crypto.NoteCommitment", SERDE_TRANSPARENT),
    (".penumbra.crypto.ValueCommitment", SERIALIZE),
    (".penumbra.crypto.ValueCommitment", SERDE_TRANSPARENT),
    (".penumbra.crypto.NotePayload", SERIALIZE),
    (".penumbra.crypto.AssetId", SERIALIZE),
    (".penumbra.crypto.AssetId", SERDE_TRANSPARENT),
    (".penumbra.crypto.Value", SERIALIZE),
    (".penumbra.crypto.Amount", SERIALIZE),
    (".penumbra.crypto.Denom", SERIALIZE),
    (".penumbra.crypto.Denom", SERDE_TRANSPARENT),
    (".penumbra.crypto.Asset", SERIALIZE),
    (".penumbra.crypto.MerkleRoot", SERIALIZE),
    (".penumbra.crypto.MerkleRoot", SERDE_TRANSPARENT),
    (".penumbra.crypto.SpendKey", SERIALIZE),
    (".penumbra.crypto.SpendKey", SERDE_TRANSPARENT),
    (".penumbra.crypto.FullViewingKey", SERIALIZE),
    (".penumbra.crypto.FullViewingKey", SERDE_TRANSPARENT),
    (".penumbra.crypto.AccountID", SERIALIZE),
    (".penumbra.crypto.AccountID", SERDE_TRANSPARENT),
    (".penumbra.crypto.Diversifier", SERIALIZE),
    (".penumbra.crypto.Diversifier", SERDE_TRANSPARENT),
    (".penumbra.crypto.AddressIndex", SERIALIZE),
    (".penumbra.crypto.AddressIndex", SERDE_TRANSPARENT),
    (".penumbra.crypto.Nullifier", SERIALIZE),
    (".penumbra.crypto.Nullifier", SERDE_TRANSPARENT),
    (".penumbra.crypto.AuthPath", SERIALIZE),
    (".penumbra.crypto.SpendAuthSignature", SERIALIZE),
    (".penumbra.crypto.SpendAuthSignature", SERDE_TRANSPARENT),
    (".penumbra.crypto.Fee", SERIALIZE),
    (".penumbra.crypto.Clue", SERIALIZE),
    (".penumbra.crypto.Clue", SERDE_TRANSPARENT),
    (".penumbra.chain.ChainParameters", SERIALIZE),
    (".penumbra.chain.FmdParameters", SERIALIZE),
    (".penumbra.chain.CompactBlock", SERIALIZE),
    (".penumbra.chain.AnnotatedNotePayload", SERIALIZE),
    (".penumbra.chain.KnownAssets", SERIALIZE),
    (".penumbra.chain.KnownAssets", SERDE_TRANSPARENT),
    (".penumbra.chain.NoteSource", SERIALIZE),
    (".penumbra.chain.NoteSource", SERDE_TRANSPARENT),
    (".penumbra.chain.GenesisAppState", SERIALIZE),
    (".penumbra.chain.GenesisAllocation", SERIALIZE),
    (".penumbra.chain.Quarantined", SERIALIZE),
    (".penumbra.chain.QuarantinedPerValidator", SERIALIZE),
    (".penumbra.chain.Ratio", SERIALIZE),
    (".penumbra.view.SpendableNoteRecord", SERIALIZE),
    (".penumbra.view.QuarantinedNoteRecord", SERIALIZE),
    (".penumbra.transaction.AuthHash", SERIALIZE),
    (".penumbra.transaction.TransactionPlan", SERIALIZE),
    (".penumbra.transaction.ActionPlan", SERIALIZE),
    (".penumbra.transaction.SpendPlan", SERIALIZE),
    (".penumbra.transaction.OutputPlan", SERIALIZE),
    (".penumbra.transaction.SwapPlan", SERIALIZE),
    (".penumbra.transaction.SwapClaimPlan", SERIALIZE),
    (".penumbra.transaction.CluePlan", SERIALIZE),
    (".penumbra.transaction.MemoPlan", SERIALIZE),
    (".penumbra.transaction.Transaction", SERIALIZE),
    (".penumbra.transaction.TransactionBody", SERIALIZE),
    (".penumbra.transaction.Action", SERIALIZE),
    (".penumbra.transaction.Spend", SERIALIZE),
    (".penumbra.transaction.SpendBody", SERIALIZE),
    (".penumbra.transaction.Output", SERIALIZE),
    (".penumbra.transaction.OutputBody", SERIALIZE),
    (".penumbra.transaction.Proposal", SERIALIZE),
    (".penumbra.transaction.Proposal.Payload", SERDE_SNAKE_CASE),
    (
        ".penumbra.transaction.Proposal.Payload.payload",
        SERDE_TAG_KIND,
    ),
    (".penumbra.transaction.Vote", SERIALIZE),
    (".penumbra.transaction.ProposalSubmit", SERIALIZE),
    (".penumbra.transaction.ProposalWithdraw", SERIALIZE),
    (".penumbra.transaction.ProposalWithdrawPlan", SERIALIZE),
    (".penumbra.transaction.ProposalWithdrawBody", SERIALIZE),
    (".penumbra.transaction.ValidatorVote", SERIALIZE),
    (".penumbra.transaction.ValidatorVotePlan", SERIALIZE),
    (".penumbra.transaction.ValidatorVoteBody", SERIALIZE),
    (".penumbra.transaction.DelegatorVote", SERIALIZE),
    (".penumbra.transaction.DelegatorVotePlan", SERIALIZE),
    (".penumbra.transaction.DelegatorVoteBody", SERIALIZE),
    (".penumbra.ibc.IBCAction", SERIALIZE),
    (".penumbra.ibc.ICS20Withdrawal", SERIALIZE),
    (".penumbra.dex.MockFlowCiphertext", SERIALIZE),
    (".penumbra.dex.MockFlowCiphertext", SERDE_TRANSPARENT),
    (".penumbra.dex.TradingPair", SERIALIZE),
    (".penumbra.dex.TradingFunction", SERIALIZE),
    (".penumbra.dex.Reserves", SERIALIZE),
    (".penumbra.dex.Position", SERIALIZE),
    (".penumbra.dex.PositionId", SERIALIZE),
    (".penumbra.dex.PositionId", SERDE_TRANSPARENT),
    (".penumbra.dex.PositionState", SERIALIZE),
    (".penumbra.dex.PositionOpen", SERIALIZE),
    (".penumbra.dex.PositionClose", SERIALIZE),
    (".penumbra.dex.PositionWithdraw", SERIALIZE),
    (".penumbra.dex.PositionRewardClaim", SERIALIZE),
    (".penumbra.dex.Swap", SERIALIZE),
    (".penumbra.dex.SwapBody", SERIALIZE),
    (".penumbra.dex.SwapClaim", SERIALIZE),
    (".penumbra.dex.SwapClaimBody", SERIALIZE),
    (".penumbra.dex.ClaimedSwap", SERIALIZE),
    (".penumbra.dex.ClaimedSwapList", SERIALIZE),
    (".penumbra.dex.SwapPlaintext", SERIALIZE),
    (".penumbra.dex.BatchSwapOutputData", SERIALIZE),
    ("penumbra.governance.Vote", SERIALIZE),
    ("penumbra.governance.Vote", SERDE_TRANSPARENT),
    (".penumbra.governance.ProposalState", SERIALIZE),
    (".penumbra.governance.ProposalOutcome", SERIALIZE),
    (".penumbra.governance.ProposalState.state", SERDE_SNAKE_CASE),
    (".penumbra.governance.ProposalState.state", SERDE_TAG_STATE),
    (
        ".penumbra.governance.ProposalOutcome.outcome",
        SERDE_SNAKE_CASE,
    ),
    (
        ".penumbra.governance.ProposalOutcome.outcome",
        SERDE_TAG_OUTCOME,
    ),
    (".penumbra.transaction.AuthHash", SERDE_TRANSPARENT),
];

static FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    // Using base64 for the validator's consensus key means that
    // the format is the same as the Tendermint json config files.
    (".penumbra.stake.Validator.consensus_key", AS_BASE64),
    (".penumbra.stake.ValidatorDefinition.auth_sig", AS_HEX),
    (".penumbra.stake.Uptime.bitvec", AS_BASE64),
    (".penumbra.crypto.Clue.inner", AS_HEX),
    (".penumbra.crypto.Address.inner", AS_BECH32_ADDRESS),
    (".penumbra.crypto.AssetId.inner", AS_BECH32_ASSET_ID),
    (".penumbra.crypto.NoteCommitment.inner", AS_HEX),
    (".penumbra.crypto.ValueCommitment.inner", AS_HEX),
    (".penumbra.crypto.MerkleRoot.inner", AS_HEX),
    (".penumbra.crypto.SpendKey.inner", AS_BECH32_SPEND_KEY),
    (
        ".penumbra.crypto.FullViewingKey.inner",
        AS_BECH32_FULL_VIEWING_KEY,
    ),
    (".penumbra.crypto.AccountID.inner", AS_HEX),
    (".penumbra.crypto.Diversifier.inner", AS_HEX),
    (".penumbra.crypto.AddressIndex.inner", AS_HEX),
    (".penumbra.crypto.IdentityKey.ik", AS_BECH32_IDENTITY_KEY),
    (
        ".penumbra.crypto.GovernanceKey.gk",
        AS_BECH32_GOVERNANCE_KEY,
    ),
    (".penumbra.crypto.ConsensusKey.inner", AS_BASE64),
    (".penumbra.crypto.Note.note_blinding", AS_HEX),
    (".penumbra.crypto.Note.transmission_key", AS_HEX),
    (
        ".penumbra.crypto.NotePayload.ephemeral_key",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.crypto.NotePayload.encrypted_note",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.crypto.Nullifier.inner", AS_HEX),
    (".penumbra.crypto.SpendAuthSignature.inner", AS_HEX),
    (".penumbra.chain.NoteSource.inner", AS_HEX),
    // TransactionPlan formatting
    (
        ".penumbra.transaction.SpendPlan.randomizer",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.SpendPlan.value_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.OutputPlan.note_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.OutputPlan.value_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.transaction.OutputPlan.esk", AS_HEX_FOR_BYTES),
    // TODO: replace if we use UTF-8 memos
    (".penumbra.transaction.OutputPlan.memo", AS_HEX_FOR_BYTES),
    (
        ".penumbra.transaction.SwapPlan.note_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.SwapPlan.fee_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.transaction.SwapPlan.esk", AS_HEX_FOR_BYTES),
    (
        ".penumbra.transaction.SwapClaimPlan.output_1_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.SwapClaimPlan.output_2_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.SwapClaimPlan.esk_1",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.SwapClaimPlan.esk_2",
        AS_HEX_FOR_BYTES,
    ),
    // Transaction formatting
    (
        ".penumbra.transaction.Transaction.binding_sig",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.transaction.Output.proof", AS_BASE64_FOR_BYTES),
    (
        ".penumbra.transaction.OutputBody.encrypted_memo",
        AS_BASE64_FOR_BYTES,
    ),
    (
        ".penumbra.transaction.OutputBody.ovk_wrapped_key",
        AS_BASE64_FOR_BYTES,
    ),
    (".penumbra.transaction.Spend.proof", AS_BASE64_FOR_BYTES),
    (".penumbra.transaction.SpendBody.rk", AS_HEX_FOR_BYTES),
    (
        ".penumbra.transaction.SpendBody.nullifier",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.dex.Swap.proof", AS_BASE64),
    (".penumbra.dex.SwapClaim.proof", AS_BASE64),
    (".penumbra.dex.Position.nonce", AS_HEX),
    (".penumbra.dex.PositionId.inner", AS_BECH32_LP_ID),
    // Proposal JSON formatting
    (".penumbra.transaction.Proposal.payload", SERDE_FLATTEN),
    (
        // IMPORTANT: this *lacks* a leading dot, to work around a bug in prost-build:
        // https://github.com/tokio-rs/prost/issues/504
        "penumbra.transaction.Proposal.Payload.payload",
        SERDE_FLATTEN,
    ),
    (
        // see above re: prost issue #504
        "penumbra.governance.ProposalState.state",
        SERDE_FLATTEN,
    ),
    (
        // see above re: prost issue #504
        "penumbra.governance.ProposalState.Finished.outcome",
        SERDE_FLATTEN,
    ),
    (
        // see above re: prost issue #504
        "penumbra.governance.ProposalOutcome.outcome",
        SERDE_FLATTEN,
    ),
    (
        ".penumbra.governance.ProposalOutcome.Failed.withdrawn_with_reason",
        SERDE_SKIP_NONE,
    ),
    (
        ".penumbra.governance.ProposalOutcome.Vetoed.withdrawn_with_reason",
        SERDE_SKIP_NONE,
    ),
    ("penumbra.governance.Vote.vote", AS_VOTE),
    (".penumbra.transaction.AuthHash.inner", AS_HEX_FOR_BYTES),
];
