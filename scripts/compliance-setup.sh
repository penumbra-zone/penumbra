#!/bin/bash
# Compliance Setup - creates wallets, registers assets and users
set -e

repo_root="$(git rev-parse --show-toplevel)"
PCLI="${PCLI:-$repo_root/target/release/pcli}"
PD="${PD:-$repo_root/target/release/pd}"
PENUMBRA_NODE_PD_URL="${PENUMBRA_NODE_PD_URL:-http://localhost:8080}"
export PENUMBRA_NODE_PD_URL

ALICE_HOME=/tmp/alice-wallet
BOB_HOME=/tmp/bob-wallet
OSCAR_HOME=/tmp/oscar-wallet

echo "=== Compliance Setup ==="

# Cleanup
rm -rf ~/.penumbra/network_data "$ALICE_HOME" "$BOB_HOME" "$OSCAR_HOME"

[ ! -f "$PCLI" ] && echo "ERROR: pcli not found" && exit 1
[ ! -f "$PD" ] && echo "ERROR: pd not found" && exit 1

# Init Alice (need address for genesis)
$PCLI --home "$ALICE_HOME" init soft-kms generate
ALICE_ADDRESS=$($PCLI --home "$ALICE_HOME" view address 0)
echo "Alice: $ALICE_ADDRESS"

echo "Stop pd and cometbft if running, then press Enter to generate new network..."
read -r

$PD network generate \
    --chain-id penumbra-local-devnet \
    --unbonding-delay 302400 \
    --epoch-duration 302400 \
    --proposal-voting-blocks 50 \
    --gas-price-simple 0 \
    --timeout-commit 500ms \
    --validators-input-file "$repo_root/testnets/validators-single.json" \
    --allocation-address "$ALICE_ADDRESS"

echo ""
echo "Network generated. Now start pd and cometbft in separate terminals:"
echo "  Terminal 1: pd start --home ~/.penumbra/network_data/node0/pd"
echo "  Terminal 2: cometbft start --home ~/.penumbra/network_data/node0/cometbft"
echo ""
echo "Press Enter when both are running..."
read -r

# Init other wallets
$PCLI --home "$BOB_HOME" init soft-kms generate
$PCLI --home "$OSCAR_HOME" init soft-kms generate

# Sync
$PCLI --home "$ALICE_HOME" view sync
$PCLI --home "$BOB_HOME" view sync
$PCLI --home "$OSCAR_HOME" view sync

BOB_ADDRESS=$($PCLI --home "$BOB_HOME" view address 0)
OSCAR_ADDRESS=$($PCLI --home "$OSCAR_HOME" view address 0)

echo "Bob: $BOB_ADDRESS"
echo "Oscar: $OSCAR_ADDRESS"

# Register assets
echo "=== Registering Assets ==="
# penumbra and test_usd are auto-registered as UNREGULATED at genesis
# We register regulated_usd as REGULATED for compliance testing
echo "Registering regulated_usd as REGULATED..."
$PCLI --home "$ALICE_HOME" tx compliance register-asset regulated_usd --regulated

$PCLI --home "$ALICE_HOME" view sync
$PCLI --home "$BOB_HOME" view sync
$PCLI --home "$OSCAR_HOME" view sync

# Register users for regulated_usd
echo "=== Registering Users ==="
ALICE_OUTPUT=$($PCLI --home "$ALICE_HOME" tx compliance register-user regulated_usd 2>&1) || true
ALICE_MCK=$(echo "$ALICE_OUTPUT" | grep "MCK (hex):" | awk '{print $3}')

BOB_OUTPUT=$($PCLI --home "$BOB_HOME" tx compliance register-user regulated_usd 2>&1) || true
BOB_MCK=$(echo "$BOB_OUTPUT" | grep "MCK (hex):" | awk '{print $3}')

OSCAR_OUTPUT=$($PCLI --home "$OSCAR_HOME" view compliance-key 2>&1) || true
OSCAR_MCK=$(echo "$OSCAR_OUTPUT" | grep "MCK (hex):" | awk '{print $3}')

echo "Alice MCK: ${ALICE_MCK:0:16}..."
echo "Bob MCK: ${BOB_MCK:0:16}..."
echo "Oscar MCK: ${OSCAR_MCK:0:16}... (NOT registered)"

# Save env
cat > /tmp/compliance-demo.env << EOF
export ALICE_HOME="$ALICE_HOME"
export BOB_HOME="$BOB_HOME"
export OSCAR_HOME="$OSCAR_HOME"
export ALICE_ADDRESS="$ALICE_ADDRESS"
export BOB_ADDRESS="$BOB_ADDRESS"
export OSCAR_ADDRESS="$OSCAR_ADDRESS"
export ALICE_MCK="$ALICE_MCK"
export BOB_MCK="$BOB_MCK"
export OSCAR_MCK="$OSCAR_MCK"
export PCLI="$PCLI"
export PENUMBRA_NODE_PD_URL="$PENUMBRA_NODE_PD_URL"
EOF

echo ""
echo "=== Setup Complete ==="
echo "Env: /tmp/compliance-demo.env"
echo "regulated_usd=REGULATED, penumbra=UNREGULATED, test_usd=UNREGULATED"
