# AWS

- [Authentication \& Configuration](#authentication--configuration)
- [STS Assume Role](#sts-assume-role)
- [Systems Manager (SSM) Parameter Store](#systems-manager-ssm-parameter-store)
- [Secrets Manager](#secrets-manager)
- [S3 file](#s3-file)
- [Advanced examples](#advanced-examples)
  - [Using `credential_process` with TOTP or other user prompt](#using-credential_process-with-totp-or-other-user-prompt)

## Authentication & Configuration

Authenticating with `aws` CLI is enough, Novops will use locally available credentials. Specify your AWS credentials as usual (see [AWS Programmatic access](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys) or [Credentials quickstart](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-quickstart.html#cli-configure-quickstart-creds)):

Credentials are loaded in order of priority:

- Environment variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc.
- Config file `.aws/config` and `.aws/credentials`
- Use IAM Role attached from ECS or EC2 instance

You can also use `config` root element override certains configs (such as AWS endpoint), for example:

```yaml
config:
  
  # Example global AWS config
  # Every field is optional
  aws:

    # Use a custom endpoint
    endpoint: "http://localhost:4566/" 

    # Set AWS region name
    region: eu-central-1 

    # Set identity cache load timeout.
    #
    # By default identity load timeout is 5 seconds
    # but some custom config may require more than 5 seconds to load identity, 
    # eg. when prompting user for TOTP.
    #
    # See Advanced examples below for usage
    identity_cache:
      load_timeout: 120 # timeout in seconds
```

## STS Assume Role

Generate temporary [IAM Role credentials with STS AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html):

Note that `aws` is an `environment` sub-key, not a `files` or `variables` sub-key as it will output multiple variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN` and `AWS_SESSION_EXPIRATION`

```yaml
environments:
  dev:
    # Output variables to assume IAM Role:
    # AWS_ACCESS_KEY_ID
    # AWS_SECRET_ACCESS_KEY
    # AWS_SESSION_TOKEN
    # AWS_SESSION_EXPIRATION (non built-in AWS variable, Linux timestamp in second specifying token expiration date)
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/my_dev_role
        source_profile: novops

        # Optionally define credential duration in seconds. Default to 3600s (1h)
        # duration_seconds: 900
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

## S3 file 

Load [S3 objects](https://aws.amazon.com/s3/) as files or environment variables:

```yaml
environments:
  dev:
    variables:
      - name: S3_OBJECT_AS_VAR
        value:
          aws_s3_object:
            bucket: some-bucket
            key: path/to/object
      
    files: 
      - symlink: my-s3-object.json
        content:
          aws_s3_object:
            bucket: some-bucket
            key: path/to/object.json
```

It's also possible to specify the region in which Bucket is located if different than configured region:

```yml
aws_s3_object:
  bucket: some-bucket
  key: path/to/object
  region: eu-central-1
```
## Advanced examples

### Using `credential_process` with TOTP or other user prompt

In some scenario you might want to use `credential_process` in your config, such as [`aws-vault`], which may ask for TOTP or other user prompts.

For example, using `~/.aws/config` such as:

```toml
[profile crafteo]
credential_process = aws-vault export --format=json crafteo
mfa_serial = arn:aws:iam::0123456789:mfa/my-mfa
```

Credential processor prompts user for TOTP but by default AWS SDK timeout after a few seconds - not enough time to enter data. You can configure identity cache load timeout to give enough time to user. In `.novops.yml`, set config such as:

```yaml
config:
  aws:
    identity_cache:
      load_timeout: 120 # Give user 2 min to enter TOTP
```