data "google_client_config" "default" {}

module "project_services" {
  source   = "../../gcp/project/services"
  services = ["compute.googleapis.com", "container.googleapis.com"]
}
