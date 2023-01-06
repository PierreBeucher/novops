# Modules reference

- [Hashicorp Vault](#hashicorp-vault)
  - [Configuration](#configuration)
  - [Key Value Version 2](#key-value-v2)
- [AWS](#aws)
  - [Configuration](#configuration-1)
  - [SSM Parameter Store](#systems-manager-ssm-parameter-store)
  - [Secrets Manager](#secrets-manager)
  - [IAM AssumeRole](#sts-assume-role)
- [Google Cloud](#google-cloud)
  - [Configuration](#configuration-2)
  - [Secret Manager](#secret-manager)
- [BitWarden](#bitwarden) - _experimental and untested, use with care_

Wanna add a module? See [contribution guide](../CONTRIBUTING.md) !

## Hashicorp Vault

### Configuration

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

[Key Value Version 2](https://www.vaultproject.io/docs/secrets/kv/kv-v2) with variables and files:

```yaml
environment:
  dev:
    variables:
    - name: APP_PASSWORD
      value:
        hvault_kv2:
          mount: "secret"
          path: "myapp/dev/creds"
          entry: "password"

    files:
    - name: SECRET_TOKEN
      dest: .token
      content:
        hvault_kv2:
          path: "myapp/dev/creds"
          entry: "token"
```

## AWS

### Configuration

Specify your AWS credentials as usual (see [AWS Programmatic access](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys) or [Credentials quickstart](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-quickstart.html#cli-configure-quickstart-creds)):

- Environment variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc.
- Config file `.aws/config` and `.aws/credentials`
- Use IAM Role attached from ECS or EC2 instance

### STS Assume Role

Generate temporary [IAM Role credentials with AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html):

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

### Configuration

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