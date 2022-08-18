# GCS Terraform State

Creates a GCS bucket to house terraform state for a remote backend for any deployment.

**If you have a validator or chain deployment** use [the specific chain backend](./)

## Bucket Location

For important or production networks, use "US" or similar as the bucket location. This creates a multi-region bucket for
redundancy. E.g. All saas deployments use multi-region.

Otherwise, use a specific region like "us-central1" if redundancy is not critical.

## Bootstrapping

Bootstrapping a new terraform root module (i.e. the directory where you `terraform apply`) requires using a local backend first. Then,
migrating the local backend to the remote backend.

1. In the `terraform {}` block, ensure no backend is set.
2. Add this module and `terraform apply` to create the bucket.
3. Add the following to the terraform block:
```hcl
backend "gcs" {
  bucket = "<name of bucket just created>"
}
```
4. Run `terraform init -migrate-state`
5. Likely enable public access prevention detailed below.

## Public Access Prevention

Public access prevention protects Cloud Storage buckets and objects from being accidentally exposed to the public.

Currently, terraforming this feature is not possible per bucket. Per [a Dec 2021 PR](https://github.com/GoogleCloudPlatform/magic-modules/pull/5519),
it appears the google-beta provider *should* support it.

However, testing with google-beta v4.28.0, the variable is invalid.

```shell
❯ terraform plan
╷
│ Error: Unsupported argument
│
│   on ../../../../terraform/modules/gcp/gcs/main.tf line 16, in resource "google_storage_bucket" "bucket":
│   16:   public_access_prevention = "true"
│
│ An argument named "public_access_prevention" is not expected here.
```

The [4.28.0 docs](https://registry.terraform.io/providers/hashicorp/google-beta/4.28.0/docs/resources/storage_bucket) do not
expose any such variable either.

It's unclear when or if this feature will be included in a google provider.

As a workaround, [manually activate public access prevention](https://cloud.google.com/storage/docs/using-public-access-prevention).

## TODOs:
* Public access prevention
* Lifecycle rules for old object versions
