set shell := ["bash", "-cu"]

gallery:
    cargo run -p gpui-luma-gallery

gallery-rel:
    cargo run -p gpui-luma-gallery --release

loc:
    tokei --types Rust

clippy:
    cargo clippy --all-targets
