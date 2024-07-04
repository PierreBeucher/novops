#!/usr/bin/env bash

set -e

current_dir=$(dirname "$0")

# Various containers
docker compose -f "$current_dir/docker-compose.yml" up -d

# Kubernetes
if ! kind get clusters | grep -q 'novops-auth-test'; then
  kind create cluster -n novops-auth-test
  docker network connect novops-test novops-auth-test-control-plane
else
  echo "Kind cluster already exists, skipping."
fi

kind get kubeconfig --name novops-auth-test > "$(dirname "$0")/k8s/kubeconfig"
kind get kubeconfig --name novops-auth-test | yq '.clusters[0].cluster["certificate-authority-data"]' -r | base64 -d > "$(dirname "$0")/k8s/ca.pem"

# Various configs via Pulumi
pulumi -C "$current_dir/pulumi/aws" -s test stack select -c
pulumi -C "$current_dir/pulumi/aws" -s test up -yfr

pulumi -C "$current_dir/pulumi/vault" -s test stack select -c
pulumi -C "$current_dir/pulumi/vault" -s test up -yfr

pulumi -C "$current_dir/pulumi/azure" -s test stack select -c
pulumi -C "$current_dir/pulumi/azure" -s test up -yfr

pulumi -C "$current_dir/pulumi/gcp" -s test stack select -c
pulumi -C "$current_dir/pulumi/gcp" -s test up -yfr