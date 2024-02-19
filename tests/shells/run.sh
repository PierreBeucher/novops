#!/bin/sh

#
# Manual script to test various shells
#

set -ex

cur_dir="$(dirname $0)"
podman build -f "${cur_dir}/Containerfile.shells" -t novops-shells-test:local

# Test novops with various shells

function test_novops_shell() {
    podman run -it --rm \
        -v "./${cur_dir}/.novops.yml:/app/.novops.yml" \
        -w /app \
        --entrypoint "$1" \
        novops-shells-test:local 
        # -c 'source <(novops load) && env | grep "^MY_APP_HOST=localhost$"'
}

test_novops_shell bash
test_novops_shell dash
test_novops_shell zsh
test_novops_shell fish
test_novops_shell tcsh
test_novops_shell csh