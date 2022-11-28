module "gcp_terraform_state_testnet" {
  source = "../../terraform/modules/gcp/terraform_state/chain"

  chain_name          = "penumbra"
  labels              = {}
  location            = "US"
  network_environment = "testnet"
}

module "gke_testnet" {
  source = "../../terraform/modules/node/v1"

  project_id    = "penumbra-sl-testnet"
  cluster_name  = "testnet"
  region        = "us-central1"
  cluster_zones = ["us-central1-a", "us-central1-b"]
  machine_type  = "n2d-standard-4"
}

// The IPv4 address used for URLs like `rpc.testnet.penumbra.zone`.
// We reserve this ahead of time, so that the DNS records can be static,
// and point to the currently running deployment.
resource "google_compute_global_address" "testnet-ingress" {
  // The 'name' field must match the name referred to in helm chart.
  // By default it's "penumbra.name" which is either "penumbra-testnet"
  // or "penumbra-testnet-preview".
  name    = "penumbra-testnet-ingress-ip"
  project = "penumbra-sl-testnet"
}

// Declare 'output' so the reserved IP is easily viewable.
output "testnet_reserved_ip" {
  value = google_compute_global_address.testnet-ingress.address
}

// There's another DNS record required, that doesn't map to the static IP.
// It's a NodePort service, so it must match the ExternalIP of a given node.
// For reference:
//
//   kubectl get svc penumbra-testnet-p2p-fn-0 \
//       --output jsonpath='{.status.loadBalancer.ingress[0].ip}'
//
// The resulting IP should have an A record for "fullnode.<network>.penumbra.zone".
