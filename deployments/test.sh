#!/bin/bash
# Sanity checks to verify that the k8s cluster is working approximately
# as expected. Use these checks to increase visibility on service behavior.
# Useful during changes to provisioning / config-management logic,
# when run by a human, interactively; not intended to run in CI regularly.
set -euo pipefail

# Ensure the service backends are "healthy".
function get_service_health() {
    kubectl get ingress penumbra-testnet-ingress -o json \
        | jq -r '.metadata.annotations["ingress.kubernetes.io/backends"]' \
        | jq -r '.[]' \
        | sort -u
}
printf 'Checking backend service health... '
svc_health="$(get_service_health)"
if [[ "$svc_health" != "HEALTHY" ]] ; then
    >&2 echo "ERROR: service endpoints are not healthy"
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
else
    echo "OK"
fi
