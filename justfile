build:
    podman build -t harbor.ruin.dev/library/penumbra:debug-witness-and-build -f ./deployments/containerfiles/Containerfile-strangelove-penumbra .
    podman push harbor.ruin.dev/library/penumbra:debug-witness-and-build
