# Getting started

## Install

First install Novops:

```sh
curl -L "https://github.com/novadiscovery/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

See [installation](install.md) for more installation methods.

## A simple example

Novops uses `novops.yml` configuration file to define the set of **environment variables** and **files** you need **per environment**.

Create a `.novops.yml` file such as:

```yaml
name: my-awesome-app

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

Run command:

```sh
novops load -s .envrc
# Select environment: dev, prod
# (type environment name and press enter)
```

A symlink `.envrc` is created pointing to secure file. It contains exportable variables generated from `.novops.yml`:

```sh
cat .envrc
# export HELLO_WORLD='Hello World!'
# export HELLO_FILE='/run/user/1000/novops/my-awesome-app/dev/file_cac...z'
```

Source this file:

```
source .envrc
```

Our environment is now loaded into our current shell !

```sh
echo $HELLO_WORLD
# Hello World!

cat $HELLO_FILE
# Hello file!
```

You can define an alias for `novops load`:

```sh
alias nload='novops load -s .envrc && source .envrc'
```

# A more complete example

Plain string values aren't very useful, are they? The power of Novops lies behind its modules: you can get values from external secret or configuration providers like [Hashicorp Vault](config/hashicorp-vault.md) or [AWS Secret Manager](config/aws.md).

For example:

```yaml
name: another-awesome-app

environments:

  dev:    

    variables:
      - name: MY_APP_HOST
        value: "localhost:8080"

      # Use an external source to retrieve secret as variable
      # Example with Hashicorp Vault KV2 module
      - name: MY_APP_PASSWORD
        value:
          hvault_kv2:
            path: crafteo/app/dev
            key: password
  
    # All files are loaded in a secure directory
    files: 

      # Load a secret into a file from external source (example with Hashicorp Vault KV2 module)
      # File path is exposed via MY_APP_TOKEN environment variable 
      - variable: MY_APP_TOKEN_PATH
        content:
          hvault_kv2:
            path: crafteo/app/dev
            key: token
      
      # A configuration for this environment
      - dest: build/my-app-config.yml
        content: |
          # Dev environment: disable TLS
          client-tls-verify: false
  
  # Another environment with its own secrets and configs
  prod:    
   variables:
      - name: MY_APP_HOST
        value: "app.crafteo.io"
      - name: MY_APP_PASSWORD
        value:
          hvault_kv2:
            path: crafteo/app/prod
            key: password
    files: 
      - variable: MY_APP_TOKEN_PATH
        content:
          hvault_kv2:
            path: crafteo/app/prod
            key: token
      - dest: build/my-app-config.yml
        content: |
          # Prod config
          client-tls-verify: true
          client-two-factor-enforce: true
```

Load config:

```sh
novops load -s .envrc && source .envrc
# ...

cat .envrc
# export MY_APP_HOST"localhost:8080"
# export MY_APP_PASSWORD='s3cret!'
# export MY_APP_TOKEN_PATH=/run/user/1000/novops/another-awesome-app/dev/file_xxx'
```

Note that files are loaded under `/run/user/1000`, the `XDG_RUNTIME_DIR` directory specified by [XDG Base Directory Specs](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) provided by most Linux distributions. This directory can only be accessed by current user and is cleaned-up on user logout, system shutdown or after a certain amount of time (6h per spec). It provides a basic but powerful layer of security: secrets are protected and will be cleaned-up quickly. See [How Novops loads files securely](advanced/security.md) for details. 

## Getting further

- See [Novops configuration details](config/config.md) to learn more about Modules, Inputs and Outputs
- [How Novops loads files securely](advanced/security.md)
- Checkout examples and use cases