# Microsoft Azure

## Authentication

Login with `az` CLI is enough. Novops use [`azure_identity`](https://crates.io/crates/azure_identity) `DefaultAzureCredential`. Provide credentials via:

- [Environment variables](https://docs.rs/azure_identity/0.9.0/azure_identity/struct.EnvironmentCredential.html)
- [Azure CLI](https://docs.rs/azure_identity/0.9.0/azure_identity/struct.AzureCliCredential.html)
- [Managed Identity](https://docs.rs/azure_identity/0.9.0/azure_identity/struct.ImdsManagedIdentityCredential.html)

## Key Vault

Retrieve secrets from [Key Vaults](https://azure.microsoft.com/en-us/products/key-vault/) as files or variables:

```yaml
environments:
  dev:
    variables:
      - name: AZ_KEYVAULT_SECRET_VAR
        value:
          azure_keyvault_secret:
            vault: my-vault
            name: some-secret
  
    files:
      - name: AZ_KEYVAULT_SECRET_FILE
        content:
          azure_keyvault_secret:
            vault: my-vault
            name: some-secret
            version: 1234118a41364a9e8a086e76c43629e4
```
