## Modules usage

A module allow retrieval or generation of secrets/values within Novops. Two kind of module exists:

- **Loader:** load values (secret or not) from external sources, such as Hashicorp Vault
- **Generator:** generate values from provided input, such as AWS module generating credentials for IAM Roles

Available modules:
- Hashicorp Vault:
  - [Key Value Version 2](https://www.vaultproject.io/docs/secrets/kv/kv-v2) 
  - _More to come..._
- AWS
  - [IAM AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html)
  - _More to come..._
- BitWarden - _experimental and untested, use with care_

Wanna add a module? See [contribution guide](../CONTRIBUTING.md) !

### Hashicorp Vault

[Key Value Version 2](https://www.vaultproject.io/docs/secrets/kv/kv-v2) with variables and files:

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

Generate temporary [IAM Role credentials with AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html):

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