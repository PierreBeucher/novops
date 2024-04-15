#!/usr/bin/env sh

set -x

#
# Manual script to test Novops installation script with various shells
#

# bash dash zsh fish ksh tcsh csh curl unzip


function test_novops_install() {
    local image=$1
    shift  # Shift the first argument to get rid of the image name, leaving only commands

    # Run the docker container with the specified image and commands
    if docker run -it --rm -v "$PWD:/local" -w /local "$image" /bin/sh -c "$*"; then
        echo "OK: $image"
    else
      echo "NOT OK: $image"
      exit 1
    fi
}

test_novops_install alpine:3.19.1 "apk update && apk add curl unzip && ./install.sh && novops --version"
test_novops_install debian:12.5-slim "apt update && apt install curl unzip -y && ./install.sh && novops --version"
test_novops_install ubuntu:22.04 "apt update && apt install curl unzip -y && ./install.sh && novops --version"

