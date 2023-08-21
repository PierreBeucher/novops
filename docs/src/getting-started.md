# Getting started

- [Getting started](#getting-started)
  - [Install](#install)
  - [Simple example](#simple-example)
  - [Load secrets from secret providers: Hashicorp Vault, AWS credentials, etc.](#load-secrets-from-secret-providers-hashicorp-vault-aws-credentials-etc)
  - [Novops Security model](#novops-security-model)
  - [Next steps](#next-steps)

## Install

```sh
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

See [installation](install.md) for more installation methods.

## Simple example

Novops uses `novops.yml` configuration file to define the set of **environment variables** and **files** you need **per environment**.

Create a `.novops.yml` file such as:

```yaml
name: myapp

environments:

  dev:    
    variables:
      - name: HELLO_WORLD
        value: "Hello World!"
    files: 
      - variable: HELLO_FILE
        content: "Hello file!"
  
  prod:    
    variables:
      - name: HELLO_WORLD
        value: "Hello World from Prod!"
    files: 
      - variable: HELLO_FILE
        content: "Hello file from Prod!"
  
```

Use `novops load` to load environment variables in your shell:

```sh
source <(novops load)
# source <(novops load -e dev) to avoid prompting for environment
```

You can now access variables:

```sh
echo $HELLO_WORLD   # "Hello World!"
```

Files are saved under a secure directory only accessible by current user:

```sh
echo $HELLO_FILE    # /run/user/1000/novops/myapp/dev/file_...
cat $HELLO_FILE     # Hello file!
```

You can define an alias for `novops load`:

```sh
alias nload='source<(novops load)'
```

## Load secrets from secret providers: Hashicorp Vault, AWS credentials, etc.

Novops primary goal is to _load secrets securely_. You can:
- Load secrets from various platforms such as [Hashicorp Vault](config/hashicorp-vault.md)
- Generate temporary credentials from [AWS Secret Manager](config/aws.md) and other platforms
- See [Modules usage](./config/README.md) for available modules.

**Novops is safe**: by default nothing is written to disks and secrets are kept temporary in-memory and discarded after use. 

Example `.novops.yml` using Hashicorp Vault secrets and generating AWS credentials:

```yaml
name: aws-and-hashicorp-vault-example

environments:
  dev:    
    variables:
      
      # Fetch Hashicorp Vault secrets
      - name: DATABASE_PASSWORD
        value:
          hvault_kv2:
            path: crafteo/app/dev
            key: db_password
  
    files: 

      # Load a secret into a file
      # TOKEN_FILE variable will point to file path
      - variable: TOKEN_FILE
        content:
          hvault_kv2:
            path: crafteo/app/dev
            key: secure_token
    
    # Generate temporary AWS credentials for IAM Role
    # Output variables AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY and AWS_SESSION_TOKEN
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/dev_deploy
```

Load config:

```sh
source <(novops load -e dev)

env | grep AWS
# AWS_ACCESS_KEY_ID=AKIA...
# AWS_SECRET_ACCESS_KEY=xxx
# AWS_SESSION_TOKEN=xxx

echo $DATABASE_PASSWORD
# secr3t!

echo $TOKEN_FILE
# /run/user/1000/novops/myapp/dev/file_...

cat $TOKEN_FILE
# super_t0k3n
```

## Novops Security model

Novops tries to be secure:

- Secrets are kept in-memory: directly sourced in shell via standard output
- Secrets won't be written to disk unless specifically asked by user
- If secrets are to be written to disk, Novops try to use a secure directory ([`XDG_RUNTIME_DIR`, see Security](advanced/security.md)) so they are protected and not persisted. 
- Secrets are loaded temporarily: being in memory, they'll disappear as soon as process finished using them. No secret is persisted. When possible, Novops generates temporary secrets (e.g. credentials)

## Next steps

- See [Novops configuration details and modules](config/config.md) to learn more about Modules, Inputs and Outputs
- [How Novops loads files securely](advanced/security.md)
- Checkout [examples and use cases](./examples/README.md)