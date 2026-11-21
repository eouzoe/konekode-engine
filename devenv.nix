{ pkgs, ... }:
{
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };
  packages = [ pkgs.just pkgs.wasm-bindgen-cli ];
}
