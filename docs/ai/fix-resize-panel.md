# Resizable Panels: Mixed Sizing & Overlay Handles

Design record and implementation status for [`crates/sdk/src/controls/resizable_panels/`](../../crates/sdk/src/controls/resizable_panels/).

## Implementation status (June 2026)

| Area | Status |
|------|--------|
| `PanelSize::{Absolute,Weight}` + two-pass solver | **Done** — [`math.rs`](../../crates/sdk/src/controls/resizable_panels/math.rs) |
| Overlay handles (no layout width), pick'em geometry | **Done** — [`template.rs`](../../crates/sdk/src/controls/resizable_panels/template.rs) |
| `ResizablePanelSpec::bg(Hsla)` for handle halves | **Done** |
| `ResizeHandleSize::{Sm,Md,Lg}` via `.resize_handle()` | **Done** — lane, hit target, grip scale together |
| Theme Studio main split (280px sidebar + fill) | **Done** — [`apps/theme-studio/src/studio/app.rs`](../../apps/theme-studio/src/studio/app.rs) |
| Gallery demos on weight + pixel constraints | **Done** — [`apps/gallery/.../resizable_panels/pane.rs`](../../apps/gallery/src/gallery/panes/resizable_panels/pane.rs) |
| Fluent `.pane()` builder chain | **Not started** (§7 below) |
| `resizable_panels!` macro | **Not started** (§8 below) |
| Opal-style full-viewport gallery shells | **Not started** (§9 below) |

Legacy percent APIs (`default_size`, `min_size`, `max_size`, `handle_size`) remain deprecated for existing callers; new code should use `size` / `weight` / `min` / `max` / `resize_handle`.

---

## 1. Problems this upgrade solved

The pre-upgrade control used percentage floats normalized to 100. That caused:

- Sidebars that should stay fixed in pixels to stretch when the window resized.
- Boilerplate keeping sibling percentages in sync.
- Unintuitive min/max as percentages.

Those issues are addressed by mixed sizing and overlay handles below.

---

## 2. Mixed sizing & proportional weights

### Sizing primitives

```rust
pub enum PanelSize {
    Absolute(gpui::Pixels),
    Weight(f32),
}
```

### Two-pass layout

```
┌────────────────────────────────────────────────────────┐
│                      Total Width                       │
├───────────────────────────────┬────────────────────────┤
│            Sidebar            │       Workspace        │
│           (280 px)            │        (Fill)          │
└───────────────────────────────┴────────────────────────┘
                               ▲
                       [Overlay Handle]
```

1. **Pass 1:** Subtract all `Absolute` panel sizes from the content axis. Handles do **not** consume layout width.
2. **Pass 2:** Distribute the remainder by `Weight` coefficients.

### Overlay handle alignment & pick'em snapping

Panels meet at the split line. The handle is an absolute overlay centered on the boundary.

For handle width $W$ and split $X_{\text{split}}$:

- **Odd $W$:** center on $X_{\text{split}}$; 1px divider on the split.
- **Even $W$:** pick'em rounding so the 1px divider at local offset $W/2$ aligns with $X_{\text{split}}$.

### Panel backgrounds

Adjacent panels should pass `.bg(Hsla)` on `ResizablePanelSpec`. Handle left/right (or top/bottom) halves use those colors so hover/grip feedback does not show parent track bleed.

### Resize handle presets

Use `.resize_handle(ResizeHandleSize::Sm | Md | Lg)` instead of raw pixel lane width. Each preset bundles visual lane, hit target, and grip dimensions.

---

## 3. Runtime layout state

- **Absolute:** stored as pixels; fixed across window resize.
- **Weight:** stored as coefficient; shares remainder after absolutes.

---

## 4. Interaction & drag

Dragging updates pixel space according to adjacent panel modes:

- **Absolute ↔ Weight:** delta adjusts absolute px; weight pane absorbs remainder.
- **Weight ↔ Weight:** delta shifts weight ratio in the pair.
- **Absolute ↔ Absolute:** both adjust in px (clamped).

Keyboard: absolute panes use `keyboard_step` / `keyboard_shift_step` in px; weight panes adjust coefficients.

---

## 5. Constraints

- **Absolute:** `min` / `max` in pixels.
- **Weight:** `min` / `max` in pixels applied during distribution (legacy percent min/max still supported on weight-only specs).

---

## 6. Backwards compatibility

- `default_size(f32)` → `PanelSize::Weight`
- `handle_size(Pixels)` → nearest `ResizeHandleSize` preset
- `sizes()` / `set_sizes()` on weight-only strips still report legacy 0–100 percents for telemetry

---

## 7. Future: fluent builder (not implemented)

```rust
let main_split = ResizablePanels::horizontal("theme-studio-main-split")
    .pane(sidebar_view)
        .size(px(280.0))
        .min(px(200.0))
        .max(px(400.0))
        .bg(theme.colors().sidebar_background)
    .split_handle()
    .pane(board_view)
        .weight(1.0)
        .bg(theme.colors().content_background)
    .spawn(cx);
```

Today, use `ResizablePanelsBuilder::panels([...])` with `ResizablePanelSpec` as in Theme Studio.

---

## 8. Future: `resizable_panels!` macro (not implemented)

Declarative `|` separators and nested splits — see original sketch in git history if needed.

---

## 9. Future: verification & Opal gallery shells (not implemented)

Additional unit tests (constraint edge cases under mixed drag) and full-viewport Opal layouts (layered inset, detached gap, icon rail, nested IDE) remain optional follow-on work.

**Current tests:** `cargo test -p gpui-luma resizable_panels`

**Manual:** Theme Studio main split; gallery pane “Resizable Panels”.
