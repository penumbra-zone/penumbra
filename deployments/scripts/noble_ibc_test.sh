#!/usr/bin/env bash
# This script automates the process of testing Noble IBC transfers.
# It assumes it'll be run from the repo root of the Noble project [0].
# It's not used in CI but may be useful in validating fixes related to GH4899.
#
# [0] https://github.com/noble-assets/noble
set -euo pipefail


# Constants
NOBLE_CHAIN_ID="grand-1"
PENUMBRA_CHAIN_ID="penumbra-testnet-phobos-2"
NODE_URL="https://noble-testnet-rpc.polkachu.com:443"
ESCAPED_URL=$(echo "$NODE_URL" | sed 's/\//\\\//g')

# To hardcode mnemonic:
# echo -n "mnemonic words go here" > ./mnemonic.txt
# (and remove the line that generates the mnemonic below)

if [[ -d "$HOME/.noble" ]]; then
    echo "$HOME/.noble config directory already exists. Please remove this directory and try again." && exit 1
fi

# Ensure jq is installed
type -P "jq" > /dev/null 2>&1 || (echo "jq is not installed. Please install jq and try again." && exit 1)

# Ensure pcli is installed
type -P "pcli" > /dev/null 2>&1 || (echo "pcli is not installed. Please install pcli and try again." && exit 1)

# Ensure the script is executing from the Noble checkout
if [ ! -f "./app.go" ] || [ ! -f "./Makefile" ]; then
    pwd
    echo "it appears you aren't in a Noble checkout directory. Please check out the Noble repository (https://github.com/noble-assets/noble/) and try again." && exit 1
fi

# Run the Noble Makefile
make && make install

# Verify the build script was created by the Makefile:
if [ ! -f "./build" ]; then
    echo "The build script was not created. Please check that 'make' can run successfully and try again." && exit 1
fi

# Generate the initial configuration
./build init --chain-id "$NOBLE_CHAIN_ID" "$NOBLE_CHAIN_ID"

# Insert the correct RPC address in the configuration
# sed -i '' 's/node = ""/node = "https:\/\/noble-testnet-rpc.polkachu.com:443"/g' ~/.noble/config/client.toml
sed -i '' "s/node = \".*\"/node = \"$ESCAPED_URL\"/g" ~/.noble/config/client.toml
sed -i '' "s/chain-id = \".*\"/chain-id = \"$NOBLE_CHAIN_ID\"/g" ~/.noble/config/client.toml

# Generate the Noble key mnemonic
./build keys mnemonic | tr -d '\n' > ./mnemonic.txt

# Remove existing "test_key" if present
set +e # Disable error checking because this command will fail if the key doesn't exist
./build keys delete test_key -y
set -e  # Re-enable error checking

# Add a key named "test_key" from the mnemonic to the keyring
./build keys add test_key --source ./mnemonic.txt --recover

# Check address
NOBLE_ADDRESS=$(./build keys show test_key --output json | jq '.address' | sed 's/"//g')
printf 'Noble address: %s\n\n' "$NOBLE_ADDRESS"

# Ensure address has funds
INITIAL_NOBLE_BALANCE=$(./build query bank balance "$NOBLE_ADDRESS" uusdc --output json | jq '.balance.amount' | sed 's/"//g')
echo -e "Initial Balance of Noble address: $INITIAL_NOBLE_BALANCE\n"

if [ "$INITIAL_NOBLE_BALANCE" -eq 0 ]; then
    echo "The address $NOBLE_ADDRESS has no funds. Please send some funds to this address and try again." && exit 1
fi

# Check Penumbra starting balance
INITIAL_PENUMBRA_BALANCE=$(pcli view balance | grep uusdc | awk '{ print $3 }' | \
awk '
{
    match($0, /^[0-9]+/)
    sum += substr($0, RSTART, RLENGTH)
}
END {
    print sum
}')

echo "Initial Penumbra balance: $INITIAL_PENUMBRA_BALANCE"

# There are probably many client IDs created for this chain ID.
# Grab the last (highest numbered) unfrozen one.
CLIENT_ID=$(./build query ibc client states --node "$NODE_URL" --limit 10000 --output json | jq --arg chain_id "$PENUMBRA_CHAIN_ID" '[.client_states[] | select(
    (.client_state.chain_id? | strings | contains($chain_id)) and
    .client_state.frozen_height.revision_number == "0" and
    .client_state.frozen_height.revision_height == "0"
) | .client_id] | last' | sed 's/"//g')

# Grab the last open connection ID associated with the client ID
CONNECTION_ID=$(./build query ibc connection connections --node "$NODE_URL" --limit 10000 --output json | jq --arg client_id "$CLIENT_ID" '[.connections[] | select(
    .state == "STATE_OPEN" and
    .client_id == $client_id
) | .id] | last' | sed 's/"//g')

# Find the Noble channel ID associated with the connection ID
CHANNEL_ID="$(./build query ibc channel connections "$CONNECTION_ID" --node "$NODE_URL" --limit 10000 --output json | jq '[.channels[] | select(
    .state == "STATE_OPEN" and
    .port_id == "transfer"
) | .channel_id] | last' | sed 's/"//g')"
# Find the counterparty (Penumbra) channel ID associated with the connection ID
COUNTERPARTY_CHANNEL_ID="$(./build query ibc channel connections "$CONNECTION_ID" --node "$NODE_URL" --limit 10000 --output json | jq '[.channels[] | select(
    .state == "STATE_OPEN" and
    .port_id == "transfer"
) | .counterparty.channel_id] | last' | sed 's/"//g')"

# Get both compat address and normal address, and test both
PENUMBRA_COMPAT_ADDRESS="$(pcli view address --compat)"
PENUMBRA_ADDRESS="$(pcli view address)"

# Send a test transfer to the normal address
echo -e "Submitting IBC transfer to $PENUMBRA_ADDRESS...\n"
./build tx ibc-transfer transfer transfer "$CHANNEL_ID" "$PENUMBRA_ADDRESS" 1uusdc --from "$NOBLE_ADDRESS" --chain-id "$NOBLE_CHAIN_ID" -y | tee /dev/stderr | grep -q "code: 0" || {
    echo -e "Submitting test transfer transaction to $PENUMBRA_ADDRESS failed"
    exit 1
}
echo -e "\nSuccessfully submitted transaction\n"

sleep 5

echo -e "Submitting IBC transfer to $PENUMBRA_COMPAT_ADDRESS...\n"
# Send a test transfer to the compat address
./build tx ibc-transfer transfer transfer "$CHANNEL_ID" "$PENUMBRA_COMPAT_ADDRESS" 1uusdc --from "$NOBLE_ADDRESS" --chain-id "$NOBLE_CHAIN_ID" -y | tee /dev/stderr | grep -q "code: 0" || {
    echo -e "Submitting test transfer transaction to $PENUMBRA_COMPAT_ADDRESS failed"
    exit 1
}
echo -e "\nSuccessfully submitted transaction\n"

sleep 20

# Check the final balances of Noble and Penumbra
FINAL_NOBLE_BALANCE=$(./build query bank balance "$NOBLE_ADDRESS" uusdc --output json | jq '.balance.amount' | sed 's/"//g')
echo -e "Final Balance of Noble address: $FINAL_NOBLE_BALANCE\n"

FINAL_PENUMBRA_BALANCE=$(pcli view balance | grep uusdc | awk '{ print $3 }' | \
awk '
{
    match($0, /^[0-9]+/)
    sum += substr($0, RSTART, RLENGTH)
}
END {
    print sum
}')

echo "Final Penumbra balance: $FINAL_PENUMBRA_BALANCE"


if ((FINAL_NOBLE_BALANCE != INITIAL_NOBLE_BALANCE - 2)); then
    echo "FINAL_NOBLE_BALANCE must be 2 less than INITIAL_NOBLE_BALANCE" && exit 1
fi

if ((FINAL_PENUMBRA_BALANCE != INITIAL_PENUMBRA_BALANCE + 2)); then
    echo "FINAL_PENUMBRA_BALANCE must be 2 greater than INITIAL_PENUMBRA_BALANCE" && exit 1
fi

# Now try transferring back in the other direction (Penumbra -> Noble)

# Initial balance for this test is the final we just obtained
INITIAL_NOBLE_BALANCE=$FINAL_NOBLE_BALANCE
INITIAL_PENUMBRA_BALANCE=$FINAL_PENUMBRA_BALANCE
unset FINAL_NOBLE_BALANCE
unset FINAL_PENUMBRA_BALANCE

PENUMBRA_CHANNEL_NUMBER=${COUNTERPARTY_CHANNEL_ID#*-}
pcli tx withdraw --to "$NOBLE_ADDRESS" --channel "$PENUMBRA_CHANNEL_NUMBER" "1transfer/${COUNTERPARTY_CHANNEL_ID}/uusdc"

sleep 20

# Ensure the transfer landed on the Noble side:
FINAL_NOBLE_BALANCE=$(./build query bank balance "$NOBLE_ADDRESS" uusdc --output json | jq '.balance.amount' | sed 's/"//g')
echo -e "Final Balance of Noble address after transfer from Penumbra: $FINAL_NOBLE_BALANCE\n"

if ((FINAL_NOBLE_BALANCE != INITIAL_NOBLE_BALANCE + 1)); then
    echo "FINAL_NOBLE_BALANCE must be 1 more than INITIAL_NOBLE_BALANCE" && exit 1
fi
