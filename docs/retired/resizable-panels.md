# Resizable Panels

`ResizablePanels` is a Luma SDK control for arbitrary 2+ pane layouts with mixed pixel and weight sizing. It complements
`SplitView`, which targets navigation/content application shells with pixel sidebar width and collapse.

## Module

```text
crates/sdk/src/controls/resizable_panels/
  math.rs
  model.rs
  control.rs
  template.rs
  theme.rs
  macros.rs
  mod.rs
```

## Public API

```rust
use gpui_luma::resizable_panels;

let panels = resizable_panels! {
    cx,
    radix = theme,
    id: "demo",
    layout: Horizontal,
    size: (px(540.0), px(220.0)),
    panels: [
        move || sidebar => px(280.0), min: px(200.0), max: px(400.0), bg: sidebar_bg;
        |
        move || content => weight(1.0), bg: content_bg;
    ]
};
```

## Events

- `ResizeStart`
- `SizesChanged { sizes_px }` — main-axis pixel width/height per panel
- `ResizeEnd { sizes_px }`

## Gallery

See **Layout → Resizable Panels** in `gpui-luma-gallery`.

## Prior art

- `gpui-opal/crates/sdk/src/controls/layout/resizable`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/resizable.rs`
