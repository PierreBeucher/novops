# Novops

![novops-features](docs/src/assets/novops-features.jpg)

Novops, the universal secret and configuration manager for development, applications and CI:

- **Secret management**: Load secrets safely from any source, including Hashicorp Vault, AWS, GCloud, Azure, SOPS [and more](https://novops.dev/config/index.html)
- **Configuration as code**: Seamlessly manage and set secure files and environment variables for your local development, applications, and CI pipelines.
- **Security**: Load secrets safely in-memory and keep them only for as long as they are needed to avoid mishandling or spreading sensitive data. 
- **Universal**: Designed to be versatile and flexible, meeting a wide range of secret management needs across different platforms and tools.
- **Free and Open Source**: Novops is free and open-source - and will always be.

[📖 Visit website for full documentation and examples](https://novops.dev/intro.html)

![](docs/demo.gif)


---

- [Getting Started](#getting-started)
- [🔐 Security](#-security)
- [Features](#features)
- [Modules](#modules)
  - [Hashicorp Vault](#hashicorp-vault)
  - [AWS: Secrets Manager, Temporary Credentials and Parameter Store](#aws-secrets-manager-temporary-credentials-and-parameter-store)
  - [Google Cloud Secret Manager](#google-cloud-secret-manager)
  - [Azure Key Vault](#azure-key-vault)
  - [SOPS](#sops)
  - [BitWarden](#bitwarden)
- [Examples](#examples)
  - [Run a sub-command with secrets](#run-a-sub-command-with-secrets)
  - [Load secrets into your shell](#load-secrets-into-your-shell)
  - [Manage multiple environments](#manage-multiple-environments)
  - [🐳 Docker \& Podman](#-docker--podman)
  - [More examples: Ansible, Terraform, Pulumi, GitLab, GitHub and more !](#more-examples-ansible-terraform-pulumi-gitlab-github-and-more-)
- [Full Documentation](#full-documentation)
- [Community and Support](#community-and-support)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [Inspiration and similar tools](#inspiration-and-similar-tools)
- [License](#license)
- [Acknowledgment](#acknowledgment)

## Getting Started

Let's deploy an application with **secret password and SSH key from Hashicorp Vault** and **temporary AWS credentials**.

Install Novops static binary ([or use another method](https://novops.dev/install.html)):

```sh
curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
unzip novops.zip
sudo mv novops /usr/local/bin/novops
```

Create `.novops.yml` and commit it safely - it does not contain any secret:

```yaml
environments:

  # Your development environment.
  # Novops supports multiple environments:
  # you can add integ, preprod, prod... 
  # with their own config.
  dev:
    
    # Environment variables for dev environment
    variables:
      
      # Fetch Hashicorp Vault secrets
      - name: DATABASE_PASSWORD
        value:
          hvault_kv2:
            path: app/dev
            key: db_password

      # Set plain string as config
      - name: DATABASE_USER
        value: postgres
    
    # Load files safely in-memory.
    # Environment variable APP_SSH_KEY
    # will point to secure in-memory file 
    files:
      - variable: APP_SSH_KEY 
        content:
          hvault_kv2:
            path: app/dev
            key: ssh_key
    
    # Generate temporary AWS credentials for an IAM Role
    # Provide environment variables:
    # - AWS_ACCESS_KEY_ID
    # - AWS_SECRET_ACCESS_KEY
    # - AWS_SESSION_TOKEN
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/dev_deploy
```

Run commands with secrets and configs and discard them as soon as they're not needed anymore:

```sh
# Run a sub-process with secrets and configs
# Secrets are discard when command finishes
novops run -- make deploy-aws-app

# Run a sub-shell with secrets and configs
# Secrets are discarded on exit
novops run -- sh

# Or source directly into your shell
# Secrets are discarded on exit
source <(novops load)
```

Secrets are available as environment variables and secure in-memory files:

```sh
echo $DATABASE_PASSWORD
# passxxxxxxx

echo $APP_SSH_KEY
# /run/user/1000/novops/... (in a secure tmpfs directory)

env | grep AWS
# AWS_ACCESS_KEY_ID=AKIAXXX
# AWS_SECRET_ACCESS_KEY=xxx
# AWS_SESSION_TOKEN=xxx
```

## 🔐 Security

Secrets are loaded temporarily as environment variables or in a protected `tmpfs` directory and kept only for as long as they are needed.

See [Novops Security Model](https://novops.dev/security.html) for details

## Features

- Securely load secrets in protected in-memory files and environment variables
- Generate temporary credentials and secrets
- Fetch secrets from anywhere: Hashicorp Vault, AWS, Google Cloud, Azure, SOPS [and more](https://novops.dev/config/index.html). Avoid syncing secrets between local tool, CI/CD, and Cloud secret services.
- Feed secrets directly to command or process with `novops run`, easing usage of tools like Terraform, Pulumi, Ansible...
- Manage multiple environments: `dev`, `preprod`, `prod`... and configure them as you need.
- Easy installation with fully static binary or Nix

## Modules

Novops uses _modules_ to load and generate temporary secrets from various platforms and providers. Configure them in `.novops.yml`:

### Hashicorp Vault

Supported Hashicorp Vault Secret Engines:

- Key Value v1/v2
- AWS: generate temporary STS credentials

See [Hashicorp Vault module reference](https://novops.dev/config/hashicorp-vault.html)

```yml
environments:
  dev:
    
    variables:
      # KV2 Secret Engine
      - name: APP_PASSWORD
        value:
          hvault_kv2:
            path: "myapp/dev/creds"
            key: "password"
      
      # KV1 Secret Engine
      - name: APP_TOKEN
        value:
          hvault_kv1:
            mount: kv1
            path: "otherapp/dev/creds"
            key: "token"

    # AWS temporary creds with Vault
    hashivault:
      aws:
        name: dev_role
        role_arn: arn:aws:iam::111122223333:role/dev_role
```

### AWS: Secrets Manager, Temporary Credentials and Parameter Store

Multiple AWS services are supported:

- Secrets Manager
- STS Assume Role for temporary IAM Role credentials
- SSM Parameter Store

See [AWS module reference](https://novops.dev/config/aws.html)

```yml
environments:
  dev:
    
    # Temporary IAM Role credentials. Output variables
    # AWS_ACCESS_KEY_ID
    # AWS_SECRET_ACCESS_KEY
    # AWS_SESSION_TOKEN
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/my_dev_role
        source_profile: novops

    variables:

      # Secrets Manager
      - name: APP_PASSWORD
        value:
          aws_secret:
            id: dev-app-password

      # SSM Parameter Store
      - name: APP_TOKEN
        value:
          aws_ssm_parameter:
            name: dev-app-token
```

### Google Cloud Secret Manager

Load secrets from Google Cloud Secret Manager. See [Google Cloud module reference](https://novops.dev/config/google-cloud.html)

```yml
environments:
  dev:
    variables:

      # Secret Manager
      - name: APP_PASSWORD
        value:
          gcloud_secret:
            name: projects/my-project/secrets/AppPasswordDev/versions/latest
```

### Azure Key Vault

Load secrets from Azure Key Vault. See [Azure Key Vault module reference](https://novops.dev/config/microsoft-azure.html)


```yml
environments:
  dev:
    variables:

      # Key Vault
      - name: APP_PASSWORD
        value:
          azure_keyvault_secret:
            vault: vault-dev
            name: app-password
```

### SOPS

Load secrets from SOPS encrypted files. See [SOPS module reference](https://novops.dev/config/sops.html)

```yml
environments: 
  dev:
    variables:

      # SOPS nested key
      - name: APP_PASSWORD
        value:
          sops:
            file: sops/dev/encrypted.yml
            extract: '["app"]["password"]'
    
    # Entire SOPS file as dotenv
    sops_dotenv:
      - file: sops/dev/encrypted-dotenv.yml
      - file: sops/dev/another-encrypted-dotenv.yml
```

### BitWarden

Load secrets from BitWarden. See [BitWarden module reference](https://novops.dev/config/bitwarden.html)

```yml
environments:
  dev:
    variables: 
      
      # BitWarden secret item
      - name: APP_PASSWORD
        value:
          bitwarden:
            entry: "App Password - Dev"
            field: login.password
```

## Examples

Example `.novops.yml` for an application with database and two environments, `dev` & `prod`:

```yaml
environments:
  # DB login, password and host for dev
  # Pass password as env var
  dev:
    variables:
      - name: DB_PASSWORD
        value:
          hvault_kv2:
            path: app/dev
            key: db_password
      
      - name: DB_USER
        value: postgres

      - name: DB_HOST
        value: db.dev.example.com

  # DB login, password and host for prod
  # Pass password as secure file
  prod:

    files:
      # Password will be saved in a secure file
      # Environment variable DB_PASSWORD_FILE will point to it
      # Such as DB_PASSWORD_FILE=/run/...
      - variable: DB_PASSWORD_FILE
        content:
          hvault_kv2:
            path: app/prod
            key: db_password

    variables:      
      - name: DB_USER
        value: postgres

      - name: DB_HOST
        value: db.prod.example.com
```

With Novops you can:

### Run a sub-command with secrets

Run your deployment script with Database secrets 

```sh
novops run -- ./deploy-app.sh

# You can run any command
#
# novops run -- ansible-playbook deploy.yml
# novops run -- terraform plan && terraform apply
# novops run -- pulumi up -yrf
```

Or directly run a sub-shell with secrets loaded:

```sh
novops run --sh
```

### Load secrets into your shell

`novops load` loads secrets directly as _dotenv_ exportable file. 

```sh
source <(novops load)
```

_Note: for security, Novops won't output anything directly in a terminal to prevent secret from being exposed directly._

### Manage multiple environments

By default Novops will prompt for environment

```sh
novops run -- ./deploy-app.sh
# Select environment: dev, prod
```

You can specify environment directly with `-e ENV`

```sh
novops run -e dev -- ./deploy-app.sh
```

### 🐳 Docker & Podman

Load environment variables directly into containers:

```sh
docker run -it --env-file <(novops load -f dotenv -e dev) alpine sh
podman run -it --env-file <(novops load -f dotenv -e dev) alpine sh
```

### More examples: Ansible, Terraform, Pulumi, GitLab, GitHub and more !

- [Shell](https://novops.dev/examples/shell.html)
- [Docker & Podman](https://novops.dev/examples/docker.html)
- [Nix](https://novops.dev/examples/nix.html)
- CI / CD
  - [GitLab CI](https://novops.dev/examples/cicd/gitlab-ci.html)
  - [GitHub Action](https://novops.dev/examples/cicd/github-action.html)
  - [Jenkins](https://novops.dev/examples/cicd/jenkins.html)
- Infra as Code
  - [Ansible](https://novops.dev/examples/iac/ansible.html)
  - [Terraform](https://novops.dev/examples/iac/terraform.html)
  - [Pulumi](https://novops.dev/examples/iac/pulumi.html)


## Full Documentation

[Checkout full documentation](https://novops.dev/intro.html)

## Community and Support

A question? A problem? A bug to report?

- [Join Discord channel](https://discord.gg/R3jzTcBEsQ)
- [File an issue](https://github.com/PierreBeucher/novops/issues)

## Roadmap

The following modules are expected to be implemented:

- More [Hashicorp Vault Secret Engines](https://developer.hashicorp.com/vault/docs/secrets) among which
  - Kubernetes
  - SSH
  - Azure Role
- [1Password](https://1password.com/)
- [Consul](https://developer.hashicorp.com/vault/docs/configuration/storage/consul)
- [Infisical](https://infisical.com)


Feel free to create an issue and [contribute a PR](https://novops.dev/contributing/index.html) !

## Contributing

We welcome contributions: bug reports/fixes, modules, proposals... :) To get started you can check [Novops internal architecture](https://novops.dev/advanced/architecture.html) and:

- [New module implementation guide](https://novops.dev/contributing/add-module.html)
- [Development guide](https://novops.dev/contributing/development.html)

## Inspiration and similar tools

- https://github.com/getsops/sops
- https://github.com/dotenv-org/dotenv-vault
- https://github.com/tellerops/teller
- https://github.com/sorah/envchain
- https://github.com/Infisical/infisical
- https://github.com/99designs/aws-vault
- https://github.com/channable/vaultenv

## License 

[GNU Lesser General Public License](LICENSE)

## Acknowledgment

Novops was initially developed and used at [Novadiscovery](https://www.novadiscovery.com/) who graciously transferred code ownership. Thanks Nova's team for your help in designing and developing Novops. 