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

color-viz:
    cargo run -p luma-color-viz --release -- monochrome-minimal-neutral

color-viz-rel:
    MTL_HUD_ENABLED=1 cargo run -p luma-color-viz --release -- monochrome-minimal-neutral

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

clippy:
    cargo clippy --workspace --all-targets

format-check:
    cargo fmt --all --check

format:
    cargo fmt --all

test:
    cargo test --workspace --all-targets

ci: format-check clippy test
