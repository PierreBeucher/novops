# Automatically authenticate with Hashicorp Vault before running Novops

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
