set shell := ["bash", "-cu"]

gallery:
    cargo run -p gpui-luma-gallery -- elegent-luxury

gallery-rel:
    cargo run -p gpui-luma-gallery --release

gallery-dbg:
    RUST_BACKTRACE=1 cargo run -p gpui-luma-gallery -- jarvis

theme-studio:
    cargo run -p gpui-luma-theme-studio -- elegent-luxury

theme-studio-rel:
    cargo run -p gpui-luma-theme-studio --release

loc:
    tokei --types Rust

clippy:
    cargo clippy --all-targets
