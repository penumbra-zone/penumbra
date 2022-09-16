resource "google_compute_firewall" "chain_node" {
  project = var.project_id
  name    = "${var.cluster_name}-firewall"
  network = google_compute_network.vpc_network.id

  allow {
    protocol = "icmp"
  }

  allow {
    protocol = "tcp"
    ports    = ["22", "26656"]
  }

  target_tags   = ["${var.cluster_name}-node"]
  source_ranges = ["0.0.0.0/0"]
}
