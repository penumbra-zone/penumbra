variable "services" {
  description = "Services (aka APIs) to enable. E.g. compute.googleapis.com. See all via $ gcloud services list --available"
  type        = set(string)
}

variable "project" {
  description = "The project ID. If not provided, the provider project is used."
  type        = string
  default     = null
}
