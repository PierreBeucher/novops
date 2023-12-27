# `.novops.yml` configuration schema

`novops` uses `.novops.yml` to load secrets. This doc details how this file can be used for various use cases. You can use another config with `novops [load|run] -c PATH`, though this doc will refer to `.novops.yml` for config file.

See [full `.novops.yml` schema](https://novops.dev/config/schema.html) for all available configurations.

## Configuration: Environments, Modules, Inputs and Outputs

`.novops.yml` defines:

- **Environments** for which secrets can be loaded
- Environments define **Inputs** (`files`, `variables`, `aws`...)
- Inputs are **resolved** into **Environment Variables** and **Files** (and other Outputs constructs internally with files and variables)
- Inputs can also use other Inputs, such as an Hashicorp Vault `hvault_kv2` Inputs used by a `variable` Input to resolve a secret into an environment variable (see below for example)

Example: environments `dev` and `prod` with inputs `files`, `variables` and `hvault_kv2`. 

```yaml
environments:

  # Environment name
  dev:    

   # "variables" is a list of "variable" inputs for environment 
   # Loading these inputs will result in envionment variables outputs
   variables:

      # - name: environment variable name
      # - value: variable value, can be a plain string or another input
      - name: MY_APP_HOST
        value: "localhost:8080"

      # here variable value is another Input resolving to a string
      # novops will read the referenced value
      # in this case from Hashicorp Vault server
      - name: MY_APP_PASSWORD
        value:
          hvault_kv2:
            path: crafteo/app/dev
            key: password
    
    # "files" is a list of "file" inputs
    files:
      
      # - content: input resolving to a string. Can be a plain string or another input resolving to a string
      # - variable: a variable name which will point to generated file
      # - dest: Optionally, the final destination where file will be generate. By default Novops create a file in a secure directory.
      #
      # This file input will resolve to two Outputs:
      # - A variable MY_APP_CONFIG=/path/to/secure/location
      # - A file created in a secure location with content "bind_addr: localhost"
      #
      - variable: MY_APP_CONFIG
        content: |
          bind_addr: localhost
    
      # Like variables input, file Input content can use another Input
      # to load value from external source
      - variable: MY_APP_TOKEN
        content: 
          hvault_kv2:
            path: crafteo/app/dev
            key: token
```

## Root `config` keyword

Root `config` is used to specifhy global configurations for Novops and its modules:

```yaml
config:

  # novops default configs
  default:

    # name of environment loaded by default
    environment: dev

  # Hashivault config
  # See Hashivault module doc
  hashivault:
    # ...

  # AWS config
  # See AWS module doc
  aws:
    # ...

  # Other module configs may exists
  # See module docs or full Novops schema for details
  <someModule>:
    # ...
```