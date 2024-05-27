#!/usr/bin/env bash

set -e 

CONTAINER_CLI=${CONTAINER_CLI:-docker}


export KIND_EXPERIMENTAL_PROVIDER=$CONTAINER_CLI
kind delete cluster -n novops-auth-test

$CONTAINER_CLI compose -f tests/docker-compose.yml down -v