variable "labels" {
  type = map(string)
}

variable "name" {
  description = "Bucket name. Must be globally unique."
  type        = string
}

variable "project" {
  description = "Project ID. If absent, uses the provider's project."
  type        = string
  default     = null
}

variable "location" {
  description = "Location/region"
  type        = string
}

variable "force_destroy" {
  description = "Delete bucket even though it has objects in it?"
  type        = bool
  default     = false
}

variable "object_versioning" {
  description = "Enable object versioning."
  type        = bool
  default     = false
}
