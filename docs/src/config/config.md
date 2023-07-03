# `.novops.yml` configuration schema

`novops` will load configuration from current directory `.novops.yml` by default. 

See modules documentation and [full configuration schema](./schema.html) for detailed schema.

## Configuration: Environments, Modules, Inputs and Outputs

Novops config is the source of truth for variables and files to be loaded. Novops config defines:

- **Environments** which can be loaded with `novops load`
- Environments define **Inputs** (`files`, `variables`, `aws`...)
- When loaded, Inputs are **resolved** to provide **Outputs** resulting into files and environment variables
- Inputs can also use other Inputs, such as an Hashicorp Vault `hvault_kv2` Inputs used by a `variable` Input to resolve a secret into an environment variable (see below for example)

Example below defines two environments (`dev` & `prod`), each using multiple inputs: `files`, `variables` and `hvault_kv2`. 

`files` and `variables` modules are the most common Novops modules. Their values/content can be defined using plain strings or other Inputs.

```yaml
name: some-app

environments:

  # Environment name
  dev:    

   # "variables" Input (mind the plural) defines one or more variables for our environment
   # it's a list of "variable" Input 
   # Loading these inputs will result in envionment variables Outputs
   variables:

      # "variable" Input has two arguments: 
      # - name: the name of the result environment variable
      # - value: define the variable value, can be a plain string or another Input
      - name: MY_APP_HOST
        value: "localhost:8080"

      # variable value can be another Input which resolves to a string
      # novops will read the referenced value, in this case from Hashicorp Vault server
      - name: MY_APP_PASSWORD
        value:
          hvault_kv2:
            path: crafteo/app/dev
            key: password
    
    # "files" another input taking a lise of "file" inputs as arguments
    files:
      
      # "file" Input arguments are:
      # - content: Input resolving to a string. Can be a plain string or another input resolving to a string
      # - variable: Optionally, a variable name which will point to generated file
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

## Generic configuration schema

More generally, a novops config looks like:

```yaml
# Required, name of the application
# should be unique, loading multiple Novops config with the same name may cause conflicts
name: crafteo-app

# Each environments represent a specific environments
# with its own variables and files output
# derived from modules config
environments:
  
  environment_A:

    # Each environments defines inputs and their config
    input_1: 
      input_config:
        # Other inputs can be used within inputs
        input_4:

    input_2: 
      some_config: # ...

  environment_B:
    input_3:

# Optional, additional configurations
config:
  # Additional config...
```

`config` is used to specifhy internal Novops config and some module configs:

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