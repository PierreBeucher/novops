# Files and Variables

`files` and `variables` are primay way to configure Novops
- Each element in `variables` will generate a single environment variable loaded from `value`
- Each element in `files` will generate a [secured temporary file](../security.md) loaded from `content`

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

      # Any input resolving to a string value can be used with variable
      # See below for available modules
      - name: APP_SECRET
        value:
          <module_name>:
            <some_config>: foo
            <another_config>: bar
    
    # List of files to load for dev
    # Each files must define either dest, variable or both
    files:

      # A symlink will be created at ./symlink-pointing-to-file, pointing to
      # a file in secure Novops working directory which will have content "foo"
      - symlink: ./symlink-pointing-to-file
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

### File `dest` deprecation

`dest` is deprecated as it may result in file being generated in insecure directory and/or persisted on disk (as file is written directly at provided path, outside of [secure Novops working directory](../security.md)). Use `symlink` instead.

```yaml
    # [...]
    files:

      # Prefer symlink
      - symlink: ./my-secret-token
        content:
          hvault_kv2:
            path: "myapp/dev/creds"
            key: "token"
      
      # DON'T DO THIS
      - dest: ./my-secret-token # not secure
        content:
          hvault_kv2:
            path: "myapp/dev/creds"
            key: "token"
```