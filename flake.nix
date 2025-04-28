{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-flake.url = "github:juspay/rust-flake";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    rust-manifest = {
      url = "https://static.rust-lang.org/dist/channel-rust-stable.toml";
      flake = false;
    };
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.rust-flake.flakeModules.default
        inputs.rust-flake.flakeModules.nixpkgs
        ./nix/rust.nix
      ];
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { self', inputs', pkgs, ... }: {
        # _module.args.pkgs = import inputs.nixpkgs pkgConfig;
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self'.devShells.rust ];
          packages = [
            inputs'.fenix.packages.rust-analyzer
          ];
        };
      };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.

      };
    };
}
