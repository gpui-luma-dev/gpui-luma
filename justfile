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
    cargo run -p luma-color-viz --release -- elegent-luxury

color-viz-rel:
    MTL_HUD_ENABLED=1 cargo run -p luma-color-viz --release -- retro-arcade

# graph-viz app is not in the workspace currently
# graph-viz:
#     cargo run -p luma-graph-viz -- awesome-open-claw

# SplitView shell demos (apps/shells/)

shell:
    cargo run -p luma-shell-unified -- default

shell-inset:
    cargo run -p luma-shell-inset -- default

shell-rail:
    cargo run -p luma-shell-icon-rail -- default

shell-detached:
    cargo run -p luma-shell-detached -- default

shell-titlebar:
    cargo run -p luma-shell-split-titlebar -- default

shell-vscode:
    cargo run -p luma-shell-vscode -- default

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
