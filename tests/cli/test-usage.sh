#!/bin/sh
#
# Test CLI to ensure it's not broken
# Cargo tests ensure most features are OK but not CLI usage itself
# Here we tests all main commands and options are working and using properly internal code
#

set -ex

novops_test_cmd="cargo run --"
novops_test_dir="/tmp/novops-cli-test"
rm -rf $novops_test_dir
mkdir -p $novops_test_dir

# load 
$novops_test_cmd load -c tests/.novops.plain-strings.yml -s $novops_test_dir/test.envrc -e dev 
cat $novops_test_dir/test.envrc | grep "^export MY_APP_HOST='localhost'$"
$novops_test_cmd load -c tests/.novops.plain-strings.yml -s $novops_test_dir/test.dotenv -e dev -f dotenv
cat $novops_test_dir/test.dotenv | grep "^MY_APP_HOST='localhost'$"

# run
$novops_test_cmd run -c tests/.novops.plain-strings.yml -e dev -- sh -c "env | grep DOG_PATH"

# env vars 
export NOVOPS_CONFIG=tests/.novops.plain-strings.yml
export NOVOPS_ENVIRONMENT=dev
$novops_test_cmd run -- sh -c "env | grep DOG_PATH"

# list
$novops_test_cmd list environments -c tests/.novops.multi-env.yml | grep preprod
$novops_test_cmd list environments -c tests/.novops.multi-env.yml -o json | grep '"preprod"'
$novops_test_cmd list outputs -e dev -c tests/.novops.multi-env.yml | grep MY_APP_HOST
$novops_test_cmd list outputs -e dev -c tests/.novops.multi-env.yml -o json | grep '"MY_APP_HOST"'
