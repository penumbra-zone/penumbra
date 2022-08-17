terraform {
  required_version = ">= 1.0"

  backend "gcs" {
    bucket = "penumbra-testnet-tfstate"
  }

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 4.12"
    }
  }
}

provider "google" {
  project = "penumbra-sl-testnet"
  region  = "us-central1"
}
