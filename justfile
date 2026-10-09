set shell := ["bash", "-cu"]

luma-studio:
    #cargo run -p luma-studio -- modern-minimal --light
    cargo run -p luma-studio

luma-studio-trace:
    RUST_BACKTRACE=1 cargo run -p luma-studio

luma-studio-rel:
    cargo run -p luma-studio --release

luma-studio-perf:
    MTL_HUD_ENABLED=1 cargo run -p luma-studio --release

luma-radix:
    cargo run -p luma-radix-studio

luma-radix-rel:
    cargo run -p luma-radix-studio --release

neumorphic-demo:
    cargo run -p luma-neumorphic-demo

neumorphic-demo-rel:
    cargo run -p luma-neumorphic-demo --release

shell-detached:
    cargo run -p luma-shell-detached -- default

shell-titlebar:
    cargo run -p luma-shell-split-titlebar -- default

shell-vscode:
    cargo run -p luma-shell-vscode -- default

shell-2026:
    cargo run -p luma-shell-2026 -- default

loc:
    tokei --types Rust

# SDK / Library (Fast)
check:
    cargo check

build:
    cargo build

test-sdk:
    cargo test

clippy-sdk:
    cargo clippy --all-targets --all-features

doc-test:
    cargo test --doc

# Workspace / Full Verification
check-all:
    cargo check --workspace --all-targets

build-all:
    cargo build --workspace

clippy:
    cargo clippy --workspace --all-targets

format-check:
    cargo fmt --all --check

format:
    cargo fmt --all

test:
    cargo test --workspace --all-targets

ci: format-check clippy test
