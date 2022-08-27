module "gcs_backend" {
  source = "../gcs"

  labels            = var.labels
  location          = var.location
  name              = "${var.name_prefix}-tfstate"
  object_versioning = true
  project           = var.project
}
