# Novops

A platform agnostic secret aggregator for CI and development environments.

## Usage

Novops will load configs and secrets defined in config (`.novops.yml` by default) as environment variable and files.

## Simple example

Consider example `.novops.yml`

```yaml
name: myapp

environments:
  dev:
    variables:
      - name: APP_HOST
        value: "127.0.0.1:8080"

      - name: APP_PASSWORD
        value:
          bitwarden:
            entry: "App password dev"
            field: login.password

    files: 
      - dest: ".token"
        content: 
          bitwarden:
            entry: "Dev Secret Token"
            field: notes
      
      - name: dog
        variabe: DOG_PATH
        content: "woof"
```

Run commands

```sh
# Load dev config and source env variables in current shell
# Creates a symlink .env -> $XDG_RUNTIME_DIR/.../vars to keep secrets safe and allow easy sourcing
novops -e dev -s .env && source .env

echo $APP_HOST 
# 127.0.0.1:8080

echo $APP_PASSWORD
# <secret>

cat $DOG_PATH
# woof

# Files are also created securely under XDG RUntime Dir 
echo $DOG_PATH
# /run/user/1000/novops/example-app/local/file_dog

```

Will result in:

- File `.token` created with content from BitWarden entry `Staging Secret Token`
- File `/run/user/1000/novops/myapp/dev/files_foo` created with content `bar`
- Variables exported:
  ```sh
  # as stated in config
  APP_HOST="127.0.0.1:8080"

  # Every file in config comes with a variable pointing to its path
  NOVOPS_FILE_DEV_NPM_TOKEN=/dir/running/novops/.token
  NOVOPS_FILE_DEV_DOG=/run/user/1000/novops/myapp/dev/files_dog
  ```

### Bash / Shell

```
novops -e dev -w ".novops"; source ".novops/vars"
```

### Docker

Include in your Dockerfile with:

```Dockerfile
FROM novops

FROM alpine
COPY --from=novops /usr/local/bin/novops /usr/local/bin/novops
```

Then use with bash/shell in container:

```
docker run -it -v $PWD/.novops.yml:/novops-config.yml
$ novops -c /novops-config.yml -w /tmp/.novops; source /tmp/.novops/vars
```

### Nix

## Updating dependencies

We use cargo2nix that can build dependencies separately (it is more granular
than nixpkgs' solution) with the inconvenient that now one needs

```sh
nix run github:cargo2nix/cargo2nix
```

## AVailable secret providers

- Bitwarden
- _Soon: Hashicorp Vault_

## Development

### Build

Plain binary:

```sh
carbo build 
```

Docker image (using BuildKit):

```sh
docker buildx build .
```

### Run test

TODO

### Concepts

Novops relies around the following concepts:

#### Inputs, resolving & Outputs 

Inputs are configurations and references (provided in `.novops.yml`) representing values to load.

Outputs are concrete objects based on Inputs. Currently only 2 types of Output exists:
- Files
- Environment variables (or a file which can be sourced into shell)

When running, Novops will:

1. Read config file and parse all Inputs
2. Resolve all Inputs to their concrete values (i.e. their Outputs equivalent)
3. Export all Outputs to system (i.e. write file and provide environment variable values)

Example: YAML Input for BitWarden entry 

```yaml
bitwarden:
  entry: Some Entry
  field: login.password
```

Would _resolve_ into a string value corresponding to the password in Bitwarden entry _Some Entry_

Depending on usage, it can output as a File or an Environment Variable, such as:

```yaml
# Output as variable
variables:
  - name: MY_PASSWORD
    value:
      bitwarden:
        entry: Some Entry
        field: login.password

# Output as file
files:
  - dest: /tmp/mypass
    content:
      bitwarden:
        entry: Some Entry
        field: login.password
```

Resolving mechanism is based on `ResolveTo` trait implemented for each Input. An example implementation for `BitwardenItemInput` into a `String` can be:

```rust
impl ResolveTo<String> for BitwardenItemInput {
    async fn resolve(&self, _: &NovopsContext) -> String {
        // resolve our input to a concrete value
        // contact Bitwarden API with client to retrieve item
        // and extract field login.password 
        bw_client = Bitwarden::client::new()
        return bw_client.get_item(self.entry).get_field(self.field)
    }
}
```

See [`src/novops.rs`](src/novops.rs) for details.

#### Modules

Novops implement Modules for various Input and Output types:

**Variables**: exportable environment variables
- Input: _Any Input resolving to a String_
- Output: Variables

**Files**: create secure files on system
- Input: A file definition (including _any Input resolving to a String_ as file content)
- Output: Files (and Variables pointing to Files)

**Bitwarden**: retrieve Bitwarden items and objects
- Input: Bitwarden item or object reference
- Output: String

**AWS**: assume IAM Roles and export `AWS_*` variables (`AWS_ACCESS_KEY_ID`, etc.)
- Input type: Assume Role definition
- Output type: Variables

**Hashicorp Vault**: (_NOT IMPLEMENTED YET_) retrieve Hashicorp Vault secrets
- Input type: Hashicorp Vault secret definition
- Output type: String
