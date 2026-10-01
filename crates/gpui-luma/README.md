# gpui-luma

Primary consumer facade crate for the **GPUI-Luma** UI framework ([gpui-luma.dev](https://gpui-luma.dev)).

Re-exports the core control SDK ([`gpui-luma-core`](https://crates.io/crates/gpui-luma-core)) and shared look provenance. Applications typically depend on `gpui-luma` and **one** look adapter.

## Installation

Add `gpui-luma` and your chosen look to `Cargo.toml`:

```toml
[dependencies]
luma = { package = "gpui-luma", version = "0.1.0" }
luma-look-shadcn = { package = "gpui-luma-look-shadcn", version = "0.1.0" }
# or: luma-look-radix = { package = "gpui-luma-look-radix", version = "0.1.0" }
```

## Quick Start

```rust,no_run
use gpui::*;
use luma::init as init_luma;
use luma_look_shadcn::ShadcnLook;

fn main() {
    Application::new().run(|cx: &mut App| {
        init_luma(cx);
        // Initialize chosen look and spawn your application window
    });
}
```

## Look System

GPUI-Luma decouples component structure and event dispatch from visual styling:
* **Facade / Core**: `gpui-luma` / `gpui-luma-core` (control layout, focus, dnd, key handling, accessibility).
* **Shadcn Look**: `gpui-luma-look-shadcn` (CSS-driven tokens, mode transitions, Radix-like utility style).
* **Radix Look**: `gpui-luma-look-radix` (stepped palette scales, high-contrast accessible states).
* **Color Primitives**: `gpui-luma-color` (palette modeling, color rings, sliders, and pickers).
