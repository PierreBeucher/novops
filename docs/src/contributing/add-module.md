# Guide: implementing a module

Thanks for your interest in contributing ! Before implementing a module you may want to understand [Novops architecture](../advanced/architecture.md). 

- [Overview](#overview)
- [1. Input and Output](#1-input-and-output)
- [2. Implement loading logic with `core::ResolveTo<E>`](#2-implement-loading-logic-with-coreresolvetoe)
- [3. Integrate module to `core`](#3-integrate-module-to-core)
- [4. (Optional) Global configuration](#4-optional-global-configuration)
- [Testing](#testing)

## Overview

A few [modules](https://github.com/PierreBeucher/novops/tree/main/src/modules) already exists from which you can take inspiration. This guide uses Hashicorp Vault Key Value v2 `hvault_kv2` as example. 

You can follow this checklist (I follow and update this checklist myself when adding new modules):

1. [ ] Define Input(s) and Output(s)
2. [ ] Implement loading logic with `core::ResolveTo<E>`
3. [ ] Integrate module to `core`
4. [ ] Optionally, define global config for module

## 1. Input and Output

Create `src/modules/hashivault/kv2.rs` and add module entry in `src/modules/hashivault/mod.rs`. Then define Input and Output `struct` for modules. Each `struct` needs a few `derive` as shown below.

A main `struct` must contain a single field matching YAML key to be used as variable value or file content:

```rust
/// src/modules/hashivault/kv2.rs 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultKeyValueV2Input {
  hvault_kv2: HashiVaultKeyValueV2
}
```

Main `struct` references a more complex `struct` with our module's usage interface. Again, each field matches YAML keys provided to end user:

```rust
/// src/modules/hashivault/kv2.rs 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultKeyValueV2 {
    /// KV v2 mount point
    /// 
    /// default to "secret/"
    pub mount: Option<String>,

    /// Path to secret
    pub path: String,

    /// Secret key to retrieve
    pub key: String
}
```

## 2. Implement loading logic with `core::ResolveTo<E>`

`ResolveTo<E>` trait defines how our module is supposed to load secrets. In other words, how are Inputs supposed to be converted to Outputs. Most of the time, `ResolveTo<String>` is used as we want to use it as environment variables or files content. 

```rust
/// src/modules/hashivault/kv2.rs 
#[async_trait]
impl ResolveTo<String> for HashiVaultKeyValueV2Input {
  async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
    
    let client = get_client(ctx)?;
    let result = client.kv2_read(
        &self.hvault_kv2.mount, 
        &self.hvault_kv2.path, 
        &self.hvault_kv2.key
    ).await?;

    Ok(result)
  }
}
```

Note arguments `self` and `ctx`:

- `self` is used to pass module argument from YAMl Config. For instance:
  ```yaml
  hvault_kv2:
    path: app/dev
    key: db_pass
  ```
  Is used as:
  ```rust
  &self.hvault_kv2.path
  &self.hvault_kv2.key
  ```
- `ctx` is global Novops context, including current environment and entire `.novops.yml` config file. We used it above to create Hashicorp Vault client from global `config` element (see below). 

## 3. Integrate module to `core`

[`src/core.rs`](https://github.com/PierreBeucher/novops/blob/main/src/core.rs) defines main Novops `struct` and the config file hierarchy, e.g:

- `NovopsConfigFile` - Config file format with `environments: NovopsEnvironments` field
- `NovopsEnvironments` and `NovopsEnvironmentInput` with `variables: Vec<VariableInput>` field
- `VariableInput` with `value: StringResolvableInput` field
- `StringResolvableInput` is an enum with all Inputs resolving to String
  
All of this allowing for YAML config such as:

```yaml
environments:  # NovopsEnvironments
  dev:         # NovopsEnvironmentInput
    variables: # Vec<VariableInput>
        
      # VariableInput   
      - name: FOO
        value: bar  # StringResolvableInput is an enum for which String and complex value can be used

      # VariableInput   
      - name: HV
        value:      # Let's add HashiVaultKeyValueV2Input !
          hvault_kv2:
            path: app/dev
            key: db_pass
```

Add `HashiVaultKeyValueV2Input` to `StringResolvableInput` and `impl ResolveTo<String> for StringResolvableInput`:

```rust
/// src/core.rs
pub enum StringResolvableInput {
    // ...
    HashiVaultKeyValueV2Input(HashiVaultKeyValueV2Input),
}

// ...

impl ResolveTo<String> for StringResolvableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
        return match self {
            // ...
            StringResolvableInput::HashiVaultKeyValueV2Input(hv) => hv.resolve(ctx).await,
        }
    }
}
```

This will make module usable as `value` with `variables` and `content` with `files`.

## 4. (Optional) Global configuration

`.novops.yml` config also have a root `config` keyword used for global configuration derived from `NovopsConfig` in `src/core.rs`.

To add a global configuration, create a `struct HashivaultConfig`:

```rust
/// src/modules/hashivault/config.rs
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]  
pub struct HashivaultConfig {
  /// Address in form http(s)://HOST:PORT
  /// 
  /// Example: https://vault.mycompany.org:8200
  pub address: Option<String>,

  /// Vault token as plain string
  /// 
  /// Use for testing only. DO NOT COMMIT NOVOPS CONFIG WITH THIS SET.
  /// 
  pub token: Option<String>,

  /// Vault token path.
  /// 
  /// Example: /var/secrets/vault-token
  pub token_path: Option<PathBuf>,

  /// Whether to enable TLS verify (true by default)
  pub verify: Option<bool>
}
```

And add it to `struct NovopsConfig`:

```rust
/// src/core.rs
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsConfig {
    // ...
    pub hashivault: Option<HashivaultConfig>
}
```

Structure content will now be passed to `ResolveTo<E>` via `ctx` and can be used to define module behaviour globally:

```rust
impl ResolveTo<String> for HashiVaultKeyValueV2Input {
  async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
    
    // create client for specified address
    let client = get_client(ctx)?;

    // ...
  }
}
```

## Testing

Tests are implemented under `tests/test_<module_name>.rs`. 

Most tests are integration tests using Docker containers for external system and a dedicated `.novops.<MODULE>.yml` file with related config.

If you depends on external component (such as Hashivault instance), use Docker container to spin-up a container and configure it accordingly. See `tests/docker-compose.yml`