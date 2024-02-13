# Helm charts for Penumbra

These helm charts are used to deploy test infrastructure via CI.
A given network deployment is composed of three charts:

  * `penumbra-network`, which runs `pd testnet generate` to create genesis
    and configure genesis validators
  * `penumbra-node`, which runs fullnodes joined to the network, and also
    exposes HTTPS frontends so their RPCs are accessible.
  * `penumbra-metrics`, which runs a grafana/prometheus setup scraping
    the metrics endpoints of the nodes and validators, and exposes
    the grafana dashboards over HTTPS.

These charts are posted publicly as a reference.
