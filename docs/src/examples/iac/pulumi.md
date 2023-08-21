# Pulumi

Leverage [Pulumi built-in environment variables](https://www.pulumi.com/docs/reference/cli/environment-variables/) to setup your environments, e.g:

- `PULUMI_CONFIG_PASSPHRASE` and `PULUMI_CONFIG_PASSPHRASE_FILE` - specify passphrase to decrypt secrets
- `PULUMI_ACCESS_TOKEN` - Secret token used to authenticate with Pulumi backend
- `PULUMI_BACKEND_URL` - Specify Pulumi backend URL, useful with self-managed backends changing with environments

Your workflow will look like:

```sh
# Access token, config passphrase and backend URL
# are set by environment variables
novops run -- pulumi up -s $PULUMI_STACK -ryf
```

- [Stack passwords](#stack-passwords)
- [Stack name per environment](#stack-name-per-environment)
- [Pulumi Cloud Backend authentication](#pulumi-cloud-backend-authentication)
- [Custom Pulumi backend](#custom-pulumi-backend)

## Stack passwords

Pulumi protect stack with passphrase. Use `PULUMI_CONFIG_PASSPHRASE` or `PULUMI_CONFIG_PASSPHRASE_FILE` variable to provide passphrase.

```yaml
environments:
  dev:
    # Use a variable
    variables:
      - name: PULUMI_CONFIG_PASSPHRASE
        value:
          hvault_kv2:
            path: myapp/dev
            key: pulumi_passphrase

    # Or a file
    files:  
      - variable: PULUMI_CONFIG_PASSPHRASE_FILE
        content: 
          hvault_kv2:
            path: myapp/dev
            key: pulumi_passphrase
```

## Stack name per environment

Pulumi does not provide a built-in `PULUMI_STACK` variable but you can still use it with `pulumi -s $PULUMI_STACK`. See [#13550](https://github.com/pulumi/pulumi/issues/13550)

```yaml
environments:
  dev:
    variables:
      - name: PULUMI_STACK
        value: dev
  prod:
    variables:
      - name: PULUMI_STACK
        value: prod
```

## Pulumi Cloud Backend authentication

Pulumi `PULUMI_ACCESS_TOKEN` built-in variable can be used to authenticate with Pulumi Cloud Backend.

```yaml
environments:
  dev:
    variables:  
      - name: PULUMI_ACCESS_TOKEN
        value:
          hvault_kv2:
            path: myapp/dev
            key: pulumi_access_token

  prod:
    variables:  
      - name: PULUMI_ACCESS_TOKEN
        value:
          hvault_kv2:
            path: myapp/prod
            key: pulumi_access_token
```

## Custom Pulumi backend

Pulumi can be used with [self-managed backends](https://www.pulumi.com/docs/concepts/state/#using-a-self-managed-backend) (AWS S3, Azure Blob Storage, Google Cloud storage, Local Filesystem). 

Use `PULUMI_BACKEND_URL` to switch backend between environments and provide properly scoped auhentication. Example for AWS S3 Backend:

```yaml
environments:
  dev:
    variables: 
      - name: PULUMI_BACKEND_URL
        value: "s3://dev-pulumi-backend"
    
    # Optionally, impersonate a dedicated IAM Role for your environment
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/app_dev_deployment

  prod:
    variables: 
      - name: PULUMI_BACKEND_URL
        value: "s3://prod-pulumi-backend"
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/app_prod_deployment
```