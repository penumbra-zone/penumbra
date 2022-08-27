resource "google_storage_bucket" "bucket" {
  name          = var.name
  project       = var.project
  location      = var.location
  force_destroy = var.force_destroy

  // Generally, using uniform bucket-level access is recommended, because it
  // unifies and simplifies how you grant access to your Cloud Storage resources.
  // https://cloud.google.com/storage/docs/uniform-bucket-level-access
  uniform_bucket_level_access = true

  versioning {
    enabled = var.object_versioning
  }

  labels = var.labels
}
