variable "labels" {
  type = map(string)
}

variable "name_prefix" {
  description = "A generic name prefix. Must be globally unique."
  type        = string
}

variable "project" {
  description = "Project ID. If absent, uses the provider's project."
  type        = string
  default     = null
}

variable "location" {
  description = "Bucket location/region"
  type        = string
}
