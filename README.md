# Novops

A platform agnostic secret aggregator for CI and development environments.

## Features

- Load secrets and config as file or environment variables in most shells
- Integrate with various secret providers: Hashicorp Vault, AWS, BitWarden...
- Manage multiple environments
- Integrate with CI to help reduce drift between CI and local environment

## Usage

Novops load configs and secrets defined in `.novops.yml` to use them as file and/or environment variables. 

### Simple example

Consider example `.novops.yml`

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

Run Novops with

```sh
# Load dev config and source env variables in current shell
# Creates a symlink .env -> $XDG_RUNTIME_DIR/.../vars to keep secrets safe and allow easy sourcing
novops -e dev -s .env && source .env

echo $APP_URL 
# 127.0.0.1:8080

echo $APP_PASSWORD
# s3cret

cat $APP_TOKEN
# SomeTokenValue

# Files are created securely under XDG Runtime Dir by default
echo $APP_TOKEN
# /run/user/1000/novops/myapp/dev/file_APP_TOKEN

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

TODO

## Secret providers

Quick reference and example of available secret providers. See advanced doc for details.

### Hashicorp Vault

Variables and files:

```yaml
environment:
  dev:
    variables:
    - name: APP_PASSWORD
      value:
        hvault_kv2:
          mount: "secret"
          path: "myapp/dev/creds"
          entry: "password"

    files:
    - name: SECRET_TOKEN
      dest: .token
      content:
        hvault_kv2:
          path: "myapp/dev/creds"
          entry: "token"
```

### AWS

Generate temporary IAM Role credentials:

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

### BitWarden

_Experimental module, requires BitWarden CLI installed locally_

```yaml
environments:
  dev:
    files: 
    - name: ssh-key
      content:
        bitwarden:
          entry: Some SSH Key entry
          field: notes
```

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

### Updating dependencies

We use cargo2nix that can build dependencies separately (it is more granular than nixpkgs' solution) with the inconvenient that now one needs

```sh
nix run github:cargo2nix/cargo2nix
```

### Run test

Integ tests are run within Docker to have a similar environment locally and on CI. 

Run tests locally (via `docker exec` within a Rust container):

```
make test-docker
```

Tests are run on CI for any non-`master` branch. 

### Advanced concepts

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
