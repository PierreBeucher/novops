{
    description = "Minimal example Flake using Novops";

    inputs = {
        novops.url = "github:novadiscovery/novops"; 
    };

    outputs = { self, nixpkgs, novops }: {
        devShells."x86_64-linux".default = nixpkgs.legacyPackages."x86_64-linux".mkShell {
            packages = [ 
                novops.packages."x86_64-linux".novops
            ];
            shellHook = ''
                # Run novops on shell startup
                novops load -s .envrc && source .envrc
            '';
        };
    };
}