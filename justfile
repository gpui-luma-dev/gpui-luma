set shell := ["bash", "-cu"]

luma-studio:
    cargo run -p luma-studio -- modern-minimal --light

luma-studio-trace:
    RUST_BACKTRACE=1 cargo run -p luma-studio

luma-studio-rel:
    cargo run -p luma-studio --release

luma-studio-perf:
    MTL_HUD_ENABLED=1 cargo run -p luma-studio --release

neumorphic-demo:
    cargo run -p gpui-luma-neumorphic-demo

neumorphic-demo-rel:
    cargo run -p gpui-luma-neumorphic-demo --release

color-viz:
    cargo run -p gpui-luma-color-viz -- elegent-luxury

color-viz-rel:
    MTL_HUD_ENABLED=1 cargo run -p gpui-luma-color-viz --release -- retro-arcade

graph-viz:
    cargo run -p gpui-luma-graph-viz -- awesome-open-claw

# SplitView shell demos (apps/shells/)

shell:
    cargo run -p gpui-luma-shell-unified -- default

shell-inset:
    cargo run -p gpui-luma-shell-inset -- default

shell-rail:
    cargo run -p gpui-luma-shell-icon-rail -- default

shell-detached:
    cargo run -p gpui-luma-shell-detached -- default

shell-titlebar:
    cargo run -p gpui-luma-shell-split-titlebar -- default

shell-vscode:
    cargo run -p gpui-luma-shell-vscode -- default

loc:
    tokei --types Rust

clippy:
    cargo clippy --all-targets

format-check:
    cargo fmt --all --check

format:
    cargo fmt --all
