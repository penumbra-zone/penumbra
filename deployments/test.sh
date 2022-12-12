#!/bin/bash
# Sanity checks to verify that the k8s cluster is working approximately
# as expected. Use these checks to increase visibility on service behavior.
# Useful during changes to provisioning / config-management logic,
# when run by a human, interactively; not intended to run in CI regularly.
#
# Usage:
#
#   ./test.sh testnet-preview.penumbra.zone
#
set -euo pipefail


TEST_TARGET="${1:-testnet-preview.penumbra.zone}"
RELEASE_NAME="${TEST_TARGET%.penumbra.zone}"

echo "Running tests against host: $TEST_TARGET"
echo "Inferred release name '$RELEASE_NAME' for k8s resources"

error_counter=0

# Ensure the service backends are "healthy".
function get_service_health() {
    ing_name="penumbra-${RELEASE_NAME}-ingress"
    kubectl get ingress "$ing_name" -o json \
        | jq -r '.metadata.annotations["ingress.kubernetes.io/backends"]' \
        | jq -r '.[]' \
        | sort -u
}
printf 'Checking backend service health... '
svc_health="$(get_service_health)"
if [[ "$svc_health" != "HEALTHY" ]] ; then
    >&2 echo "ERROR: service endpoints are not healthy"
    error_counter="$((error_counter + 1))"
else
    echo "OK"
fi

# Ensure all pods are running.
function get_pod_status() {
    kubectl get pods -o json \
        | jq '.items[].status.phase' -r \
        | sort -u
}
printf 'Checking whether pods are running... '
pod_status="$(get_pod_status)"
if [[ "$pod_status" != "Running" ]] ; then
    >&2 echo "ERROR: pods are not all running"
    >&2 echo "output was: '$pod_status'"
    error_counter="$((error_counter + 1))"
else
    echo "OK"
fi

function get_fn_ip() {
    kubectl get svc "penumbra-${RELEASE_NAME}-p2p-fn-0" --output jsonpath='{.status.loadBalancer.ingress[0].ip}'
}
fn_ip="$(get_fn_ip)"
# Ensure DNS matches expectations
printf 'Checking that DNS A record matches expectations... '
if host "$TEST_TARGET" | grep -qF "$TEST_TARGET has address $fn_ip" ; then
    echo "OK"
else
    >&2 echo "ERROR: DNS lookup failed"
    >&2 echo "Expected '$TEST_TARGET' to point to $fn_ip"
    error_counter="$((error_counter + 1))"
fi

# Now we want to ensure that we can connect pcli to pd over 8080/tcp.
pcli view reset || true
printf 'Trying to connect via pcli... '
# TODO: pcli surprisingly prints to stdout; we should make it stderr
if pcli --node "$fn_ip" view sync > /dev/null; then
    echo "OK"
else
    >&2 echo "ERROR: pcli could not connect to '$fn_ip'"
    error_counter="$((error_counter + 1))"
fi

# Error reporting for readability
if [[ "$error_counter" -gt 0 ]] ; then
    echo "Encountered $error_counter errors"
else
    echo "All tests passed!"
fi

exit "$error_counter"
