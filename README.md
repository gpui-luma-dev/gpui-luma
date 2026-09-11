# gpui-luma

> **Early version.** This project is pre-release / beta. APIs, crate layout, and examples may change without notice.

GPUI component library. Requires a recent Rust stable toolchain (`rust-toolchain.toml` pins `stable`).

## Build

```bash
git clone https://github.com/scottcg/gpui-luma.git
cd gpui-luma
cargo build
```

Library crates of interest: `gpui-luma-core`, `gpui-luma-color`, `gpui-luma-look-core`, `gpui-luma-look-shadcn`, `gpui-luma-look-radix`.

## Example programs

**Luma Studio** — Shadcn look workbench (control docs, theme inspection):

```bash
cargo run -p luma-studio
# or: just luma-studio
```

**Luma Radix Studio** — Radix look stub workbench:

```bash
cargo run -p luma-radix-studio
# or: just luma-radix
```

Release builds: `cargo run -p luma-studio --release` / `cargo run -p luma-radix-studio --release` (or `just luma-studio-rel` / `just luma-radix-rel`).

## Prebuilt executables

The [Build Luma Studio](https://github.com/scottcg/gpui-luma/actions/workflows/build.yml) workflow publishes macOS and Windows `luma-studio` binaries as Actions artifacts (download from a successful run).
