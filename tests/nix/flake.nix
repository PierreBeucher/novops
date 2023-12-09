{
    description = "Minimal example Flake using Novops";

    nixConfig = {
        extra-substituters = [
            "https://novops.cachix.org"
        ];
        extra-trusted-public-keys = [
            "novops.cachix.org-1:xm1fF2MoVYRmg89wqgQlM15u+2bk0LBfVktN9EgDaHY="
        ];
    };
    
    inputs = {
        novops.url = "github:PierreBeucher/novops"; 
    };

    outputs = { self, nixpkgs, novops }: {
        devShells."x86_64-linux".default = nixpkgs.legacyPackages."x86_64-linux".mkShell {
            packages = [ 
                novops.packages."x86_64-linux".novops
            ];
            shellHook = ''
                # Run novops on shell startup
                source <(novops load)
            '';
        };
    };
}