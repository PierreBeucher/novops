#!/bin/sh
sops --encrypt \
    --age "$(cat tests/setup/sops/age1.pub),$(cat tests/setup/sops/age2.pub)" \
    tests/setup/sops/test-dotenv.clear.yml > tests/setup/sops/test-dotenv.encrypted.yml

sops --encrypt \
    --age "$(cat tests/setup/sops/age1.pub),$(cat tests/setup/sops/age2.pub)" \
    tests/setup/sops/test-nested.clear.yml > tests/setup/sops/test-nested.encrypted.yml

SOPS_AGE_KEY_FILE=tests/setup/sops/age1 sops --decrypt --output-type json tests/setup/sops/test-nested.encrypted.yml
echo
echo "---"
SOPS_AGE_KEY_FILE=tests/setup/sops/age2 sops --decrypt --output-type dotenv tests/setup/sops/test-dotenv.encrypted.yml