# Automatically authenticate with Hashicorp Vault before running Novops

[Hashicorp Vault](https://www.vaultproject.io/) is a popular secret manager. Typical workflow using Hashicorp Vault requires authentication and loading of multiple secrets into environment, often hard to reproduce between CI and local development environment.

Leverage Novops and [Hashicorp Vault module](../load/hashicorp-vault.md) to load secrets seamlessly, whether on CI or locally:

Create a `.novops.yml` file such as:

```yaml
name: my-app

environments:
  dev:
    variables:

      # Load  Hashicorp Vault secret as variable
      - name: HASHIVAULT_KV_V2_TEST
        value:
          hvault_kv2:
            mount: kv2
            path: test_hashivault_kv2
            key: novops_secret
    
    files:
      # Load Hashicorp Vault secret as file
      - variable: HVAULT_SECRET_PATH
        content:
          hvault_kv2:
            mount: kv2
            path: test_hashivault_kv2
            key: novops_secret
          
# Set non-sensitive Hashicorp Vault config
config:
  hashivault:
    # Hashivault from docker-compose.yml service
    # Alternatively, use VAULT_ADDR and VAULT_TOKEN env var
    address: https://vault.mycompany.org
    
    # You can pass path to a vault token
    # token_path: /path/to/vault/token
```

Developers can run `vault login` locally to generate a `VAULT_TOKEN`. 

CI systems provide ways to authenticate with Hashicorp Vault transparently:

- GitLab CI: [OpenID Connect (OIDC) Authentication Using ID Tokens](https://docs.gitlab.com/ee/ci/secrets/id_token_authentication.html#manual-id-token-authentication) combined with [Hashicorp Vault JWT/OIDC Auth Method](https://developer.hashicorp.com/vault/docs/auth/jwt)
- [GitHub Hashicorp Vault Action](https://github.com/hashicorp/vault-action) provides various authentication method
- [Jenkins Hashicorp Vault plugin](https://plugins.jenkins.io/hashicorp-vault-plugin) provides various authentication method

Alternatively, you can set CI variable `VAULT_TOKEN` to authenticate with Hashicorp Vault.

### Example setup with Nix Flake

You can setup Hashicorp Vault for Novops within a Nix Flake such as:

```nix
{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    novops.url = "github:novadiscovery/novops/main";
  };

  outputs = { self, nixpkgs, flake-utils, novops }: 
    flake-utils.lib.eachDefaultSystem (system:
      let  
        novopsPkg = novops.packages.${system}.novops;
      in {
        devShells = {
          default = pkgs.mkShell {
            packages = with pkgs; [
              novopsPkg
              vault
              # ...
            ];
            
            shellHook = ''
              # Create vault-token in XDG_RUNTIME_DIR if not exists
              # Then export token as VAULT_TOKEN
              if [ ! -f "$XDG_RUNTIME_DIR/vault-token" ]; then
                vault login -token-only > $XDG_RUNTIME_DIR/vault-token
              fi
              export VAULT_TOKEN=$(cat "$XDG_RUNTIME_DIR/vault-token")

              novops load -s .envrc && source .envrc
            '';
          };
        };
      }
    );
}
```
