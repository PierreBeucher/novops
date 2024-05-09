# AWS

- [Authentication \& Configuration](#authentication--configuration)
- [STS Assume Role](#sts-assume-role)
- [Systems Manager (SSM) Parameter Store](#systems-manager-ssm-parameter-store)
- [Secrets Manager](#secrets-manager)
- [S3 file](#s3-file)

## Authentication & Configuration

Authenticating with `aws` CLI is enough, Novops will use locally available credentials. Specify your AWS credentials as usual (see [AWS Programmatic access](https://docs.aws.amazon.com/general/latest/gr/aws-sec-cred-types.html#access-keys-and-secret-access-keys) or [Credentials quickstart](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-quickstart.html#cli-configure-quickstart-creds)):

Credentials are loaded in order of priority:

- Environment variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc.
- Config file `.aws/config` and `.aws/credentials`
- Use IAM Role attached from ECS or EC2 instance

You can also use `config` root element override certains configs (such as AWS endpoint), for example:

```yaml
config:
  aws:
    endpoint: "http://localhost:4566/" # Use LocalStack endpoint
    region: eu-central-1 # Set AWS region name
```

## STS Assume Role

Generate temporary [IAM Role credentials with STS AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html):

Note that `aws` is an `environment` sub-key, not a `files` or `variables` sub-key as it will output multiple variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`

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
