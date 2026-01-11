# Compliance System - Local Devnet Tests

This document describes how to test the Penumbra compliance system on a local devnet.

## Overview

The compliance system has three types of assets:

| Asset Type | Registry Status | Compliance Behavior |
|------------|-----------------|---------------------|
| **Regulated** | Registered with `--regulated` | Ciphertexts encrypted to user's WCK. Scannable by registered users. |
| **Unregulated** | Registered with `--unregulated` | Ciphertexts encrypted to BLACK_HOLE. Nobody can scan. |
| **Unregistered** | Not in registry | Transfers **FAIL** - cannot prove asset status |

## Prerequisites

```bash
# Build binaries
cargo build --release -p pd -p pcli

# Make scripts executable
chmod +x scripts/compliance-*.sh
```

---

## Quick Start

```bash
# 1. Run setup (creates wallets, registers assets/users)
./scripts/compliance-setup.sh

# 2. Run test scenarios (after setup completes)
./scripts/compliance-test-regulated.sh      # Scenario 1
./scripts/compliance-test-unregulated.sh    # Scenario 2
./scripts/compliance-test-unregistered.sh   # Scenario 3
```

---

## Setup Script

**Script**: `./scripts/compliance-setup.sh`

The setup script:
1. Creates three wallets: Alice, Bob, Oscar
2. Regenerates devnet with Alice as allocation address (so she has funds)
3. Registers `penumbra` as **REGULATED** asset
4. Registers `test_usd` as **UNREGULATED** asset
5. Registers Alice and Bob for the regulated asset
6. Does **NOT** register Oscar (he's an outsider)
7. Does **NOT** register `unknown_token` (for testing unregistered transfers)

**Output**: Creates `/tmp/compliance-demo.env` with all environment variables.

### Running the Setup

```bash
./scripts/compliance-setup.sh
```

The script will pause twice:
1. **First pause**: Stop any running pd/cometbft, then press Enter
2. **Second pause**: Start pd and cometbft (commands shown), then press Enter

**Starting pd and cometbft:**
```bash
# Terminal 1
cd ~/.penumbra/network_data/node0
~/Documents/Source/penumbra/target/release/pd start --home . --grpc-bind 0.0.0.0:8080 --abci-bind 127.0.0.1:26658

# Terminal 2
cometbft start --home ~/.penumbra/network_data/node0/cometbft
```

---

## Scenario 1: Regulated Transfer

**Script**: `./scripts/compliance-test-regulated.sh`

**Goal**: Demonstrate that regulated asset transfers can be scanned by registered users.

### What it does:
1. Alice sends 100 penumbra to Bob
2. Scans with Alice's daily key → **sees the transfer** (she's sender)
3. Scans with Bob's daily key → **sees the transfer** (he's receiver)
4. Scans with Oscar's daily key → **sees NOTHING** (not registered)

### Expected Output:

```
--- Alice's scan (SENDER - should see transfer) ---
Scanning blocks 1 to <N> for regulated transfers...
📋 Detected Transfer at height <N>
   Asset: penumbra
   Amount: 100
   ...
✅ Scan complete. Detected 1 transfer.

--- Bob's scan (RECEIVER - should see transfer) ---
Scanning blocks 1 to <N> for regulated transfers...
📋 Detected Transfer at height <N>
   Asset: penumbra
   Amount: 100
   ...
✅ Scan complete. Detected 1 transfer.

--- Oscar's scan (NOT REGISTERED - should see NOTHING) ---
Scanning blocks 1 to <N> for regulated transfers...
✅ Scan complete. Detected 0 transfers.
```

### Key Concepts:
- **Registered users** (Alice, Bob) can scan their own transfers
- **Unregistered users** (Oscar) cannot decrypt any ciphertexts
- Each user only sees transfers where they are sender OR receiver

---

## Scenario 2: Unregulated Transfer

**Script**: `./scripts/compliance-test-unregulated.sh`

**Goal**: Demonstrate that unregulated asset transfers cannot be scanned by anyone.

### What it does:
1. Attempts to transfer `test_usd` (registered as unregulated)
2. Scans with all users' daily keys → **nobody sees anything**

### Expected Behavior (after Phase 4):
- Transfer succeeds
- Compliance ciphertexts are generated but encrypted to BLACK_HOLE_ACK
- Nobody can decrypt them (maximum privacy)
- On-chain appearance is **identical** to regulated transfers

### Current Limitation:
The planner currently **skips compliance entirely** for unregulated assets. This means:
- No ciphertexts are generated
- Transfers are **distinguishable** from regulated ones

Phase 4 will fix this by always generating ciphertexts (using BLACK_HOLE for unregulated).

---

## Scenario 3: Unregistered Asset Transfer

**Script**: `./scripts/compliance-test-unregistered.sh`

**Goal**: Demonstrate that unregistered asset transfers **FAIL**.

### What it does:
1. Attempts to transfer `unknown_token` (not in registry)
2. Transfer is **REJECTED** by the validator

### Expected Output:

```
Attempting: Alice -> Bob (100 unknown_token)

ERROR: spend asset_anchor does not match chain state
  - body.asset_anchor: 0
  - chain_asset_anchor: <non-zero hash>

Transfer FAILED (as expected)
```

### Why it fails:
1. Planner sets `asset_anchor = 0` (default for unregistered assets)
2. Chain has `asset_anchor = ZERO_HASHES[16]` (empty tree root, non-zero)
3. Validator checks: `body.asset_anchor == chain_asset_anchor`
4. `0 != ZERO_HASHES[16]` → **REJECTED**

### Security Implication:
- Users **cannot bypass compliance** by using unregistered assets
- Asset issuers **must explicitly register** their assets
- This ensures all transfers prove the asset's regulation status

---

## Summary Table

| Scenario | Asset | Registration | Transfer | Scanning |
|----------|-------|--------------|----------|----------|
| 1 | penumbra | Regulated | ✅ Success | Registered users can scan |
| 2 | test_usd | Unregulated | ✅ Success* | Nobody can scan (BLACK_HOLE) |
| 3 | unknown_token | Not registered | ❌ Fails | N/A |

*After Phase 4 implementation

---

## Cleanup

```bash
# Stop pd and cometbft (Ctrl+C in each terminal)

# Remove all test data
rm -rf ~/.local/share/pcli
rm -rf ~/.penumbra/network_data
rm -rf /tmp/alice-wallet /tmp/bob-wallet /tmp/oscar-wallet
rm -f /tmp/compliance-demo.env
```

---

## Manual Commands Reference

If you want to run commands manually instead of using the scripts:

```bash
# Source the environment
source /tmp/compliance-demo.env

# Register an asset
$PCLI --home $ALICE_HOME tx compliance register-asset <asset> --regulated
$PCLI --home $ALICE_HOME tx compliance register-asset <asset> --unregulated

# Register a user for an asset
$PCLI --home $ALICE_HOME tx compliance register-user <asset>

# Make a transfer
$PCLI --home $ALICE_HOME tx send 100penumbra --to "$BOB_ADDRESS"

# Derive a daily key
DATE=$(python3 -c "import time; print(int(time.time() // 86400))")
$PCLI tx compliance derive-daily-key --mck-hex $ALICE_MCK --date $DATE

# Scan for transfers
$PCLI tx compliance scan --node $PENUMBRA_NODE_PD_URL --daily-key-hex <DAILY_KEY>

# Check balances
$PCLI --home $ALICE_HOME view balance
```

---

## Key Derivation Workflow

The compliance system uses a two-step workflow:

### Step 1: Derive Daily Key (Issuer Role)
The asset issuer derives time-limited daily keys from the user's MCK:

```bash
pcli tx compliance derive-daily-key --mck-hex <MCK> --date <DAY_INDEX>
```

### Step 2: Scan with Daily Key (Auditor Role)
The auditor uses the daily key to scan (without needing the MCK):

```bash
pcli tx compliance scan --daily-key-hex <DAILY_KEY> --node <URL>
```

### Benefits:
- **Time-limited access**: Auditors only have access to specific dates
- **MCK protection**: Master Compliance Key never leaves issuer's control
- **Separation of concerns**: Key derivation (issuer) vs scanning (auditor)
