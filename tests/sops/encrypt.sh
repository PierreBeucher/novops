#!/bin/sh
sops --encrypt \
    --age "$(cat tests/sops/age1.pub),$(cat tests/sops/age2.pub)" \
    tests/sops/test-dotenv.clear.yml > tests/sops/test-dotenv.encrypted.yml

sops --encrypt \
    --age "$(cat tests/sops/age1.pub),$(cat tests/sops/age2.pub)" \
    tests/sops/test-nested.clear.yml > tests/sops/test-nested.encrypted.yml

SOPS_AGE_KEY_FILE=tests/sops/age1 sops --decrypt --output-type json tests/sops/test-nested.encrypted.yml
echo
echo "---"
SOPS_AGE_KEY_FILE=tests/sops/age2 sops --decrypt --output-type dotenv tests/sops/test-dotenv.encrypted.yml