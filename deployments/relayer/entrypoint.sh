#!/bin/bash
# Container entrypoint for running an IBC relayer for Penumbra,
# specifically between penumbra-testnet and penumbra-preview.
set -euo pipefail


# We set a custom debug address (default is 5183) to support
# healthchecks determining whether it's running.
# Setting all-interfaces rather than localhost so that k8s
# probes can access the socket.
RELAYER_DEBUG_ADDR="${RELAYER_DEBUG_ADDR:-0.0.0.0:5100}"

# Generate latest configs, polling chain id from RPC endpoints
cd /usr/src/penumbra-relayer || exit 1
./generate-configs preview
./generate-configs testnet

# Generate relayer YAML config, specifying Penumbra path.
./configure-relayer
rly --debug transact link penumbra_path
cat <<EOM
##############################################
Finished configuring the relayer for Penumbra!
Starting service...
##############################################
EOM
# Run the relayer as a blocking service.
exec rly start penumbra_path --debug-addr "$RELAYER_DEBUG_ADDR"
