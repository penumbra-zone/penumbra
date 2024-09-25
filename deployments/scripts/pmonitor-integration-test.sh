#!/bin/bash
# quick script to test the `pmonitor` tool during review
# set -euo pipefail
set -eu

>&2 echo "Preparing pmonitor test bed..."
num_wallets=10

# ideally we'd use a tempdir but using a hardcoded dir for debugging
# pmonitor_integration_test_dir="$(mktemp -p /tmp -d pmonitor-integration-test.XXXXXX)"
pmonitor_integration_test_dir="/tmp/pmonitor-integration-test"
rm -rf "$pmonitor_integration_test_dir"
mkdir "$pmonitor_integration_test_dir"

pmonitor_home="${pmonitor_integration_test_dir}/pmonitor"
wallets_dir="${pmonitor_integration_test_dir}/wallets"
wallet_addresses="${pmonitor_integration_test_dir}/addresses.txt"
allocations_csv="${pmonitor_integration_test_dir}/pmonitor-test-allocations.csv"
fvks_json="${pmonitor_integration_test_dir}/fvks.json"
cargo run --release --bin pd -- network unsafe-reset-all || true
cargo run --release --bin pmonitor -- reset || true
mkdir "$wallets_dir"
# override process-compose default port of 8080, which we use for pd
export PC_PORT_NUM="8888"
process-compose down || true

>&2 echo "creating pcli wallets"
for i in $(seq 1 "$num_wallets"); do
  yes | cargo run -q --release --bin pcli -- --home "${wallets_dir}/wallet-$i" init --grpc-url http://localhost:8080 soft-kms generate
done

# collect addresses
>&2 echo "collecting pcli wallet addresses"
for i in $(seq 1 "$num_wallets"); do
  cargo run -q --release --bin pcli -- --home "${wallets_dir}/wallet-$i" view address
done > "$wallet_addresses"


# generate genesis allocations
>&2 echo "generating genesis allocations"
printf 'amount,denom,address\n' > "$allocations_csv"
while read -r a ; do
  printf '1_000_000__000_000,upenumbra,%s\n1000,test_usd,%s\n' "$a" "$a"
done < "$wallet_addresses" >> "$allocations_csv"

# generate network data
>&2 echo "generating network data"
cargo run --release --bin pd -- network generate \
  --chain-id penumbra-devnet-pmonitor \
  --unbonding-delay 50 \
  --epoch-duration 50 \
  --proposal-voting-blocks 50 \
  --timeout-commit 3s \
  --gas-price-simple 500 \
  --allocations-input-file "$allocations_csv"

# run network
>&2 echo "running local devnet"
process-compose up --detached --config deployments/compose/process-compose.yml

# ensure network is torn down afterward; comment this out if you want
# to interact with the network after tests complete.
trap 'process-compose down || true' EXIT

# wait for network to come up; lazily sleeping, rather than polling process-compose for "ready" state
sleep 8

>&2 echo "collecting fvks"
fd config.toml "$wallets_dir" -x toml get {} full_viewing_key | jq -s > "$fvks_json"

>&2 echo "initializing pmonitor"
cargo run --release --bin pmonitor -- \
  --home "$pmonitor_home" \
  init --fvks "$fvks_json" --grpc-url http://localhost:8080

>&2 echo "running pmonitor audit"
# happy path: we expect this audit to exit 0, because no transfers have occurred yet
cargo run --release --bin pmonitor -- \
  --home "$pmonitor_home" \
  audit

>&2 echo "exiting BEFORE misbehavior"
exit 0



>&2 echo "committing misbehavior"
alice_wallet="${wallets_dir}/wallet-alice"
yes | cargo run --quiet --release --bin pcli -- --home "$alice_wallet" init --grpc-url http://localhost:8080 soft-kms generate
alice_address="$(cargo run --quiet --release --bin pcli -- --home "$alice_wallet" view address)"
misbehaving_wallet="${wallets_dir}/wallet-2"
cargo run --quiet --release --bin pcli -- --home "$misbehaving_wallet" tx send --memo "take these tokens, but tell no one" 500penumbra --to "$alice_address"

>&2 echo "re-running pmonitor audit"
# unhappy path: we expect this audit to exit 10, because a transfer occurred from a monitored wallet
# TODO: make pmonitor exit non-zero when there's bad misbehavior
cargo run --release --bin pmonitor -- \
  --home "$pmonitor_home" \
  audit | tee "${wallets_dir}/pmonitor-log-1.txt"

printf '#################################\n'
printf 'PMONITOR INTEGRATION TEST SUMMARY\n'
printf '#################################\n'

if grep -q "Unexpected balance! Balance is less than the genesis balance" "${wallets_dir}/pmonitor-log-1.txt" ; then
  >&2 echo "OK: 'pmonitor audit' reported unexpected balance, due to misbehavior"
else
  >&2 echo "ERROR: 'pmonitor audit' failed to identify misbehavior, which we know occurred"
  exit 1
fi
