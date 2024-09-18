{
  description = "Half shell, half mux - shux";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          rustToolchainFile = (pkgs.lib.importTOML ./rust-toolchain.toml).toolchain;
          rustToolchain = (inputs'.fenix.packages.fromToolchainName {
            name = rustToolchainFile.channel;
            sha256 = "sha256-VZZnlyP69+Y3crrLHQyJirqlHrTtGTsyiSnZB8jEvVo=";
          });
          rust = rustToolchain.toolchain;
          rustPlatform = pkgs.makeRustPlatform {
            inherit (rustToolchain) rustc cargo;
          };
        in
        {
          packages.default = rustPlatform.buildRustPackage {
            pname = "shux";
            version = "0.0.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };

          devShells.default = pkgs.mkShell { buildInputs = [ rust pkgs.bacon ]; };

          formatter = pkgs.nixfmt-rfc-style;
        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.

      };
    };
}
