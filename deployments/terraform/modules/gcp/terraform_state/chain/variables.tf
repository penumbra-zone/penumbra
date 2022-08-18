variable "labels" {
  type = map(string)
}

variable "chain_name" {
  description = "Short, canonical name for the chain. E.g. osmosis, juno, cosmoshub"
  type        = string
}

variable "network_environment" {
  description = "One of mainnet, testnet, devnet"
  type        = string
  validation {
    condition     = contains(["mainnet", "testnet", "devnet"], var.network_environment)
    error_message = "Must be mainnet, testnet, or devnet."
  }
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
