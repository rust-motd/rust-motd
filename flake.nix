{
    description = "Development environment for rust-motd";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
    };

    outputs = { self, nixpkgs, rust-overlay, ... }:
        let
            system = "x86_64-linux";
            pkgs = import nixpkgs {
                inherit system;
                overlays = [ rust-overlay.overlays.default ];
            };

            rust = pkgs.rust-bin.stable.latest.default.override {
                extensions = [ "clippy" "rustfmt" ];
            };
        in {
            devShells.${system}.default = pkgs.mkShell {
                packages = [
                    rust
                    pkgs.rust-analyzer
                    pkgs.openssl.dev
                    pkgs.figlet
                ];

                shellHook = ''
                    export OPENSSL_DIR=${pkgs.openssl.dev}
                    export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
                    export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
                '';
            };
        };
}
