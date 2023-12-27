# SOPS (Secrets OPerationS)

Load SOPS encryped values as files or environment variables.

- [Requirements](#requirements)
- [Load a single value](#load-a-single-value)
- [Load entire file as dotenv](#load-entire-file-as-dotenv)
- [Pass additional flags to SOPS](#pass-additional-flags-to-sops)

Example below consider example files:

```yml
# clear text for path/to/encrypted.yml
nested:
  data:
    nestedKey: nestedValue
```

```yml
# clear text for path/to/encrypted-dotenv.yml
APP_TOKEN: secret
APP_PASSWORD: xxx
```

## Requirements

You need `sops` CLI available locally as Novops will wrap calls to `sops --decrypt` under the hood. 

All SOPS decryptions methods are supported as would be done using CLI command `sops --decrypt`. See [SOPS official doc](https://github.com/getsops/sops) for details. 

## Load a single value

Extract a single value as environment variable or file.

```yml
environments: 
  dev:
    variables:

      # Load a single SOPS nested key as environment variable
      # Equivalent of `sops --decrypt --extract '["nested"]["data"]["nestedKey"]' path/to/encrypted.yml`
      - name: SOPS_VALUE
        value:
          sops:
            file: path/to/encrypted.yml
            extract: '["nested"]["data"]["nestedKey"]'

      # YOU PROBABLY DON'T WANT THAT
      # Without 'extract', SOPS entire file content is set as environment variable
      # Instead, use environment top-level key sops
      # - name: SOPS_ENTIRE_FILE
      #   value:
      #     sops:
      #       file: path/to/encrypted.yml

    files:

      # Load SOPS decrypted content into secure temporary file
      # SOPS_DECRYPTED would point to decrypted file content such as SOPS_DECRYPTED=/run/...
      # Equivalent of `sops --decrypt path/to/encrypted.yml`
      - variable: SOPS_DECRYPTED
        content:
          sops:
            file: path/to/encrypted.yml
```

## Load entire file as dotenv

Load entire SOPS file(s) as `dotenv` environment variables:

```yml
environments: 
  dev:
    # This is a direct sub-key of environment name
    # Not a sub-key of files or variables
    sops_dotenv:

      # Use plain file content as dotenv values
      - file: path/to/encrypted-dotenv.yml

      # Use a nested key as dotenv values 
      - file: path/to/encrypted.yml
        extract: '["nested"]["data"]'

```

_Note: SOPS won't be able to decrypt complex or nested values (this is a SOPS limitation). Only dotenv-compatible files or file parts with extract can be used this way._

## Pass additional flags to SOPS 

By default Novops will load SOPS secrets using `sops` CLI such as `sops --decrypt [FILE]`. It's possible to pass additional flags with `additional_flags`.

**Warning:** it may break Novops loading mechanism if output is not as expected by Novops. Only use this if an equivalent feature is not already provided by a module option. Feel free to [create an issue](https://github.com/PierreBeucher/novops/issues) or [contribute](https://github.com/PierreBeucher/novops/blob/main/CONTRIBUTING.md) to add missing feature !

Example: enable SOPS verbose output

```yaml
environments: 
  dev:
    variables:
      - name: SOPS_VALUE_WITH_ADDITIONAL_FLAGS
        value:
          sops:
            file: path/to/encrypted.yml
            extract: '["nested"]["data"]["nestedKey"]'
            additional_flags: [ "--verbose" ]
```

Novops `debug` logging will show `sops` stderr (stout is not shown to avoid secret leak):

```sh 
RUST_LOG=novops=debug novops load
```