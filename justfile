set shell := ["bash", "-cu"]

gallery:
    cargo run -p gpui-luma-gallery -- astrovista

gallery-rel:
    cargo run -p gpui-luma-gallery --release

gallery-dbg:
    RUST_BACKTRACE=1 cargo run -p gpui-luma-gallery -- jarvis
loc:
    tokei --types Rust

clippy:
    cargo clippy --all-targets
