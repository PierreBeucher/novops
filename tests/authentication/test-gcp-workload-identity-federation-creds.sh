#!/usr/bin/env bash

# Test novops Google Workload Identity Federation authentication using Azure credentials
# - Run authentication steps in container to avoid side-effect with locally configured accounts
# - Generate an Azure token with Google credential file which Novops can read
# - Run Novops and check authentication happens properly

set -e

AZ_STACK_OUTPUTS=$(pulumi -C tests/setup/pulumi/azure/ -s test stack output --json --show-secrets)
AZ_TENANT=$(echo $AZ_STACK_OUTPUTS | jq -r .tenantId)
AZ_USERNAME=$(echo $AZ_STACK_OUTPUTS | jq -r .servicePrincipalClientId)
AZ_PASSWORD=$(echo $AZ_STACK_OUTPUTS | jq -r .password)
AZ_IDENTIFIER_URI=$(echo $AZ_STACK_OUTPUTS | jq -r .identifierUri)

echo "Login to Azure using Tenant ID '$AZ_TENANT' Service Principal '$AZ_USERNAME' to get token for Identifier URI '$AZ_IDENTIFIER_URI'"

mkdir -p ./tmp

# Authenticate with Azure in a container (to avoid side effect with local Azure config)
# And save token to file via bind mount
podman run -u 0 --rm -it -v $PWD:/novops --entrypoint bash bitnami/azure-cli:2.67.0 -c \
    "az login --service-principal --username $AZ_USERNAME --password $AZ_PASSWORD --tenant $AZ_TENANT && \
    az account get-access-token --resource $AZ_IDENTIFIER_URI --query accessToken --output tsv > /novops/tmp/az-token.txt"

# Generate Google credential JSON file for our Azure token
GCP_STACK_OUTPUTS=$(pulumi -C tests/setup/pulumi/gcp/ -s test stack output --json)
WIF_NAME=$(echo $GCP_STACK_OUTPUTS | jq -r .workloadIdentityPoolProviderName)
GCP_PROJECT_NAME=$(echo $GCP_STACK_OUTPUTS | jq -r .projectName)

echo "Using Workload Identity Federation provider: $WIF_NAME"

# Project name is asked but actually not used
gcloud iam workload-identity-pools create-cred-config \
    --project "dummy" \
    $WIF_NAME \
    --credential-source-file=$PWD/tmp/az-token.txt \
    --output-file=$PWD/tmp/gcp-auth.json

export GOOGLE_APPLICATION_CREDENTIALS="$PWD/tmp/gcp-auth.json"

echo "Google auth file ready in $GOOGLE_APPLICATION_CREDENTIALS"

RUST_LOG=novops=debug cargo run -- load -c tests/.novops.gcloud_secretmanager.yml --skip-tty-check
