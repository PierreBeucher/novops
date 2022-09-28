# Novops

A platform agnostic secret aggregator for CI and development environments.

## Features

- Load secrets and config as file or environment variables in most shells
- Integrate with various secret providers: 
 - Hashicorp Vault
   - Key Value v2
 - AWS:
   - IAM AssumeRole
 - BitWarden
 - _More to come..._
- Manage multiple environments
- Integrate with CI to help reduce drift between CI and local environment

## Getting started

Create a `.novops.yml` file:

```yaml
name: myapp

environments:
  dev:
    variables:
      # Plain string
      - name: APP_URL
        value: "http://127.0.0.1:8080"

      # Retrieve secret from Hashicorp Vault using KV v2 Secret Engine
      - name: APP_PASSWORD
        value:
          hvault_kv2:
            mount: "secret"
            path: "myapp/dev/creds"
            entry: "password"

    files: 
      # Retrieve secret from BitWarden and save it to file
      # File path will be exposed via env var APP_TOKEN
      - name: APP_TOKEN
        content: 
          bitwarden:
            entry: "Dev Secret Token"
            field: notes
```

Load Novops config:
  
```sh
# Load dev config and source env variables in current shell
novops load -e dev -s .myenvs && source .myenvs
```

Though you can source manually, recommended usage is with [`direnv`](https://direnv.net/) installed:

```sh
novops load -e dev -s .envrc
# ...
# direnv: loading ~/myproject/.envrc  
```

Your shell session is now loaded!

```sh
env | grep APP_
# APP_URL=127.0.0.1:8080
# APP_PASSWORD=s3cret
# APP_TOKEN=/run/user/1000/novops/myapp/dev/file_APP_TOKEN
```

## Documentation

- [Advanced usages and examples: Bash, Docker, CI...](./docs/usage.md)
- [Available modules: Hashivault, BitWarden, AWS...](./docs/modules.md)
- [Internal architecture: Inputs, Outputs and resolving](./docs/internals.md)

## Contributing

Feel free to open a Pull Request to contribute code, tests, doc...

## License

TODO add LICENSE.txt