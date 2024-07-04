#!/usr/bin/env bash

set -e

current_dir=$(dirname "$0")

# Pulumi

# These stacks can be deleted safely as they only rely on ephemeral containers 
pulumi -C "$current_dir/pulumi/aws" -s test stack rm -yf --preserve-config || true
pulumi -C "$current_dir/pulumi/vault" -s test stack rm -yf --preserve-config || true

# Real resources on cloud, must be deleted
pulumi -C "$current_dir/pulumi/azure" -s test down -yfr

# Kubernetes cluster
kind delete cluster -n novops-auth-test

# Containers
docker compose -f "$current_dir/docker-compose.yml" down -v