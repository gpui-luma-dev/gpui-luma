# Resizable Panels

`ResizablePanels` is a Luma SDK control for arbitrary 2+ pane layouts sized in percents. It complements
`SplitView`, which targets navigation/content application shells with pixel sidebar width and collapse.

## Module

```text
crates/sdk/src/controls/resizable_panels/
  math.rs
  model.rs
  control.rs
  template.rs
  theme.rs
  mod.rs
```

## Public API

```rust
let panels = ResizablePanels::new("demo")
    .orientation(ResizablePanelsOrientation::Horizontal)
    .size(px(540.0), px(220.0)) // optional; omit to fill parent
    .panel(
        ResizablePanelSpec::new_render(|| sidebar_content)
            .default_size(30.0)
            .min_size(20.0)
            .max_size(70.0),
    )
    .panel(ResizablePanelSpec::new_render(|| content))
    .spawn(cx);
```

## Events

- `ResizeStart`
- `SizesChanged { sizes }`
- `ResizeEnd { sizes }`

## Gallery

See **Layout → Resizable Panels** in `gpui-luma-gallery` (ported from Opal `resizable` demos).

## Prior art

- `gpui-opal/crates/sdk/src/controls/layout/resizable`
- `gpui-opal/apps/gallery/src/gallery/pages/layout/resizable.rs`
