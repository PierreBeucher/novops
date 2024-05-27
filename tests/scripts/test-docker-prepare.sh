#!/usr/bin/env bash

set -e 

CONTAINER_CLI=${CONTAINER_CLI:-docker}

export KIND_EXPERIMENTAL_PROVIDER=$CONTAINER_CLI

$CONTAINER_CLI compose -f tests/docker-compose.yml up -d

if ! kind get clusters | grep -q 'novops-auth-test'; then
  kind create cluster -n novops-auth-test
  $CONTAINER_CLI network connect tests_default novops-auth-test-control-plane
else
  echo "Kind cluster already exists"
fi