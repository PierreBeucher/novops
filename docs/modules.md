# Modules reference

- [Modules reference](#modules-reference)
  - [Files and Variables](#files-and-variables)
  - [Hashicorp Vault](#hashicorp-vault)
    - [Authentication & Configuration](#authentication--configuration)
    - [Key Value v2](#key-value-v2)
    - [Key Value v1](#key-value-v1)
  - [AWS](#aws)
    - [Authentication & Configuration](#authentication--configuration-1)
    - [STS Assume Role](#sts-assume-role)
    - [Systems Manager (SSM) Parameter Store](#systems-manager-ssm-parameter-store)
    - [Secrets Manager](#secrets-manager)
  - [Google Cloud](#google-cloud)
    - [Authentication](#authentication)
    - [Secret Manager](#secret-manager)
  - [Microsoft Azure](#microsoft-azure)
    - [Authentication](#authentication-1)
    - [Key Vault](#key-vault)
  - [BitWarden](#bitwarden)

Wanna add a module? See [contribution guide](../CONTRIBUTING.md) !

## Files and Variables

`files` and `variables` lists are primay way to configure Novops. 
- `variables` element generate a single environment variable from `value`
- `files` elements generate a file using defined `content`

When loaded an environment variable is generated which can be [automatically sourced with `direnv`](./novops-direnv.md) or sourced manually with `source`

```yaml
environments:
  dev:
    
    # Variables to load
    # name and value are required keys
    # value can take a plain string or a module
    variables:
      # Plain string
      - name: APP_URL
        value: "http://127.0.0.1:8080"

      # Use Hashicorp Vault KV2 module to set variable value
      - name: APP_PASSWORD
        value:
          hvault_kv2:
            path: crafteo/app/dev
            key: password

      # Any module loading string value can be used with variable
      # See below for available modules
      - name: APP_SECRET
        value:
          module_name:
            some_config: foo
            aother_config: bar
    
    # List of files to load for dev
    # Each files must define either dest, variable or both
    files:

      # File will be created at /tmp/myfile with content "foo"
      - dest: /tmp/myfile
        content: foo

      # Fille will be generated in a secure folder
      # APP_TOKEN variable will point to file
      # Such as APP_TOKEN=/run/user/1000/novops/.../file_VAR_NAME
      - variable: APP_TOKEN
        content:
          hvault_kv2:
            path: "myapp/dev/creds"
            key: "token"
```

## Hashicorp Vault

### Authentication & Configuration

Specify your Vault instance in config:

```yaml
environments:
# ...

config:
  hashivault:
    address: http://localhost:8200
    # token: xxx # token can be set in config for testing
```

Or set [Vault built-in environment variables](https://developer.hashicorp.com/vault/docs/commands#environment-variables) prior to running `novops`:

```sh
export VAULT_TOKEN=xxx
export VAULT_ADDR=https://vault.mycompany.org:8200
```

### Key Value v2

Hashicorp Vault [Key Value Version 2](https://www.vaultproject.io/docs/secrets/kv/kv-v2) with variables and files:

```yaml
environment:
  dev:
    variables:
    - name: APP_PASSWORD
      value:
        hvault_kv2:
          mount: "secret"
          path: "myapp/dev/creds"
          key: "password"

    files:
    - name: SECRET_TOKEN
      dest: .token
      content:
        hvault_kv2:
          path: "myapp/dev/creds"
          key: "token"
```

### Key Value v1

Hashicorp Vault [Key Value Version 1](https://www.vaultproject.io/docs/secrets/kv/kv-v1) with variables and files:

```yaml
environments:
  dev:
    variables:
      - name: APP_PASSWORD
        value:
          hvault_kv1:
            path: app/dev
            key: password
            mount: kv1 # Override secret engine mount ('secret' by default)
    
    files:
      - variable: APP_TOKEN
        content:
          hvault_kv1:
            path: app/dev
            key: token
```

## AWS

### Authentication & Configuration

Specify your AWS credentials as usual (see [AWS Programmatic access](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys) or [Credentials quickstart](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-quickstart.html#cli-configure-quickstart-creds)):

- Environment variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc.
- Config file `.aws/config` and `.aws/credentials`
- Use IAM Role attached from ECS or EC2 instance

You can also set global AWS configuration to override certains configs (such as AWS endpoint), for example:

```yaml
environments:
  # ...

aws:
  endpoint: "http://localhost:4566/" # Use LocalStack endpoint
```

### STS Assume Role

Generate temporary [IAM Role credentials with AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html):

Note that `aws` is an `environment` key rather than a `files` or `variables`. That's because it will output multiple variables.

```yaml
environments:
  dev:
    # Output variables to assume IAM Role:
    # AWS_ACCESS_KEY_ID
    # AWS_SECRET_ACCESS_KEY
    # AWS_SESSION_TOKEN
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/my_dev_role
        source_profile: novops
```

### Systems Manager (SSM) Parameter Store

Retrieve key/values from [AWS SSM Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html) as env variables or files:

```yaml
environments:
  dev:
    variables:
    - name: MY_SSM_PARAM_STORE_VAR
      value:
        aws_ssm_parameter:
          name: some-param
          # with_decryption: true/false
    
    files:
    - name: MY_SSM_PARAM_STORE_FILE
      content:
        aws_ssm_parameter:
          name: some-var-in-file
```

### Secrets Manager

Retrieve secrets from [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/) as env var or files:

```yaml
environments:
  dev:
    variables:
    - name: MY_SECRETSMANAGER_VAR
      value:
        aws_secret:
          id: my-string-secret

    files:
    - name: MY_SECRETSMANAGER_FILE
      content:
        aws_secret:
          id: my-binary-secret
```

## Google Cloud

### Authentication

Provide credentials using [Application Default Credentials](https://cloud.google.com/docs/authentication/application-default-credentials):

- Set `GOOGLE_APPLICATION_CREDENTIALS` to a credential JSON file
- Setup creds using `gcloud` CLI
- Attached service account

### Secret Manager

Retrieve secrets from [GCloud Secret Manager](https://cloud.google.com/secret-manager/docs) as env var or files:

```yaml
environments:
  dev:
    variables:
    - name: SECRETMANAGER_VAR_STRING
      value:
        gcloud_secret:
          name: projects/my-project/secrets/SomeSecret/versions/latest
          # validate_crc32c: true
  
    files:
    - name: SECRETMANAGER_VAR_FILE
      content:
        gcloud_secret:
          name: projects/my-project/secrets/SomeSecret/versions/latest
```

## Microsoft Azure

### Authentication

Novops use [`azure_identity`](https://crates.io/crates/azure_identity) `DefaultAzureCredential`. Provide credentials via:

- [Environment variables](https://docs.rs/azure_identity/0.9.0/azure_identity/struct.EnvironmentCredential.html)
- [Azure CLI](https://docs.rs/azure_identity/0.9.0/azure_identity/struct.AzureCliCredential.html)
- [Managed Identity](https://docs.rs/azure_identity/0.9.0/azure_identity/struct.ImdsManagedIdentityCredential.html)

### Key Vault

Retrieve secrets from [Key Vaults](https://azure.microsoft.com/en-us/products/key-vault/) as files or variables:

```yaml
environments:
  dev:
    variables:
    - name: AZ_KEYVAULT_SECRET_VAR
      value:
        azure_keyvault_secret:
          vault: my-vault
          name: some-secret
  
    files:
    - name: AZ_KEYVAULT_SECRET_FILE
      content:
        azure_keyvault_secret:
          vault: my-vault
          name: some-secret
          version: 1234118a41364a9e8a086e76c43629e4
```

## BitWarden

_Experimental module, requires BitWarden CLI installed locally_

```yaml
environments:
  dev:
    files: 
    - name: ssh-key
      content:
        bitwarden:
          entry: Some SSH Key entry
          field: notes
```