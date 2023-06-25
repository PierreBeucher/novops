#/bin/sh

# Test Google client manually since it's only tested with dry-run
# and we had to implement lots of authentication logic

cargo build

# With user authentication
rm -f ~/.config/gcloud/application_default_credentials.json 
gcloud auth application-default login

target/debug/novops load -c tests/.novops.gcloud_secretmanager.yml -e dev

# With Service Account
rm -f ~/.config/gcloud/application_default_credentials.json 
unset GOOGLE_APPLICATION_CREDENTIALS

target/debug/novops load -c tests/manual/.novops-before.yml -s tests/manual/.envrc -e dev && source tests/manual/.envrc 

target/debug/novops load -c tests/.novops.gcloud_secretmanager.yml -e dev 