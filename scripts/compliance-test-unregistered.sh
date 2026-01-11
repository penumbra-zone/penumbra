#!/bin/bash
# Scenario 3: Unregistered Asset Transfer (unknown_token)
# Should FAIL - asset not registered
set -e

ENV_FILE=/tmp/compliance-demo.env
[ ! -f "$ENV_FILE" ] && echo "Run compliance-setup.sh first" && exit 1
source "$ENV_FILE"

echo "=== Scenario 3: Unregistered Asset Transfer ==="

# Attempt transfer (should fail)
set +e
TRANSFER_OUTPUT=$($PCLI --home "$ALICE_HOME" tx send 1000unknown_token --to "$BOB_ADDRESS" 2>&1)
TRANSFER_EXIT_CODE=$?
set -e

if [ $TRANSFER_EXIT_CODE -ne 0 ]; then
    echo "Transfer failed as expected:"
    echo "$TRANSFER_OUTPUT" | grep -i "not registered" || echo "$TRANSFER_OUTPUT"
else
    echo "ERROR: Transfer succeeded unexpectedly!"
    exit 1
fi

echo "=== Scenario 3 Complete ==="
