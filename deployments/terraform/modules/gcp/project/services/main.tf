resource "google_project_service" "enable_service" {
  for_each = var.services
  service  = each.value
  project  = var.project

  // Intentionally avoid disabling in case project has other resources that depend on this service.
  disable_on_destroy = false
}
