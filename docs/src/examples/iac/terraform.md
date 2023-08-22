# Terraform

Leverage [Terraform built-in environment variables](https://developer.hashicorp.com/terraform/cli/config/environment-variables) to setup your environments, e.g:

- `TF_WORKSPACE` - Set workspace per environment
- `TF_VAR_name` - Set Terraform variable `name` via environment variables
- `TF_CLI_ARGS` and `TF_CLI_ARGS_name` - Specify additional CLI arguments

Your workflow will then look like:

```sh
source <(novops load)

# No need to set workspace or custom variables 
# They've all been loaded as environment variables and files
terraform plan
terraform apply
```

Use a `.novops.yml` such as:

```yaml
environments:
  dev:
    variables:
      # Set workspace instead of running 'terraform workspace select (workspace]' manually
      - name: TF_WORKSPACE
        value: dev_workspace

      # Set environment config file and other environment specific argument using TF_CLI_ARGS_*
      - name: TF_CLI_ARGS_var-file
        value: dev.tfvars
      
      - name: TF_CLI_ARGS_input
        value: false
        
      # - name: TF_CLI_ARGS_xxx
      #   value: foo

      # Use TF_VAR_* to set declared variables
      # Such as loading a secret variable
      - name: TF_VAR_database_password
        value:
          hvault_kv2:
            path: myapp/dev
            key: db_password

      # - name: TF_VAR_[varname]
      #   value: ...

    files:
      # Terraform CLI configuration file for dev environment
      - variable: TF_CLI_CONFIG_FILE
        content: |
          ...
      
  # Production environment
  prod:
    variables:
      - name: TF_WORKSPACE
        value: prod_workspace
      - name: TF_CLI_ARGS_var-file
        value: prod.tfvars
      - name: TF_VAR_database_password
        value:
          hvault_kv2:
            path: myapp/prod
            key: db_password
    files:
      - variable: TF_CLI_CONFIG_FILE
        content: |
          ...
```
