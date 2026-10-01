# gpui-luma-look-radix

Radix theme and look adapter for **GPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides palette scales, semantic token resolution, custom color generation, light/dark mode handling, and concrete control builders adhering to the Radix design language.

## Features

* **Stepped Palette Scales**: Full 12-step Radix color scales for accent and neutral palettes.
* **Accessible States**: Built-in high-contrast variants, focus metrics, and alpha steps.
* **Control Builders**: Pre-styled builders for Button, Checkbox, Radio, Switch, Slider, Tabs, Toolbar, PopupMenu, and TreeView.
* **Display Elements**: Direct element implementations for Avatar, Badge, Card, and Callout.

## Installation

```sh
cargo add gpui-luma --rename luma --git https://github.com/scottcg/gpui-luma
cargo add gpui-luma-look-radix --rename luma-look-radix --git https://github.com/scottcg/gpui-luma
```

## Quick Start

```rust,no_run
use gpui::*;
use luma_look_radix::{Look, Button, ButtonSize};

fn save_button<M: 'static>(look: &Look, cx: &mut Context<M>) -> Entity<luma::controls::button::Button> {
    Button::new("save")
        .look(look)
        .solid()
        .size(ButtonSize::Two)
        .label("Save")
        .spawn(cx)
}
```
