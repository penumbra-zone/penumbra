resource "google_compute_network" "vpc_network" {
  depends_on = [
    module.project_services
  ]
  project                 = var.project_id
  name                    = "vpc-${var.cluster_name}"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "subnetwork" {
  project       = var.project_id
  name          = "subnetwork-${var.cluster_name}"
  ip_cidr_range = var.subnet_cidr
  region        = var.region
  network       = google_compute_network.vpc_network.id
  secondary_ip_range {
    range_name    = "pods-${var.cluster_name}"
    ip_cidr_range = var.subnet_pods_cidr
  }
  secondary_ip_range {
    range_name    = "services-${var.cluster_name}"
    ip_cidr_range = var.subnet_service_cidr
  }
}
