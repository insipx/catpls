{ inputs, ... }:
{
  debug = true;
  perSystem =
    { inputs', pkgs, lib, self', ... }:
    let
      inherit (pkgs.stdenv) isDarwin;
      inherit (pkgs.darwin) apple_sdk;
      rust = inputs'.fenix.packages.fromManifestFile inputs.rust-manifest;
      toolchain = inputs'.fenix.packages.combine [
        rust.defaultToolchain
        rust."clippy"
        rust."rust-docs"
        rust."rustfmt-preview"
        rust."clippy-preview"
      ];
    in
    {
      rust-project.crates."cat-rest-v2" = {
        imports = [ ];
        crane = {
          args = {
            buildInputs =
              lib.optionals isDarwin
                [
                  apple_sdk.frameworks.Security
                ]
              ++ [
                pkgs.openssl
              ];
          };
        };
      };
      rust-project = {
        inherit toolchain;
      };
      packages.default = self'.packages.cat-rest-v2;
    };
}
