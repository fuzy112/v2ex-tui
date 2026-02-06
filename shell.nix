{ pkgs ? import <nixpkgs> {}, withRustOverlay ? false, rustToolchain ? null }:

let
  effectivePkgs = if withRustOverlay && rustToolchain != null then
    pkgs // { rustToolchain = rustToolchain; }
  else
    pkgs;
in

pkgs.mkShell {
   buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    pkg-config
    openssl
    # Additional tools
    git
    ripgrep
    fd
    python3
  ];

  RUST_BACKTRACE = 1;
  
  shellHook = ''
    echo "V2EX TUI Development Environment"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
  '';
}
