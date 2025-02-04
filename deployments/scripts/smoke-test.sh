#!/usr/bin/env bash
# Run smoke test suite, via process-compose config.
set -euo pipefail


# Fail fast if network dir exists, otherwise `cargo run ...` will block
# for a while, masking the error.
#
# If any network data is present, we shouldn't reuse it: the smoke tests assume
# a fresh devnet has been created specifically for the test run. In the future
# we should make this a temp dir so it can always run regardless of pre-existing state.
repo_root="$(git rev-parse --show-toplevel)"
"${repo_root}/deployments/scripts/warn-about-pd-state"

# Check for dependencies. All of these will be installed automatically
# as part of the nix env.
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

>&2 echo "Building all test targets before running smoke tests..."
# We want a warm cache before the tests run
cargo build --release -p pcli -p pclientd -p pd

# Reuse existing dev-env script
# Temporary: cannot set ``--config ./deployments/compose/process-compose-postgres.yml`
# because pindexer is currently failing due to GH4999.
if ! "${repo_root}/deployments/scripts/run-local-devnet.sh" \
        --config ./deployments/compose/process-compose-metrics.yml \
        --config ./deployments/compose/process-compose-dev-tooling.yml \
        --config ./deployments/compose/process-compose-smoke-test.yml \
        ; then
    >&2 echo "ERROR: smoke tests failed"
    >&2 echo "Review logs in: deployments/logs/smoke-*.log"
    find "${repo_root}/deployments/logs/smoke-"*".log" | sort >&2
    exit 1
else
    echo "SUCCESS! Smoke test complete."
fi
