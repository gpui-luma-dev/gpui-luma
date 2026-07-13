set shell := ["bash", "-cu"]

gallery:
    #cargo run -p gpui-luma-gallery -- cosmic-night
    #cargo run -p gpui-luma-gallery -- steve
    cargo run -p gpui-luma-gallery -- astrovista
    #cargo run -p gpui-luma-gallery -- elegent-luxury
    #cargo run -p gpui-luma-gallery -- jarvis

gallery-rel:
    cargo run -p gpui-luma-gallery --release

gallery-rel-hud:
    MTL_HUD_ENABLED=1 cargo run -p gpui-luma-gallery --release

gallery-dbg:
    RUST_BACKTRACE=1 cargo run -p gpui-luma-gallery -- jarvis

theme-studio:
    cargo run -p gpui-luma-theme-studio -- modern-minimal --light
    #cargo run -p gpui-luma-theme-studio -- elegent-luxury

theme-studio-rel:
    cargo run -p gpui-luma-theme-studio --release

theme-studio-perf:
    MTL_HUD_ENABLED=1 cargo run -p gpui-luma-theme-studio --release

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

loc:
    tokei --types Rust

clippy:
    cargo clippy --all-targets
