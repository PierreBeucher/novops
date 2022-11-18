# Contributing

Wanna contribute? Awesome ! We'll happily review issues and PRs.

## Adding a module

A few [modules](docs/modules.md) already exists for popular Secret Manager and system such as Hashicorp Vault. 

Some popular secret manager we intent to integrate:
- [AWS Secret Manager](https://aws.amazon.com/secrets-manager/)
- [AWS System Manager Paramater Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
- [Google Cloud Secret Manager](https://cloud.google.com/secret-manager)
- [Azure Key Vault](https://azure.microsoft.com/en-us/products/key-vault/)

Generator modules we could implement:

- [GitLab CI](https://docs.gitlab.com/ee/ci/) - to generate a dummy set of [Gitlab CI predefined variable](https://docs.gitlab.com/ee/ci/variables/predefined_variables.html), helping in reproduce CI behaviours locally

You want to add a module? Great, we welcome contributions ! Before writing a module, you may want to understand [Novops architecture](docs/architecture.md).

### Guide: how to add a module?

- [ ] Create a file named after your module under `src/modules/`
- [ ] Define Input (and Output if it's not a Rust existing type/trait) for your modules (make sure to `derive` from everything shown in example below)
- [ ] Implement `core::ResolveTo<YourOutputTypeOrTrait>` for your module
- [ ] If your module resolves to a `String` (probably the case if you're loading/generating string-like values), add it to `src/core` enum `pub enum StringResolvableInput` so that it can be used directly with any Input accepting String outputs (such as `files` and `variables` modules)
- [ ] If you want your module to be available directly within an environment, add id to `src/core` enum `NovopsEnvironmentInput`
- [ ] If your module takes specific config, such as server adress, credentials, etc. add a related struct in `src/core` struct `NovopsConfig`


#### Example: a simple Loader module

Example implementation of a module loading values from an external source storing key / JSON values:

```rust
// src/modules/my_module.rs
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct MyModuleInput {
    pub key: String
}

#[async_trait]
impl ResolveTo<String> for MyModuleInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        // retrieve the value for key from our system
        let my_value = key_value_system_client.get(&self.key).await?;

        return Ok(my_value)
    }
}
```

`MyModule` will generate `String` Output, we can add it to `StringResolvableInput`:

```rust
// src/core.rs
#[derive( JsonSchema /* ... */ )]
pub enum StringResolvableInput {
    // ...
    MyModuleInput(my_module::MyModuleInput),
}
```

As both Inputs derive from `JsonSchema`, our module becomes available within Novops config such as:

```yaml
environments:
  dev:
    variables:
      - name: MY_SECRET_VALUE
        value:
          # Here comes our module
          my_module:
            key: key_to_read
```

How did we integrate it directly? Because `variables` module takes as input... another input resolving to a String:

```rust
// src/modules/variables.rs
pub struct VariableInput {
    name: String,
    // As we integrated our module to StringResolvableInput, it can now be uwed within variables module
    // and any module allowing string resolvable inputs
    value: StringResolvableInput
}
```

Now let's add a config component to our module: the server host/port from which to load values (with sane defaults)

```rust
// src/modules/my_module.rs
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]  
pub struct MyModuleConfig {
  host: String,
  port: Option<String>
}

impl Default for MyModuleConfig {
  fn default() -> HashivaultConfig {
    HashivaultConfig{
      host: None,
      port: 443
    }
  }
}
```

And add it to `NovopsConfig`:

```rust
// src/core.rs
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsConfig {
    // ...
    pub my_module: Option<my_module::MyModuleConfig>
}
```

We can then add to Novops config:

```yaml
config:
  my_module:
    host: awesomesecretmanager.crafteo.io
    port: 8080
```

#### Example: a simple Generator module

Example implementation of a random-value generator module:

```rust
// src/modules/random_string.rs
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct RandomStringInput {
    pub length: i32
}

impl Default for RandomStringInput {
  fn default() -> RandomStringInput {
    RandomStringInput{
      lenght: 42
    }
  }
}

#[async_trait]
impl ResolveTo<String> for RandomStringInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        // retrieve the value for key from our system
        let random_str = generate_random_string(&self.length)

        return Ok(random_str)
    }
}
```

`RandomStringInput` will generate `String` Output, we can add it to `StringResolvableInput`:

```rust
// src/core.rs
#[derive( JsonSchema /* ... */ )]
pub enum StringResolvableInput {
    // ...
    RandomStringInput(random_string::RandomStringInput),
}
```

As both Inputs derive from `JsonSchema`, our module becomes available within Novops config such as:

```yaml
environments:
  dev:
    variables:
      - name: MY_SECRET_VALUE
        value:
          # Here comes our module
          # No need for parameter as length is set as default
          random_string:
```

### Testing

Tests are implemented under `tests/test_<module_name>.rs`. 

Most tests are integration tests using Docker containers for external system and a dedicated `.novops.<MODULE>.yml` file with related config.

If you depends on external component (such as Hashivault instance), use Docker container to spin-up a container and configure it accordingly. See `tests/docker-compose.yml`