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
    echo "Available commands:"
    echo "  cargo build          - Build the project"
    echo "  cargo build --release - Build optimized release"
    echo "  cargo run            - Build and run"
    echo "  cargo test           - Run tests"
    echo "  cargo clippy         - Run linter"
    echo "  cargo fmt            - Format code"
    echo ""
    echo "Run ./target/release/v2ex-tui --help for usage"
  '';
}
