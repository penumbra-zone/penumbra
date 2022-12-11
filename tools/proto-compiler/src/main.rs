use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{}", root.display());
    let target_dir = root
        .join("..")
        .join("..")
        .join("proto")
        .join("src")
        .join("gen");
    println!("{}", target_dir.display());

    let mut config = prost_build::Config::new();
    config.out_dir(&target_dir);

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
        ".penumbra.core.transaction",
        // The byte fields in a compact block will also be converted to fixed-size
        // byte arrays and then discarded.
        ".penumbra.core.crypto.v1alpha1.EncryptedNote",
        ".penumbra.core.chain.v1alpha1.CompactBlock",
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
    // The same applies for some of the tendermint types.
    // config.extern_path(
    //     ".tendermint.types.Validator",
    //     "::tendermint::types::Validator",
    // );
    // config.extern_path(
    //     ".tendermint.p2p.DefaultNodeInfo",
    //     "::tendermint::p2p::DefaultNodeInfo",
    // );

    config.compile_protos(
        &[
            "../../proto/proto/penumbra/core/crypto/v1alpha1/crypto.proto",
            "../../proto/proto/penumbra/core/transaction/v1alpha1/transaction.proto",
            "../../proto/proto/penumbra/core/stake/v1alpha1/stake.proto",
            "../../proto/proto/penumbra/core/chain/v1alpha1/chain.proto",
            "../../proto/proto/penumbra/core/ibc/v1alpha1/ibc.proto",
            "../../proto/proto/penumbra/core/dex/v1alpha1/dex.proto",
            "../../proto/proto/penumbra/core/transparent_proofs/v1alpha1/transparent_proofs.proto",
            "../../proto/proto/penumbra/core/governance/v1alpha1/governance.proto",
            "../../proto/ibc-go-vendor/tendermint/types/validator.proto",
            "../../proto/ibc-go-vendor/tendermint/p2p/types.proto",
        ],
        &["../../proto/proto/", "../../proto/ibc-go-vendor/"],
    )?;

    // For the client code, we also want to generate RPC instances, so compile via tonic:
    tonic_build::configure()
        .out_dir(&target_dir)
        .compile_with_config(
            config,
            &[
                "../../proto/proto/penumbra/client/v1alpha1/client.proto",
                "../../proto/proto/penumbra/view/v1alpha1/view.proto",
                "../../proto/proto/penumbra/custody/v1alpha1/custody.proto",
                "../../proto/ibc-go-vendor/cosmos/base/tendermint/v1beta1/query.proto",
                "../../proto/ibc-go-vendor/tendermint/types/validator.proto",
                "../../proto/ibc-go-vendor/tendermint/p2p/types.proto",
            ],
            &["../../proto/proto/", "../../proto/ibc-go-vendor/"],
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
    (".penumbra.core.stake.v1alpha1.Validator", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.FundingStream", SERIALIZE),
    (
        ".penumbra.core.stake.v1alpha1.ValidatorDefinition",
        SERIALIZE,
    ),
    (".penumbra.core.stake.v1alpha1.ValidatorInfo", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.ValidatorList", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.ValidatorState", SERIALIZE),
    (
        ".penumbra.core.stake.v1alpha1.ValidatorStateEnum",
        SERIALIZE,
    ),
    (".penumbra.core.stake.v1alpha1.ValidatorStatus", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.BondingState", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.RateData", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.BaseRateData", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.Delegate", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.Undelegate", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.UndelegateClaim", SERIALIZE),
    (
        ".penumbra.core.stake.v1alpha1.UndelegateClaimBody",
        SERIALIZE,
    ),
    (
        ".penumbra.core.stake.v1alpha1.UndelegateClaimPlan",
        SERIALIZE,
    ),
    (".penumbra.core.stake.v1alpha1.DelegationChanges", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.CommissionAmount", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.CommissionAmounts", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.Uptime", SERIALIZE),
    (".penumbra.core.stake.v1alpha1.Penalty", SERIALIZE),
    (
        ".penumbra.core.stake.v1alpha1.CurrentConsensusKeys",
        SERIALIZE,
    ),
    (".penumbra.core.crypto.v1alpha1.IdentityKey", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.IdentityKey",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.GovernanceKey", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.GovernanceKey",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.ConsensusKey", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.ConsensusKey",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.Address", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.Address", SERDE_TRANSPARENT),
    (".penumbra.core.crypto.v1alpha1.Note", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.StateCommitment", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.StateCommitment",
        SERDE_TRANSPARENT,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.BalanceCommitment",
        SERIALIZE,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.BalanceCommitment",
        SERDE_TRANSPARENT,
    ),
    (".tendermint.types.Validator", SERIALIZE),
    (".tendermint.p2p.DefaultNodeInfo", SERIALIZE),
    (".tendermint.p2p.DefaultNodeInfoOther", SERIALIZE),
    (".tendermint.p2p.ProtocolVersion", SERIALIZE),
    (".tendermint.crypto.PublicKey", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.EncryptedNote", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.AssetId", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.AssetId", SERDE_TRANSPARENT),
    (".penumbra.core.crypto.v1alpha1.Value", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.Amount", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.Denom", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.Denom", SERDE_TRANSPARENT),
    (".penumbra.core.crypto.v1alpha1.Asset", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.MerkleRoot", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.MerkleRoot",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.SpendKey", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.SpendKey", SERDE_TRANSPARENT),
    (".penumbra.core.crypto.v1alpha1.FullViewingKey", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.FullViewingKey",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.AccountID", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.AccountID",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.Diversifier", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.Diversifier",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.AddressIndex", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.AddressIndex",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.Nullifier", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.Nullifier",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.AuthPath", SERIALIZE),
    (
        ".penumbra.core.crypto.v1alpha1.SpendAuthSignature",
        SERIALIZE,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.SpendAuthSignature",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.crypto.v1alpha1.Fee", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.Clue", SERIALIZE),
    (".penumbra.core.crypto.v1alpha1.Clue", SERDE_TRANSPARENT),
    (".penumbra.core.chain.v1alpha1.ChainParameters", SERIALIZE),
    (".penumbra.core.chain.v1alpha1.FmdParameters", SERIALIZE),
    (".penumbra.core.chain.v1alpha1.CompactBlock", SERIALIZE),
    (".penumbra.core.chain.v1alpha1.StatePayload", SERIALIZE),
    (".penumbra.core.chain.v1alpha1.KnownAssets", SERIALIZE),
    (
        ".penumbra.core.chain.v1alpha1.KnownAssets",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.chain.v1alpha1.NoteSource", SERIALIZE),
    (
        ".penumbra.core.chain.v1alpha1.NoteSource",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.chain.v1alpha1.GenesisAppState", SERIALIZE),
    (".penumbra.core.chain.v1alpha1.GenesisAllocation", SERIALIZE),
    (".penumbra.core.chain.v1alpha1.Ratio", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.AuthHash", SERIALIZE),
    (
        ".penumbra.core.transaction.v1alpha1.TransactionPlan",
        SERIALIZE,
    ),
    (".penumbra.core.transaction.v1alpha1.ActionPlan", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.SpendPlan", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.OutputPlan", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapPlan", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapClaimPlan", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.CluePlan", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.MemoPlan", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.Transaction", SERIALIZE),
    (
        ".penumbra.core.transaction.v1alpha1.TransactionBody",
        SERIALIZE,
    ),
    (".penumbra.core.transaction.v1alpha1.Action", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.Spend", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.SpendBody", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.Output", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.OutputBody", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.Proposal", SERIALIZE),
    (
        ".penumbra.core.transaction.v1alpha1.Proposal.Payload",
        SERDE_SNAKE_CASE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.Proposal.Payload.payload",
        SERDE_TAG_KIND,
    ),
    (".penumbra.core.transaction.v1alpha1.Vote", SERIALIZE),
    (
        ".penumbra.core.transaction.v1alpha1.ProposalSubmit",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.ProposalWithdraw",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.ProposalWithdrawPlan",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.ProposalWithdrawBody",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.ValidatorVote",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.ValidatorVotePlan",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.ValidatorVoteBody",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.DelegatorVote",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.DelegatorVotePlan",
        SERIALIZE,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.DelegatorVoteBody",
        SERIALIZE,
    ),
    (".penumbra.core.transaction.v1alpha1.SpendView", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.OutputView", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapClaimView", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapView", SERIALIZE),
    (".penumbra.core.transaction.v1alpha1.ActionView", SERIALIZE),
    (
        ".penumbra.core.transaction.v1alpha1.TransactionView",
        SERIALIZE,
    ),
    (".penumbra.core.ibc.v1alpha1.IbcAction", SERIALIZE),
    (".penumbra.core.ibc.v1alpha1.Ics20Withdrawal", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.MockFlowCiphertext", SERIALIZE),
    (
        ".penumbra.core.dex.v1alpha1.MockFlowCiphertext",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.core.dex.v1alpha1.TradingPair", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.TradingFunction", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.Reserves", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.Position", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.PositionId", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.PositionId", SERDE_TRANSPARENT),
    (".penumbra.core.dex.v1alpha1.PositionState", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.PositionOpen", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.PositionClose", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.PositionWithdraw", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.PositionRewardClaim", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.Swap", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapBody", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapPayload", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapClaim", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapClaimBody", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.SwapPlaintext", SERIALIZE),
    (".penumbra.core.dex.v1alpha1.BatchSwapOutputData", SERIALIZE),
    // see below re: prost issue #504
    ("penumbra.core.governance.v1alpha1.Vote", SERIALIZE),
    ("penumbra.core.governance.v1alpha1.Vote", SERDE_TRANSPARENT),
    (
        "penumbra.core.governance.v1alpha1.MutableChainParameter",
        SERIALIZE,
    ),
    (
        ".penumbra.core.governance.v1alpha1.ProposalState",
        SERIALIZE,
    ),
    (
        ".penumbra.core.governance.v1alpha1.ProposalOutcome",
        SERIALIZE,
    ),
    (
        ".penumbra.core.governance.v1alpha1.ProposalState.state",
        SERDE_SNAKE_CASE,
    ),
    (
        ".penumbra.core.governance.v1alpha1.ProposalState.state",
        SERDE_TAG_STATE,
    ),
    (
        ".penumbra.core.governance.v1alpha1.ProposalOutcome.outcome",
        SERDE_SNAKE_CASE,
    ),
    (
        ".penumbra.core.governance.v1alpha1.ProposalOutcome.outcome",
        SERDE_TAG_OUTCOME,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.AuthHash",
        SERDE_TRANSPARENT,
    ),
    (".penumbra.view.v1alpha1.SpendableNoteRecord", SERIALIZE),
    (".penumbra.view.v1alpha1.SwapRecord", SERIALIZE),
];

static FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    // Using base64 for the validator's consensus key means that
    // the format is the same as the Tendermint json config files.
    (
        ".penumbra.core.stake.v1alpha1.Validator.consensus_key",
        AS_BASE64,
    ),
    (
        ".penumbra.core.stake.v1alpha1.ValidatorDefinition.auth_sig",
        AS_HEX,
    ),
    (".penumbra.core.stake.v1alpha1.Uptime.bitvec", AS_BASE64),
    (".penumbra.core.crypto.v1alpha1.Clue.inner", AS_HEX),
    (
        ".penumbra.core.crypto.v1alpha1.Address.inner",
        AS_BECH32_ADDRESS,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.AssetId.inner",
        AS_BECH32_ASSET_ID,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.StateCommitment.inner",
        AS_HEX,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.BalanceCommitment.inner",
        AS_HEX,
    ),
    (".penumbra.core.crypto.v1alpha1.MerkleRoot.inner", AS_HEX),
    (
        ".penumbra.core.crypto.v1alpha1.SpendKey.inner",
        AS_BECH32_SPEND_KEY,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.FullViewingKey.inner",
        AS_BECH32_FULL_VIEWING_KEY,
    ),
    (".penumbra.core.crypto.v1alpha1.AccountID.inner", AS_HEX),
    (".penumbra.core.crypto.v1alpha1.Diversifier.inner", AS_HEX),
    (".penumbra.core.crypto.v1alpha1.AddressIndex.inner", AS_HEX),
    (
        ".penumbra.core.crypto.v1alpha1.IdentityKey.ik",
        AS_BECH32_IDENTITY_KEY,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.GovernanceKey.gk",
        AS_BECH32_GOVERNANCE_KEY,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.ConsensusKey.inner",
        AS_BASE64,
    ),
    (".penumbra.core.crypto.v1alpha1.Note.note_blinding", AS_HEX),
    (
        ".penumbra.core.crypto.v1alpha1.Note.transmission_key",
        AS_HEX,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.EncryptedNote.ephemeral_key",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.crypto.v1alpha1.EncryptedNote.encrypted_note",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.core.crypto.v1alpha1.Nullifier.inner", AS_HEX),
    (
        ".penumbra.core.crypto.v1alpha1.SpendAuthSignature.inner",
        AS_HEX,
    ),
    (".penumbra.core.chain.v1alpha1.NoteSource.inner", AS_HEX),
    // TransactionPlan formatting
    (
        ".penumbra.core.transaction.v1alpha1.SpendPlan.randomizer",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.SpendPlan.value_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.OutputPlan.note_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.OutputPlan.value_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.OutputPlan.esk",
        AS_HEX_FOR_BYTES,
    ),
    // TODO: replace if we use UTF-8 memos
    (
        ".penumbra.core.transaction.v1alpha1.OutputPlan.memo",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.core.dex.v1alpha1.SwapPlan.note_blinding", AS_HEX),
    (".penumbra.core.dex.v1alpha1.SwapPlan.fee_blinding", AS_HEX),
    (".penumbra.core.dex.v1alpha1.SwapPlan.esk", AS_HEX),
    (
        ".penumbra.core.dex.v1alpha1.SwapClaimPlan.output_1_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.dex.v1alpha1.SwapClaimPlan.output_2_blinding",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.core.dex.v1alpha1.SwapClaimPlan.esk_1", AS_HEX),
    (".penumbra.core.dex.v1alpha1.SwapClaimPlan.esk_2", AS_HEX),
    // Transaction formatting
    (
        ".penumbra.core.transaction.v1alpha1.Transaction.binding_sig",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.Output.proof",
        AS_BASE64_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.OutputBody.wrapped_memo_key",
        AS_BASE64_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.OutputBody.ovk_wrapped_key",
        AS_BASE64_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.Spend.proof",
        AS_BASE64_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.SpendBody.rk",
        AS_HEX_FOR_BYTES,
    ),
    (
        ".penumbra.core.transaction.v1alpha1.SpendBody.nullifier",
        AS_HEX_FOR_BYTES,
    ),
    (".penumbra.core.dex.v1alpha1.Swap.proof", AS_BASE64),
    (".penumbra.core.dex.v1alpha1.SwapClaim.proof", AS_BASE64),
    (".penumbra.core.dex.v1alpha1.Position.nonce", AS_HEX),
    (
        ".penumbra.core.dex.v1alpha1.PositionId.inner",
        AS_BECH32_LP_ID,
    ),
    // Proposal JSON formatting
    (
        ".penumbra.core.transaction.v1alpha1.Proposal.payload",
        SERDE_FLATTEN,
    ),
    (
        // IMPORTANT: this *lacks* a leading dot, to work around a bug in prost-build:
        // https://github.com/tokio-rs/prost/issues/504
        "penumbra.core.transaction.v1alpha1.Proposal.Payload.payload",
        SERDE_FLATTEN,
    ),
    (
        // see above re: prost issue #504
        "penumbra.core.governance.v1alpha1.ProposalState.state",
        SERDE_FLATTEN,
    ),
    (
        // see above re: prost issue #504
        "penumbra.core.governance.v1alpha1.ProposalState.Finished.outcome",
        SERDE_FLATTEN,
    ),
    (
        // see above re: prost issue #504
        "penumbra.core.governance.v1alpha1.ProposalOutcome.outcome",
        SERDE_FLATTEN,
    ),
    (
        "penumbra.core.governance.v1alpha1.ProposalOutcome.Failed.withdrawn_with_reason",
        SERDE_SKIP_NONE,
    ),
    (
        "penumbra.core.governance.v1alpha1.ProposalOutcome.Vetoed.withdrawn_with_reason",
        SERDE_SKIP_NONE,
    ),
    // see above re: prost issue #504
    ("penumbra.core.governance.v1alpha1.Vote.vote", AS_VOTE),
    (
        ".penumbra.core.transaction.v1alpha1.AuthHash.inner",
        AS_HEX_FOR_BYTES,
    ),
];
