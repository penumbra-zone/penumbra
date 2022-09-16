locals {
  name_prefix = join("-", [var.chain_name, var.network_environment])
}
