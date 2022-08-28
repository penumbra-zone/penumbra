#!/bin/bash

CLUSTER_NAME=testnet
REGION=us-central1

gcloud container clusters update $CLUSTER_NAME --region=$REGION --update-addons=ConfigConnector=ENABLED,HttpLoadBalancing=ENABLED

SERVICE_ACCOUNT=config-connector

gcloud iam service-accounts create $SERVICE_ACCOUNT

PROJECT_ID=penumbra-sl-testnet

gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:$SERVICE_ACCOUNT@$PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/compute.publicIpAdmin"

gcloud iam service-accounts add-iam-policy-binding \
    $SERVICE_ACCOUNT@$PROJECT_ID.iam.gserviceaccount.com \
    --member="serviceAccount:$PROJECT_ID.svc.id.goog[cnrm-system/cnrm-controller-manager]" \
    --role="roles/iam.workloadIdentityUser"

NAMESPACE=configcontroller
kubectl create namespace $NAMESPACE

kubectl annotate namespace \
  $NAMESPACE cnrm.cloud.google.com/project-id=$PROJECT_ID