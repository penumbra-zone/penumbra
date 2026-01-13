# Penumbra Compliance System

Privacy-preserving compliance for regulated assets. Authorized parties (issuers, auditors) can scan transactions for regulated assets without compromising privacy of other transactions.

## Status

**POC** - Proof of Concept on branch `Antoine/global-cvk`

## Documentation

| Section | Description |
|---------|-------------|
| [Architecture](architecture/) | Key hierarchy, ciphertext design, registries |
| [Integration](integration/) | Transaction flow, component compatibility, compliance flow |
| [Roadmap](roadmap/) | Features grouped by priority for GitHub issues |

## CLI Quick Reference

```bash
# Asset registration (governance/issuer)
pcli tx compliance register-asset <asset> --regulated
pcli tx compliance register-asset <asset> --unregulated

# User registration (wallet)
pcli tx compliance register-user <asset>

# Key derivation (issuer -> auditor)
pcli tx compliance derive-daily-key --mck-hex <hex> --date <day_index>

# Scanning (auditor)
pcli tx compliance scan --daily-key-hex <hex> --node <url>
```

## Testing

### Unit & Integration Tests

```bash
# Unit tests
cargo test -p penumbra-sdk-compliance --lib

# Integration tests
cargo test -p penumbra-sdk-app-tests --test compliance_full_flow

# Planner tests
cargo test -p penumbra-sdk-view --lib planner::tests
```

### Local Devnet Tests

End-to-end tests on a local devnet.

```bash
# Prerequisites
cargo build --release -p pd -p pcli
chmod +x scripts/compliance-*.sh

# Run setup (creates wallets, registers assets/users)
./scripts/compliance-setup.sh

# Run test scenarios
./scripts/compliance-test-regulated.sh      # Regulated asset transfers
./scripts/compliance-test-unregulated.sh    # Unregulated (BLACK_HOLE) transfers
./scripts/compliance-test-unregistered.sh   # Unregistered asset (should FAIL)
```

#### Test Scenarios

| Scenario | Asset | Registration | Transfer | Scanning |
|----------|-------|--------------|----------|----------|
| 1 | penumbra | Regulated | Success | Registered users can scan |
| 2 | test_usd | Unregulated | Success | Nobody can scan (BLACK_HOLE) |
| 3 | unknown_token | Not registered | **FAILS** | N/A |

## Key Files

| Component | Location |
|-----------|----------|
| Key Hierarchy | `crates/core/keys/src/keys/cvk.rs` |
| Encryption | `crates/core/component/compliance/src/crypto.rs` |
| Registry | `crates/core/component/compliance/src/registry.rs` |
| Planner | `crates/view/src/planner.rs` |
| SpendPlan | `crates/core/component/shielded-pool/src/spend/plan.rs` |
| OutputPlan | `crates/core/component/shielded-pool/src/output/plan.rs` |
