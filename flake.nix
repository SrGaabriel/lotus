{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, fenix }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      toolchain = fenix.packages.${system}.latest.toolchain;
      rustPlatform = pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      };
    in
    {
      packages.${system} = {
        lopus = rustPlatform.buildRustPackage {
          pname = "lopus";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "ariadne-0.7.0" = "sha256-O8dh1XMtTaPNyQ5DsdGWGVJZL/3I9Tkw7rMSMwT7h6A=";
            };
          };
          cargoBuildFlags = [ "--package" "lopus" ];
        };
        default = self.packages.${system}.lopus;
      };
    };
}
