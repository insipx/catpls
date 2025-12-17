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
      rust-project.crates."catpls" = {
        imports = [ ];
        crane = {
          args = {
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.lib.getLib pkgs.openssl}/lib";
            OPENSSL_NO_VENDOR = 1;
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
          };
        };
      };
      rust-project = {
        inherit toolchain;
      };
      packages.default = self'.packages.catpls;
    };
}
