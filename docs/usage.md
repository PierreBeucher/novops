# Usage and examples

Novops main purpose is to manage secrets and config in multi-environment context without having to rely on local `.env`, CI variables config, etc. Following examples show usage with various tools. 

- [Usage and examples](#usage-and-examples)
  - [Run Novops from...](#run-novops-from)
    - [Local shell](#local-shell)
    - [Docker](#docker)
    - [GitLab CI](#gitlab-ci)
  - [Leverage Novops to configure and run...](#leverage-novops-to-configure-and-run)
    - [Ansible](#ansible)
    - [Terraform](#terraform)
    - [Pulumi](#pulumi)

## Run Novops from...

### Local shell

For local usage, [`direnv`](https://direnv.net/) usage is [recommended for security reasons](./novops-direnv.md). 

```sh
# Generate env file at .envrc
# direnv will automatically load it
novops -e dev -s .envrc
```

It's also possible to source manually when using `direnv` is not practical or necessary (such as in a Docker container or CI job):

```sh
# Source manually
# But variables won't be unset unless done manually
novops -e dev -s .env && source .env
```

### Nix

Setup a development shell with [Nix Flakes](https://nixos.wiki/wiki/Flakes):

```nix
{
  description = "Example Flake using Novops";

  inputs = {
    novops.url = "github:novadiscovery/novops";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, novops, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let 
        pkgs = nixpkgs.legacyPackages.${system};
        novopsPackage = novops.packages.${system}.novops;
      in {
        devShells = {
          default = pkgs.mkShell {
            packages = [ 
              novopsPackage
            ];
            shellHook = ''
              novops load -s .envrc
              source .envrc
            '';
          };
        };
      }
    );    
}

```

Run with:

```sh
nix develop
```

### Docker

_NOTE: Novops Docker image is not yet published but [Dockerfile is available at root](../Dockerfile)_

Include in your Dockerfile with:

```Dockerfile
FROM novops

FROM alpine
COPY --from=novops /usr/local/bin/novops /usr/local/bin/novops
```

Then run a container with your `.novops.yml` such as:

```sh
# Mount novops config in container and run novops
docker run -it -v $PWD/.novops.yml:/novops-config.yml IMAGE
$ novops -c /novops-config.yml && source /tmp/.novops/vars

# Alternatively mount entire directory
docker run -it -v $PWD:/myapp -w /myapp IMAGE
$ novops && source /tmp/.novops/vars
```

### GitLab CI

Load Novops at beginning of job:

```yaml
job-using-novops:
  # ...
  script:
    - novops load -e dev -s .env && source .env
    # Variables will be available for subsequent commands
    - make test 
```

## Leverage Novops to configure and run...

Use Novops with built-in variables from your favorite tool for easy switch between environments

### Ansible

Use [Ansible built-in variables](https://docs.ansible.com/ansible/latest/reference_appendices/config.html#environment-variables):

```yaml
environments:
  dev:
    variables:
      # Comma separated list of Ansible inventory sources
      - name: ANSIBLE_INVENTORY
        value: inventories/dev
      # Add more as needed
      # - name: ANSIBLE_*
      #   value: ...

    files:
      # Connect to hosts using a certificate or key file
      - variable: ANSIBLE_PRIVATE_KEY_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: ssh_key

      # Vault password file
      - variable: ANSIBLE_VAULT_PASSWORD_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: inventory_password
  
  prod:
    variables:
      - name: ANSIBLE_INVENTORY
        value: inventories/prod
      # ...
```

### Terraform

Use [Terraform built-in environment variables](https://developer.hashicorp.com/terraform/cli/config/environment-variables):

```yaml
environments:
  dev:
    variables:
      # Set workspace instead of running 
      # terraform workspace select (workspace]
      - name: TF_WORKSPACE
        value: dev_workspace

      # Use TF_VAR_* to set declared variables
      # See https://developer.hashicorp.com/terraform/language/values/variables#environment-variables
      # and https://developer.hashicorp.com/terraform/cli/config/environment-variables#tf_var_name
      - name: TF_VAR_region
        value: eu-central-1
      - name: TF_VAR_some_list
        value: '[1,2,3]'
      # - name: TF_VAR_[varname]
      #   value: ...

    files:
      # Terraform CLI configuration file
      - variable: TF_CLI_CONFIG_FILE
        content: |
          ...
      
  prod:
    variables:
      - name: TF_WORKSPACE
        value: prod_workspace
  # ...
```

### Pulumi

Use [Pulumi built-in environment variables](https://www.pulumi.com/docs/reference/cli/environment-variables/):

```yaml
environments:
  dev:
    variables:
      
      # Authenticate into the Pulumi Service backend 
      # and bypass access token prompt when running pulumi login
      - name: PULUMI_ACCESS_TOKEN
        value:
          hvault_kv2:
            path: myapp/dev
            key: pulumi_access_token

      # Specify backend instead of the default backend
      # See https://www.pulumi.com/docs/intro/concepts/state/#using-a-self-managed-backend
      - name: PULUMI_BACKEND_URL
        value: "s3://crafteo-pulumi-backend"

      # Passphrase for configuration
      - name: PULUMI_CONFIG_PASSPHRASE
        value:
          hvault_kv2:
            path: myapp/dev
            key: pulumi_passphrase

    files:  
      # Alternative to PULUMI_CONFIG_PASSPHRASE
      - variable: PULUMI_CONFIG_PASSPHRASE_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: pulumi_passphrase
      
  prod:
    variables:
      - name: PULUMI_ACCESS_TOKEN
        value:
          hvault_kv2:
            path: myapp/prod
            key: pulumi_access_token
    # ...
```


