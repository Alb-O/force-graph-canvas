{
  description = "proj";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              (rust-bin.nightly.latest.default.override {
                targets = [ "wasm32-unknown-unknown" ];
                extensions = [ "rust-src" ];
              })
              openssl
              pkg-config
              cargo-sort
              cargo-generate
              leptosfmt
              tailwindcss_4
              trunk
              dart-sass
              wasm-bindgen-cli_0_2_104
            ];
            shellHook = ''
              export PATH=$CARGO_HOME/bin:$PATH
            '';
          };
      }
    );
}
