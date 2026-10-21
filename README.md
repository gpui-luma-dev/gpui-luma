# gpui-luma

GPUI component library with a lookless control SDK, Shadcn and Radix look adapters,
color controls, and desktop example apps. Documentation is hosted at
[gpui-luma.dev](https://gpui-luma.dev). Source lives at
[gpui-luma-dev/gpui-luma](https://github.com/gpui-luma-dev/gpui-luma).

Requires a recent Rust stable toolchain (`rust-toolchain.toml` tracks `stable`).

## Build an app

Use the published crates directly; a GitHub checkout is only needed to run the
repository’s example apps or contribute to the libraries.

```bash
cargo new my-luma-app
cd my-luma-app
cargo add gpui-luma@0.2 gpui-luma-look-shadcn@0.2
cargo add gpui-unofficial@=1.24.2 --rename gpui
cargo add gpui-platform-gpui-unofficial@=1.24.2 --rename gpui_platform --features font-kit
```

Add the [complete app example](https://gpui-luma.dev/#sample) to `src/main.rs`, then
run `cargo build` or `cargo run`. Choose the Radix look instead of Shadcn, or use
the SDK’s built-in styling. Add color controls and plots as needed.

Use `0.2` for Luma dependencies to accept compatible `0.2.x` releases. Run
`cargo update` to refresh versions already recorded in `Cargo.lock`.

```toml
[dependencies]
gpui-luma = "0.2"
gpui-luma-look-shadcn = "0.2"
# Alternatively: gpui-luma-look-radix = "0.2"
# Optional: gpui-luma-color = "0.2"
# Optional: gpui-luma-plot = "0.2"
```

## Library crates

| Crate | Purpose |
|---|---|
| [`gpui-luma`](https://crates.io/crates/gpui-luma) | Lookless controls, layouts, focus, interaction, motion, and shared theme contracts |
| [`gpui-luma-look-shadcn`](https://crates.io/crates/gpui-luma-look-shadcn) | Shadcn themes, CSS tokens, styled builders, and optional inspection tooling |
| [`gpui-luma-look-radix`](https://crates.io/crates/gpui-luma-look-radix) | Radix themes, palette scales, custom colors, and styled builders |
| [`gpui-luma-color`](https://crates.io/crates/gpui-luma-color) | Color fields, rings, sliders, swatches, and color rendering primitives |
| [`gpui-luma-plot`](https://crates.io/crates/gpui-luma-plot) | Cartesian, pie, radar, and radial charts |

Depend on **`gpui-luma`**, then choose a look adapter. Add `gpui-luma-color` for
color controls. All five library crates are available on crates.io.

Import the SDK with `use gpui_luma::…`. Look adapter imports use
`gpui_luma_look_shadcn` or `gpui_luma_look_radix`.

## Styling

Use the SDK’s built-in light/dark styling, or choose the
[Shadcn](https://gpui-luma.dev/look-shadcn.html) or
[Radix](https://gpui-luma.dev/look-radix.html) Look. See the
[web docs](https://gpui-luma.dev/looks.html) for theme configuration.

## Example programs

Clone the repository to run its desktop examples:

```bash
git clone https://github.com/gpui-luma-dev/gpui-luma.git
cd gpui-luma
```

**Use release builds for optimal app performance. Debug builds are intended for
development and can make rendering and interactions noticeably slower.**

The [justfile](justfile) runs apps in release mode by default. Use the `-debug`
recipe for a faster development build. Without `just`, run the corresponding
`cargo run -p <package> --release` command.

**Luma Studio** — Shadcn look workbench (control docs, theme inspection):

![Luma Studio](docs/screenshots/luma-shadcn.png)

```bash
just luma-studio
# Debug: just luma-studio-debug
```

**Luma Radix Studio** — Radix style guide, custom palette editor, and control previews:

![Luma Radix Studio](docs/screenshots/luma-radix.png)

```bash
just luma-radix
# Debug: just luma-radix-debug
```

**Luma Plot Viz** — chart galleries using `gpui-luma-plot` and the SDK’s built-in styling, with line, area, bar, pie, radar, and radial examples:

![Luma Plot Viz showing bar charts in dark mode](docs/screenshots/luma-plot.png)

```bash
just luma-plot
# Debug: just luma-plot-debug
```

Additional examples (also release builds by default):

```bash
just neumorphic-demo
just shell-detached
just shell-titlebar
just shell-vscode
just shell-2026
```

For profiling, use `just luma-studio-perf` to enable the Metal HUD, or
`just luma-studio-trace` to run a debug build with Rust backtraces.

The shell demos accept `default` or a built-in Shadcn theme ID,
such as `retro-arcade`, as their first argument. The justfile recipes pass
`default`; choose a theme with Cargo directly:

```bash
cargo run -p luma-shell-vscode --release -- retro-arcade
```

## Prebuilt executables

The [Build Luma Studio](https://github.com/gpui-luma-dev/gpui-luma/actions/workflows/build.yml)
workflow runs **manually only** using GitHub Actions' **Run workflow** button.
It builds `luma-studio` for macOS Apple Silicon, Windows x64, and Windows ARM64.
Download the executable ZIPs from a successful run's **Artifacts** section;
artifacts are retained for seven days.

## Development checks

From the repository root, use the library checks during development:

```bash
just check
just build
just test-sdk
just clippy-sdk
just doc-test
```

These target Cargo’s default members: the five library crates. For the full
workspace, use `just check-all`, `just build-all`, or `just ci` (format check,
workspace Clippy, and workspace tests). Run `just format` to format the workspace.
Before sending a PR, run:

```bash
just pre-pr-checks
```

This runs formatting, workflow lint, native and WASM Clippy, library tests and
doc-tests, documentation example compilation, Python helper/packaging tests,
JavaScript syntax checks, and whitespace checks. It checks all app targets with
Clippy but does not run sample app tests, launch apps, or build deployment assets.
It stops at the first failure.

Requires Node.js and [actionlint](https://github.com/rhysd/actionlint) on `PATH`,
plus the WASM Rust target (`rustup target add wasm32-unknown-unknown`). On macOS,
install the tools with `brew install node actionlint`. Native-only Rust linker
flags are cleared for the WASM check.

The `test-support` features enable headless GPUI interaction tests.

## License

Apache-2.0. See [LICENSE](LICENSE). Bundled third-party material retains its own
license terms; attribution files are included with the relevant crates and assets.
