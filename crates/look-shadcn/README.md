# gpui-luma-look-shadcn

Shadcn theme and look adapter for **GPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides token resolution, CSS stylesheet ingestion, shadow ladders, light/dark mode transitions, and concrete control builders adhering to the Shadcn design language.

## Features

* **CSS Token Engine**: Parses CSS variable catalogs (`parse_css_catalog`) with embedded fallback stylesheet support.
* **Control Builders**: Pre-styled builders for Button, Input, Checkbox, Radio, Switch, Slider, Tabs, TreeView, Table, and Overlays.
* **Theme Inspection**: Optional `inspect` feature providing runtime token resolution introspection for developer tooling.

## Installation

```toml
[dependencies]
luma = { package = "gpui-luma", version = "0.1.0" }
luma-look-shadcn = { package = "gpui-luma-look-shadcn", version = "0.1.0" }
```

## Quick Start

```rust,no_run
use gpui::*;
use luma_look_shadcn::{ShadcnLook, Button, ShadcnSize};

fn render_save_button(cx: &mut App) {
    let look = ShadcnLook::built_in();
    // Use look builders to construct and spawn controls
}
```
