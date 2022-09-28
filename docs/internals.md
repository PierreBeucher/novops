# Novops internal code architecture and concepts

Novops relies around the following concepts:

## Inputs, resolving & Outputs 

_Inputs_ are configurations and references (provided in config, i.e. `.novops.yml`) representing values to load.

_Outputs_ are concrete objects based on Inputs. Currently only 2 types of Output exists:
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
