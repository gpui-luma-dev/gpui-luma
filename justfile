set shell := ["bash", "-cu"]

gallery:
    cargo run -p gpui-luma-gallery

gallery-rel:
    cargo run -p gpu-luma-gallery --release
