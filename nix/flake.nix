{
  description = "EarDo development / release-build shell (x86_64 + aarch64 Linux)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      }));
    in {
      devShells = forAll (pkgs:
        let
          rust = pkgs.rust-bin.nightly."2026-10-20".minimal.override {
            extensions = [ "rustfmt" "clippy" ];
            targets = [ "wasm32-unknown-unknown" ];
          };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rust cargo-leptos wasm-bindgen-cli_0_2_121 binaryen tailwindcss_4
              nodejs_22 pkg-config gcc gnumake cacert curl git jq postgresql_16
            ];
            buildInputs = [ pkgs.openssl ];
            LEPTOS_OUTPUT_NAME = "eardo";
            LEPTOS_TAILWIND_VERSION = "v4.3.3";
          };
        });
    };
}
