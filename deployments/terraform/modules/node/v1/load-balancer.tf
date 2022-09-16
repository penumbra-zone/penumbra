resource "google_compute_forwarding_rule" "default" {
  project               = var.project_id
  name                  = "load-balancer-${var.cluster_name}"
  target                = google_compute_target_pool.default.self_link
  load_balancing_scheme = "EXTERNAL"
  port_range            = "1317-26660"
  region                = var.region
  ip_protocol           = "TCP"
}

resource "google_compute_target_pool" "default" {
  project          = var.project_id
  name             = "load-balancer-${var.cluster_name}"
  region           = var.region
  session_affinity = "NONE"

  health_checks = [google_compute_http_health_check.default.0.self_link]
}

resource "google_compute_http_health_check" "default" {
  count   = 1
  project = var.project_id
  name    = "load-balancer-${var.cluster_name}-hc"

  check_interval_sec  = null
  healthy_threshold   = null
  timeout_sec         = null
  unhealthy_threshold = null

  port         = 31251
  request_path = "/"
  host         = null
}

resource "google_compute_firewall" "default-lb-fw" {
  project = var.project_id
  name    = "load-balancer-${var.cluster_name}-vm-service"
  network = google_compute_network.vpc_network.name

  allow {
    protocol = "tcp"
    ports    = [1317, 9090, 9091, 26657]
  }

  source_ranges = ["0.0.0.0/0"]

  target_tags = ["${var.cluster_name}-node"]

  target_service_accounts = null
}

resource "google_compute_firewall" "default-hc-fw" {
  count   = 1
  project = var.project_id
  name    = "load-balancer-${var.cluster_name}-hc"
  network = google_compute_network.vpc_network.name

  allow {
    protocol = "tcp"
    ports    = [31251]
  }

  source_ranges = ["35.191.0.0/16", "209.85.152.0/22", "209.85.204.0/22"]

  target_tags = ["${var.cluster_name}-node"]

  target_service_accounts = null
}
