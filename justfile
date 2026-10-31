# Run from the repository root. Windows: put Git for Windows Bash on PATH.
set shell := ["bash", "-cu"]
set windows-shell := ["bash", "-cu"]
set positional-arguments

# App launches and explicit builds default to release; checks/tests use Cargo defaults.
profile := "release"
python := if os() == "windows" { "python" } else { "python3" }
run-options := "--locked --profile " + profile

# List development commands without launching an app.
default:
    @just --justfile "{{ justfile() }}" --list

# --- Apps ---

# Run Luma Studio; accepts a theme and --light/--dark.
app-studio *args:
    cargo run {{ run-options }} -p luma-studio -- "$@"

# Run Radix Studio.
app-radix *args:
    cargo run {{ run-options }} -p luma-radix-studio -- "$@"

# Run Plot Viz.
app-plot *args:
    cargo run {{ run-options }} -p luma-plot-viz -- "$@"

# Run the Neumorphic Demo.
app-neumorphic *args:
    cargo run {{ run-options }} -p luma-neumorphic-demo -- "$@"

# Run Luma Studio with panic backtraces enabled.
app-studio-backtrace *args:
    RUST_BACKTRACE=1 cargo run {{ run-options }} -p luma-studio -- "$@"

# --- Shells (optional theme; defaults to default) ---

# Run the detached shell.
shell-detached *args:
    cargo run {{ run-options }} -p luma-shell-detached -- "$@"

# Run the split-titlebar shell.
shell-split-titlebar *args:
    cargo run {{ run-options }} -p luma-shell-split-titlebar -- "$@"

# Run the VS Code-style shell.
shell-vscode *args:
    cargo run {{ run-options }} -p luma-shell-vscode -- "$@"

# Run the 2026 shell.
shell-2026 *args:
    cargo run {{ run-options }} -p luma-shell-2026 -- "$@"

# --- Profiling ---

# Run Luma Studio with the macOS Metal HUD enabled.
perf-studio *args:
    MTL_HUD_ENABLED=1 cargo run {{ run-options }} -p luma-studio -- "$@"

# Run the native SDK fixture: <scene> <look> <size> [--drive|--idle].
perf-sdk *args:
    cargo run {{ run-options }} -p luma-sdk-perf --features native -- "$@"

# --- SDK / Libraries ---

# Check the default SDK/library workspace members.
sdk-check *args:
    cargo check --locked "$@"

# Build the default SDK/library workspace members.
sdk-build *args:
    cargo build {{ run-options }} "$@"

# Run the library suites selected by CI, excluding benchmarks and demo binaries.
sdk-test *args:
    cargo test --locked --lib --bins --tests --all-features -p gpui-luma -p gpui-luma-color -p gpui-luma-plot -p gpui-luma-look-radix -p gpui-luma-look-shadcn -p luma-app-common "$@"

# Lint default SDK/library members with every target and feature.
sdk-clippy *args:
    cargo clippy --locked --all-targets --all-features "$@"

# Run documentation tests for the library packages selected by CI.
sdk-doc-test *args:
    cargo test --locked --doc --all-features -p gpui-luma -p gpui-luma-look-radix -p gpui-luma-look-shadcn -p gpui-luma-plot "$@"

# --- Workspace ---

# Check all workspace targets and features, including the native profiling app.
workspace-check *args:
    cargo check --locked --workspace --all-targets --all-features "$@"

# Build the full workspace, including the native profiling app.
workspace-build *args:
    cargo build {{ run-options }} --workspace --all-features "$@"

# Run workspace unit/integration tests, excluding benchmark executables.
workspace-test *args:
    cargo test --locked --workspace --lib --bins --tests --all-features "$@"

# Lint the full workspace with the same targets and features as CI.
workspace-clippy *args:
    cargo clippy --locked --workspace --all-targets --all-features "$@"

# Build all native app binaries without launching them.
apps-build *args:
    cargo build {{ run-options }} --bins -p luma-studio -p luma-radix-studio -p luma-plot-viz -p luma-neumorphic-demo -p luma-shell-detached -p luma-shell-split-titlebar -p luma-shell-vscode -p luma-shell-2026 -p luma-sdk-perf --features luma-sdk-perf/native "$@"

# --- Formatting ---

# Format Rust source across the workspace.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all --check

# --- Documentation / Supporting Checks ---

# Build and package the embedded WASM demos (profile: release or dev).
docs-build:
    sh apps/docs-embed/build.sh "{{ profile }}"

# Check site JavaScript and compile the control documentation examples.
docs-check: _node-check _control-docs-check
    node --check docs/site.js
    node --check docs/navigation.js
    node --check docs/examples/embed.js

# Serve the documentation locally without opening a browser.
docs-serve port="8766":
    {{ python }} -m http.server "$1" --bind 127.0.0.1 --directory docs

# Test repository and embedded-demo Python helpers.
scripts-check:
    {{ python }} -B -m unittest discover -s scripts -p 'test_*.py'
    {{ python }} -B -m unittest discover -s apps/docs-embed -p 'test_*.py'

# Validate GitHub Actions workflows (requires actionlint).
workflows-check: _actionlint-check
    actionlint

# Lint the WASM demos (requires the wasm32-unknown-unknown target).
wasm-check:
    env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo clippy --locked -p luma-docs-embed --target wasm32-unknown-unknown -- -D warnings

# --- Aggregate Checks ---

# Run the local equivalents of CI formatting, script, lint and library/doc checks.
ci: fmt-check scripts-check workspace-clippy sdk-test sdk-doc-test _control-docs-check

# Check a PR without launching apps or building deployment artifacts.
pre-pr: _node-check _actionlint-check ci workflows-check docs-check wasm-check
    git diff --check

# --- Utilities ---

# Count Rust source lines (requires tokei).
loc:
    tokei --types Rust

# --- Internal Prerequisites ---

_node-check:
    @command -v node >/dev/null || { echo "Install Node.js and add node to PATH." >&2; exit 1; }

_actionlint-check:
    @command -v actionlint >/dev/null || { echo "Install actionlint and add it to PATH (macOS: brew install actionlint)." >&2; exit 1; }

_control-docs-check:
    {{ python }} -B scripts/check_control_docs.py
