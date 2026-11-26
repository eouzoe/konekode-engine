{ pkgs, inputs, ... }:
let
  rustBin = inputs.rust-overlay.lib.mkRustBin { } pkgs;
  nightly = rustBin.nightly."2026-11-25".minimal.override {
    extensions = [ "llvm-tools-preview" "rust-src" ];
  };
in
{
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };
  packages = [
    pkgs.just
    pkgs.wasm-bindgen-cli
    pkgs.cargo-llvm-cov
    pkgs.cargo-mutants
    pkgs.cargo-fuzz
    pkgs.elan
  ];
  env.NIGHTLY = "${nightly}";
}
