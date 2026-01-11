#!/bin/bash
# Scenario 2: Unregulated Transfer (test_usd)
# Alice->Bob, encrypted to BLACK_HOLE (nobody can scan)
set -e

ENV_FILE=/tmp/compliance-demo.env
[ ! -f "$ENV_FILE" ] && echo "Run compliance-setup.sh first" && exit 1
source "$ENV_FILE"

echo "=== Scenario 2: Unregulated Transfer ==="

# Transfer
set +e
TRANSFER_OUTPUT=$($PCLI --home "$ALICE_HOME" tx send 1000test_usd --to "$BOB_ADDRESS" 2>&1)
TRANSFER_EXIT_CODE=$?
set -e

if [ $TRANSFER_EXIT_CODE -ne 0 ]; then
    echo "ERROR: Transfer failed"
    echo "$TRANSFER_OUTPUT"
    exit 1
fi

# Sync
$PCLI --home "$ALICE_HOME" view sync
$PCLI --home "$BOB_HOME" view sync

# Derive daily keys
DATE=$(python3 -c "import time; print(int(time.time() // 86400))")
ALICE_DAILY=$($PCLI tx compliance derive-daily-key --mck-hex "$ALICE_MCK" --date "$DATE" 2>&1 | grep "Daily Key (hex):" | awk '{print $4}')
BOB_DAILY=$($PCLI tx compliance derive-daily-key --mck-hex "$BOB_MCK" --date "$DATE" 2>&1 | grep "Daily Key (hex):" | awk '{print $4}')

# Scan (both should see NOTHING - BLACK_HOLE encryption)
echo "Alice (should see NOTHING - BLACK_HOLE):"
$PCLI tx compliance scan --node "$PENUMBRA_NODE_PD_URL" --daily-key-hex "$ALICE_DAILY"

echo "Bob (should see NOTHING - BLACK_HOLE):"
$PCLI tx compliance scan --node "$PENUMBRA_NODE_PD_URL" --daily-key-hex "$BOB_DAILY"

# Final balances
echo "=== Final Balances ==="
echo "Alice:" && $PCLI --home "$ALICE_HOME" view balance
echo "Bob:" && $PCLI --home "$BOB_HOME" view balance

echo "=== Scenario 2 Complete ==="
echo "Unregulated transfers use BLACK_HOLE encryption - maximum privacy."
