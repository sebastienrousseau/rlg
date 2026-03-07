{
  description = "RLG (RustLogs) — High-Performance Lock-Free Observability Engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "miri" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        
        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.systemd ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.apple_sdk.frameworks.Security ];
        };

        rlg-pkg = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
      in
      {
        checks = {
          inherit rlg-pkg;
        };

        packages.default = rlg-pkg;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [
            rustToolchain
            cargo-miri
            pkg-config
          ] ++ pkgs.lib.optionals stdenv.isLinux [ systemd ];
          
          shellHook = ''
            export RUST_BACKTRACE=1
            export MIRIFLAGS="-Zmiri-tree-borrows"
          '';
        };
      });
}
