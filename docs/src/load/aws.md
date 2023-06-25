# AWS

- [AWS](#aws)
  - [Authentication & Configuration](#authentication--configuration)
  - [STS Assume Role](#sts-assume-role)
  - [Systems Manager (SSM) Parameter Store](#systems-manager-ssm-parameter-store)
  - [Secrets Manager](#secrets-manager)

## Authentication & Configuration

See [AWS Examples](../examples/aws-role.md) for authentication methods you can use CI or for local development environments.

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

## STS Assume Role

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

## Systems Manager (SSM) Parameter Store

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

## Secrets Manager

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
