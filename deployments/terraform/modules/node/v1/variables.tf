variable "cluster_name" {
  description = "The name of the GKE cluster"
}

variable "project_id" {
  description = "The project ID to host the cluster in"
}

variable "region" {
  description = "The region to host the cluster in"
}

variable "cluster_zones" {
  description = "The zones where the cluster will be deployed"
  type        = list(any)
}

variable "num_nodes" {
  description = "The number of nodes per zone"
  default     = 1
}

variable "dns_managed_zone" {
  description = "Name of existing DNS managed zone"
  default     = ""
}

variable "fqdn" {
  description = "Fully-qualified domain name (DNS A record). Should be subdomain of DNS managed zone"
  default     = ""
}

### The rest of the variables have logical defaults
variable "machine_type" {
  description = "The machine type for the nodes"
  default     = "e2-highmem-4"
}

variable "image_type" {
  description = "The image type for the nodes"
  default     = "COS_CONTAINERD"
}

variable "disk_type" {
  description = "The disk type for the nodes"
  default     = "pd-ssd"
}

variable "disk_size_gb" {
  description = "The disk space (GB) for the nodes"
  default     = 100
}

# Sentry Cluster CIDR Ranges
variable "master_cidr" {
  description = "The CIDR for the cluster"
  default     = "192.168.2.0/28"
}

variable "subnet_cidr" {
  description = "The CIDR for the node subnet"
  default     = "192.168.5.0/24"
}

variable "subnet_pods_cidr" {
  description = "The CIDR for the node pods subnet"
  default     = "10.7.0.0/16"
}

variable "subnet_service_cidr" {
  description = "The CIDR for the node service subnet"
  default     = "10.8.0.0/16"
}
