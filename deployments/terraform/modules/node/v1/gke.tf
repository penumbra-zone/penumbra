provider "kubernetes" {
  host                   = "https://${module.gke.endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(module.gke.ca_certificate)
}

resource "google_project_service" "enable_api_gke" {
  service = "container.googleapis.com"

  // Intentionally prevent disabling service in case project shares other resources that use this api.
  disable_on_destroy = false
}

module "gke" {
  depends_on = [
    module.project_services,
    resource.google_compute_subnetwork.subnetwork,
  ]
  source              = "terraform-google-modules/kubernetes-engine/google//modules/private-cluster"
  version             = "22.1.0"
  project_id          = var.project_id
  name                = var.cluster_name
  region              = var.region
  zones               = var.cluster_zones
  network             = google_compute_network.vpc_network.name
  subnetwork          = "subnetwork-${var.cluster_name}"
  ip_range_pods       = "pods-${var.cluster_name}"
  ip_range_services   = "services-${var.cluster_name}"
  http_load_balancing = true
  # config_connector           = true
  horizontal_pod_autoscaling = false
  network_policy             = false
  enable_private_endpoint    = false
  enable_private_nodes       = false
  master_ipv4_cidr_block     = var.master_cidr
  remove_default_node_pool   = true

  node_pools = [
    {
      name         = "chain-node-pool"
      node_count   = var.num_nodes
      disk_size_gb = var.disk_size_gb
      machine_type = var.machine_type
      disk_type    = var.disk_type
      image_type   = var.image_type
      auto_repair  = true
      auto_upgrade = true
      autoscaling  = false
      preemptible  = false
    },
  ]

  node_pools_oauth_scopes = {
    all = []

    chain-node-pool = [
      "https://www.googleapis.com/auth/cloud-platform",
    ]
  }

  node_pools_tags = {
    chain-node-pool = ["${var.cluster_name}-node"]
  }
}

module "kubernetes-engine_workload-identity" {
  source              = "terraform-google-modules/kubernetes-engine/google//modules/workload-identity"
  version             = "22.1.0"
  gcp_sa_name         = "config-connector"
  cluster_name        = module.gke.name
  name                = "cnrm-controller-manager"
  location            = var.region
  use_existing_k8s_sa = true
  annotate_k8s_sa     = false
  namespace           = "cnrm-system"
  project_id          = var.project_id
  roles               = ["roles/compute.publicIpAdmin"]
}
