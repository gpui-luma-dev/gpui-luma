# gpui-luma-look-shadcn

Shadcn theme and look adapter for **GPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides token resolution, CSS stylesheet ingestion, shadow ladders, light/dark mode transitions, and concrete control builders adhering to the Shadcn design language.

## Features

* **CSS Token Engine**: Parses CSS variable catalogs (`parse_css_catalog`) with embedded fallback stylesheet support.
* **Control Builders**: Pre-styled builders for Button, Input, Checkbox, Radio, Switch, Slider, Tabs, TreeView, Table, and Overlays.
* **Theme Inspection**: Optional `inspect` feature providing runtime token resolution introspection for developer tooling.

## Installation

```sh
cargo add gpui-luma
cargo add gpui-luma-look-shadcn
cargo add gpui-unofficial --rename gpui
```

The package is named `gpui-luma-look-shadcn`; Rust imports use `gpui_luma_look_shadcn`.

## Quick Start

```rust,no_run
use gpui::{Context, Entity};
use gpui_luma::infra::presenter::HasPresenter;
use gpui_luma_look_shadcn::{ShadcnLook, Button, ShadcnSize};

fn save_button<M: 'static>(
    look: &ShadcnLook,
    cx: &mut Context<M>,
) -> Entity<gpui_luma::controls::button::Button> {
    Button::new("save")
        .look(look)
        .primary()
        .size(ShadcnSize::Md)
        .label("Save")
        .spawn(cx)
}
```

Use `ShadcnLook::built_in()` for the bundled fallback theme, or
`ShadcnLook::from_css_str` to parse your application's theme CSS.

To enable optional theme inspection:

```sh
cargo add gpui-luma-look-shadcn --features inspect
```

## Build

In your application, run `cargo check` or `cargo build`.
From the Luma repository, run `cargo check -p gpui-luma-look-shadcn`.

## License

Apache-2.0. See `LICENSE` in the package. Third-party material retains its
own license terms.
