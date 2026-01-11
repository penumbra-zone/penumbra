#!/bin/bash
# Scenario 1: Regulated Transfer (regulated_usd)
# Alice->Bob, scannable by registered users
set -e

ENV_FILE=/tmp/compliance-demo.env
[ ! -f "$ENV_FILE" ] && echo "Run compliance-setup.sh first" && exit 1
source "$ENV_FILE"

echo "=== Scenario 1: Regulated Transfer ==="

# Transfer regulated_usd (Alice has genesis allocation)
$PCLI --home "$ALICE_HOME" tx send 100regulated_usd --to "$BOB_ADDRESS"

# Sync
$PCLI --home "$ALICE_HOME" view sync
$PCLI --home "$BOB_HOME" view sync
$PCLI --home "$OSCAR_HOME" view sync

# Derive daily keys
DATE=$(python3 -c "import time; print(int(time.time() // 86400))")
ALICE_DAILY=$($PCLI tx compliance derive-daily-key --mck-hex "$ALICE_MCK" --date "$DATE" 2>&1 | grep "Daily Key (hex):" | awk '{print $4}')
BOB_DAILY=$($PCLI tx compliance derive-daily-key --mck-hex "$BOB_MCK" --date "$DATE" 2>&1 | grep "Daily Key (hex):" | awk '{print $4}')
OSCAR_DAILY=$($PCLI tx compliance derive-daily-key --mck-hex "$OSCAR_MCK" --date "$DATE" 2>&1 | grep "Daily Key (hex):" | awk '{print $4}')

# Scan
echo "Alice (sender - should see transfer):"
$PCLI tx compliance scan --node "$PENUMBRA_NODE_PD_URL" --daily-key-hex "$ALICE_DAILY"

echo "Bob (receiver - should see transfer):"
$PCLI tx compliance scan --node "$PENUMBRA_NODE_PD_URL" --daily-key-hex "$BOB_DAILY"

echo "Oscar (not registered - should see NOTHING):"
$PCLI tx compliance scan --node "$PENUMBRA_NODE_PD_URL" --daily-key-hex "$OSCAR_DAILY"

# Final balances
echo "=== Final Balances ==="
echo "Alice:" && $PCLI --home "$ALICE_HOME" view balance
echo "Bob:" && $PCLI --home "$BOB_HOME" view balance

echo "=== Scenario 1 Complete ==="
