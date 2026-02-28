{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ (import rust-overlay) ]; };
        rust = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-wasip2" "wasm32-wasip1" "x86_64-unknown-linux-gnu" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ 
            rust 
            pkgs.sqlite 
            pkgs.openssl
            pkgs.nodejs_latest
            pkgs.pnpm
            pkgs.just
            pkgs.nodePackages.typescript-language-server
          ];
          shellHook = "echo 'Scry Dev Shell (Rust + Node.js Latest + pnpm)'";
        };
      });
}
