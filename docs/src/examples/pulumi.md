# Pulumi

A typical Pulumi workflow requires developers and CI to provide per environment:

- Pulumi stack name
- Pulumi stack passphrase
- Pulumi authentication token or backend config

Instead of duplicating configs and spreading secrets on CI and your local environment, you can use Novops and [Pulumi built-in environment variables](https://www.pulumi.com/docs/reference/cli/environment-variables/).

Your workflow will then look like:

```sh
novops load -s .envrc && source .envrc
# Select environment: dev, prod (default: dev)

# No need to specify stack name, passphrase or backend
# They've all been loaded as environment variables and files
pulumi up
```

- [Pulumi](#pulumi)
    - [Stack passwords](#stack-passwords)
    - [Pulumi Cloud Backend authentication](#pulumi-cloud-backend-authentication)
    - [Custom Pulumi backend](#custom-pulumi-backend)

### Stack passwords

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
      
  prod:
    variables:
      - name: PULUMI_ACCESS_TOKEN
        value:
          hvault_kv2:
            path: myapp/prod
            key: pulumi_access_token
```

### Pulumi Cloud Backend authentication

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

### Custom Pulumi backend

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

---

Your workflow will now be the same on CI or for local development, and you'll be able to switch environments seamlessly !