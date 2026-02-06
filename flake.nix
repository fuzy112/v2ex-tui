{
  description = "V2EX TUI - A terminal UI viewer for V2EX";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        v2ex-tui = pkgs.rustPlatform.buildRustPackage {
          pname = "v2ex-tui";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          
          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          
          meta = with pkgs.lib; {
            description = "A terminal UI viewer for V2EX";
            homepage = "https://github.com/yourusername/v2ex-tui";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        # Import shell.nix with rust-overlay enabled
        devShell = import ./shell.nix { 
          pkgs = pkgs; 
          withRustOverlay = true;
          rustToolchain = rustToolchain;
        };
      in
      {
        # Package output
        packages = {
          default = v2ex-tui;
          v2ex-tui = v2ex-tui;
        };

        # Development shell output - uses shell.nix
        devShells.default = devShell;

        # Apps output for 'nix run'
        apps.default = flake-utils.lib.mkApp {
          drv = v2ex-tui;
        };
      });
}
