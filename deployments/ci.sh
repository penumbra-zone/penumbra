#!/bin/bash
# Utility script to deploy Penumbra testnet(s) to k8s,
# used as part of CI. At a high level, this script does the following:
#
#  * reads env vars (e.g. from github actions) to set helm values
#  * runs a container with `pd testnet generate` to create genesis
#  * munges the generated data into valid (but internal) peer strings
#  * deploys helm chart to kubernetes cluster, replacing running pods
#  * waits a while, then fetches the public ip addresses
#  * re-munges the generated data into publicly-routable peer strings
#  * re-deploys the helm chart to overwrite the config
#
set -euo pipefail

# The following env vars can be used to override config fars
# for the helm chart. N.B. these env vars are also configured
# in GitHub Actions, so the values below may be out of date.
IMAGE="${IMAGE:-ghcr.io/penumbra-zone/penumbra}"
PENUMBRA_VERSION="${PENUMBRA_VERSION:-main}"
# Default to bespoke devnet for deployments; less likely to break public testnets.
# Useful for running ad-hoc via CLI. The workflows override this for testnet/preview.
HELM_RELEASE="${HELM_RELEASE:-penumbra-devnet}"

# Check that the network we're trying to configure has a valid config.
HELMFILE_MANIFEST="./helmfile.d/${HELM_RELEASE}.yaml"
if [[ ! -e "$HELMFILE_MANIFEST" ]]; then
    >&2 echo "ERROR: helm release name '$HELM_RELEASE' not supported"
    >&2 echo "Consider creating '$HELMFILE_MANIFEST'"
    exit 1
fi

# Remove existing deployment and associated storage. Intended to omit removal
# of certain durable resources, such as LoadBalancer and ManagedCertificate.
# We intentionally don't use "helm uninstall" because GCP takes a while
# to propagate ingress recreation, causing delays in endpoint availability.
function helm_uninstall() {
    # Use helm uninstall to purge all managed resources.
    # grep will return non-zero if no matches are found, so disable pipefail
    set +o pipefail
    helm list --filter "^${HELM_RELEASE}" -o json | jq -r '.[].name' | grep -v metrics \
        | xargs -r helm uninstall --wait
    set -o pipefail
    # Follow up with a specific task to remove PVCs.
    kubectl delete jobs -l app.kubernetes.io/part-of="$HELM_RELEASE" --wait=true
    kubectl delete pvc -l app.kubernetes.io/part-of="$HELM_RELEASE" --wait=true
}

# Apply the Helm configuration to the cluster. Will overwrite resources
# as necessary. Will *not* replace certain durable resources like
# the LoadBalancer Service objects, which are annotated with helm.sh/resource-policy=keep.
function helm_install() {
    # TODO: make sure helmfile is present in ci environemnt.
    helmfile sync -f "$HELMFILE_MANIFEST" --args \
        --set="image.tag=${PENUMBRA_VERSION}"
}

function wait_for_pods_to_be_running() {
    echo "Waiting for pods to be running..."
    kubectl wait --for=condition=ready pods --timeout=5m \
        -l app.kubernetes.io/part-of="$HELM_RELEASE"
}

# Deploy a fresh testnet, destroying all prior chain state with new genesis.
function full_ci_rebuild() {
    echo "Shutting down existing testnet if necessary..."
    helm_uninstall
    # Wait a bit longer, to ensure that no lingering references are left in the API.
    sleep 20

    echo "Installing latest config..."
    helm_install
    # Wait longer, because even though we used `--wait`, some resources will still be coming up.
    sleep 20

    # Report results
    if wait_for_pods_to_be_running ; then
        echo "Deploy complete!"
    else
        echo "ERROR: pods failed to enter running start. Deploy has failed."
        return 1
    fi
}

# Determine whether the version to be deployed constitutes a semver "patch" release,
# e.g. 0.1.2 -> 0.1.3.
function is_patch_release() {
    # Ensure version format is semver, otherwise fail.
    if ! echo "$PENUMBRA_VERSION" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+' ; then
        return 1
    fi

    # Split on '.', inspect final field.
    z="$(perl -F'\.' -lanE 'print $F[-1]' <<< "$PENUMBRA_VERSION")"
    # If "z" in x.y.z is 0, then it's a minor release. (Or a major release,
    # but we don't need to worry about that yet.)
    if [[ $z = "0" ]] ; then
        return 2
    else
        return 0
    fi
}

# Bump the version of pd running for the deployment, across all
# fullnodes and validators. Allow the cluster to reconcile the changes
# by terminating and creating pods to match. Does *not* alter chain state.
# Allows us to handle "patch" versions.
function update_image_for_running_deployment() {
    kubectl set image deployments \
        -l "app.kubernetes.io/part-of=${HELM_RELEASE}, app.kubernetes.io/component in (fullnode, genesis-validator)" \
        "pd=${IMAGE}:${PENUMBRA_VERSION}"
    # Wait for rollout to complete. Will block until pods are marked Ready.
    kubectl rollout status deployment \
        -l "app.kubernetes.io/part-of=${HELM_RELEASE}, app.kubernetes.io/component in (fullnode, genesis-validator)"
}

function main() {
    echo "Deploying network '${HELM_RELEASE}'..."
    # TODO: to deploy older versions, e.g. v0.53.1, an override is necessary here
    if is_patch_release ; then
        echo "Release target '$PENUMBRA_VERSION' is a patch release; will preserve testnet while bumping version."
        update_image_for_running_deployment
    else
        echo "Release target '$PENUMBRA_VERSION' requires a full re-deploy; will generate new testnet chain info."
        full_ci_rebuild
    fi
}

main
