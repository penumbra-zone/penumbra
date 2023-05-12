#!/usr/bin/env bash

set -euo pipefail

PREVIOUS_TESTNET_DIRECTORY=$(find . -mindepth 1 -type d | tail -n1)
PREVIOUS_TESTNET_NUMBER=$(find . -mindepth 1 -type d | wc -l)
NEW_TESTNET_NUMBER="0$(echo "1 + $PREVIOUS_TESTNET_NUMBER" | bc)"
NEW_TESTNET_DIRECTORY="$NEW_TESTNET_NUMBER-$(head -"$NEW_TESTNET_NUMBER" < names.txt | tail -1 | tr '[:upper:]' '[:lower:]')"

echo "Creating new testnet directory $NEW_TESTNET_DIRECTORY..."

mkdir "$NEW_TESTNET_DIRECTORY"
cp "$PREVIOUS_TESTNET_DIRECTORY/validators.json" "$NEW_TESTNET_DIRECTORY/validators.json"

echo "Setting up allocations for new testnet..."

cut -d , -f 6 < discord_history.csv \
    | tail -n +2 | sort | uniq \
    | grep ^penumbrav \
    | awk '{ printf "\"1_000__000_000\",\"upenumbra\",\""; printf $1; print "\""; printf "\"1_000\",\"test_usd\",\""; printf $1; print "\""; }' \
    | cat base_allocations.csv - > "$NEW_TESTNET_DIRECTORY/allocations.csv"

echo "All done!"
