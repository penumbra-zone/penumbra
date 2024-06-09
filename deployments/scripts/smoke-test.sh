#!/usr/bin/env bash
# Run smoke test suite, via process-compose config.
set -euo pipefail


# Fail fast if testnet dir exists, otherwise `cargo run ...` will block
# for a while, masking the error.
if [[ -d ~/.penumbra/testnet_data ]] ; then
    >&2 echo "ERROR: testnet data directory exists at ~/.penumbra/testnet_data"
    >&2 echo "Not removing this directory automatically; to remove, run: pd testnet unsafe-reset-all"
    exit 1
fi

if ! hash cometbft > /dev/null 2>&1 ; then
    >&2 echo "ERROR: cometbft not found in PATH"
    >&2 echo "See install guide: https://guide.penumbra.zone/main/pd/build.html"
    exit 1
fi

if ! hash process-compose > /dev/null 2>&1 ; then
    >&2 echo "ERROR: process-compose not found in PATH"
    >&2 echo "Install it via https://github.com/F1bonacc1/process-compose/"
    exit 1
fi

if ! hash grpcurl > /dev/null 2>&1 ; then
    >&2 echo "ERROR: grpcurl not found in PATH"
    >&2 echo "Install it via https://github.com/fullstorydev/grpcurl/"
    exit 1
fi

# Check for interactive terminal session, enable TUI if yes.
if [[ -t 1 ]] ; then
    use_tui="true"
else
    use_tui="false"
fi

repo_root="$(git rev-parse --show-toplevel)"
# Override the pc API port 8080 -> 9191, to avoid conflict with pd.
if ! process-compose --config deployments/compose/process-compose-smoke-test.yml --port 9191 -t="$use_tui" ; then
    >&2 echo "ERROR: smoke tests failed"
    >&2 echo "Review logs in: deployments/logs/smoke-*.log"
    find "${repo_root}/deployments/logs/smoke-"*".log" | sort >&2
    exit 1
else
    echo "SUCCESS! Smoke test complete."
fi
