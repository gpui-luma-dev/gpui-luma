set shell := ["bash", "-cu"]

# --- Apps (Release by default) ---

luma-studio:
    cargo run -p luma-studio --release

luma-radix:
    cargo run -p luma-radix-studio --release

luma-plot:
    cargo run -p luma-plot-viz --release

neumorphic-demo:
    cargo run -p luma-neumorphic-demo --release

shell-detached:
    cargo run -p luma-shell-detached --release -- default

shell-titlebar:
    cargo run -p luma-shell-split-titlebar --release -- default

shell-vscode:
    cargo run -p luma-shell-vscode --release -- default

shell-2026:
    cargo run -p luma-shell-2026 --release -- default

# --- Apps (Debug / Profiling) ---

luma-studio-debug:
    cargo run -p luma-studio

luma-studio-trace:
    RUST_BACKTRACE=1 cargo run -p luma-studio

luma-studio-perf:
    MTL_HUD_ENABLED=1 cargo run -p luma-studio --release

luma-radix-debug:
    cargo run -p luma-radix-studio

luma-plot-debug:
    cargo run -p luma-plot-viz

neumorphic-demo-debug:
    cargo run -p luma-neumorphic-demo

shell-detached-debug:
    cargo run -p luma-shell-detached -- default

shell-titlebar-debug:
    cargo run -p luma-shell-split-titlebar -- default

shell-vscode-debug:
    cargo run -p luma-shell-vscode -- default

shell-2026-debug:
    cargo run -p luma-shell-2026 -- default

# --- SDK / Library (Fast) ---

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

# --- Workspace / Full Verification ---

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

# Check a PR locally without launching apps or building deployment artifacts.
# Requires Node.js, actionlint, and the wasm32-unknown-unknown Rust target.
pre-pr-checks:
    @command -v node >/dev/null || { echo "Install Node.js and add node to PATH." >&2; exit 1; }
    @command -v actionlint >/dev/null || { echo "Install actionlint and add it to PATH (macOS: brew install actionlint)." >&2; exit 1; }
    cargo fmt --all --check
    actionlint
    python3 -B -m unittest discover -s scripts -p 'test_*.py'
    python3 -B -m unittest discover -s apps/docs-embed -p 'test_*.py'
    node --check docs/site.js
    node --check docs/navigation.js
    node --check docs/examples/embed.js
    cargo clippy --locked --workspace --all-targets --all-features
    cargo test --locked --lib --bins --tests --all-features -p gpui-luma -p gpui-luma-color -p gpui-luma-plot -p gpui-luma-look-radix -p gpui-luma-look-shadcn -p luma-app-common
    cargo test --locked --doc --all-features -p gpui-luma -p gpui-luma-look-radix -p gpui-luma-look-shadcn -p gpui-luma-plot
    python3 -B scripts/check_control_docs.py
    env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo clippy --locked -p luma-docs-embed --target wasm32-unknown-unknown -- -D warnings
    git diff --check

# --- Documentation & Utilities ---

docs-embed:
    sh apps/docs-embed/build.sh

docs-serve:
    python3 -m http.server 8766 --bind 127.0.0.1 --directory docs

loc:
    tokei --types Rust
