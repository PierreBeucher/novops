#!/usr/bin/env bash

#
# Test Macos install
#

# Remove previous test container if any
test_container_name=novops-macos-install-test
docker ps -a | grep $test_container_name && docker rm -f $test_container_name || true

docker run -d \
    --name $test_container_name \
    --device /dev/kvm \
    -p 50922:10022 \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    -e "DISPLAY=${DISPLAY:-:0.0}" \
    -e GENERATE_UNIQUE=true \
    -e EXTRA="-display none" \
    -e OSX_COMMANDS="echo 'Started MacOS' && sleep 9999" \
    sickcodes/docker-osx:auto@sha256:b1c060e60a8a8272a143e3cc9d05b8a405b124d1fe3b8d4f8473ad392405799b

timeout 300 bash -c 'until sshpass -p "alpine" ssh -o ConnectTimeout=5 user@localhost -p 50922 "echo connected"; do sleep 5; done'

mkdir -p macosmnt
sshpass -p "alpine" sshfs user@localhost:/testdir -p 50922 $PWD/macosmnt

sshpass -p "alpine" ssh -o ConnectTimeout=5 user@localhost -p 50922 "echo connected"

