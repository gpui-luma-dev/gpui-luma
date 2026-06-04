# Architecture Review: Semantic Density and Structural Scaling

This document details the architectural design for introducing contextual density scaling and math-driven structural layout metrics across the GPUI control suite, keeping visual themes and spatial layout formulas strictly decoupled.

---

## 1. Rationale

The current SDK architecture relies on a decoupled, three-layer pattern (Control $\rightarrow$ Template $\rightarrow$ Theme). However, the theme-to-template interface resolves flat, primitive value models eagerly (returning raw `f32` and `Hsla` values). 

This design leads to two core issues:
1. **Dumb Layouts**: Micro-metrics—such as checkbox checkmark stroke widths, internal paddings, and switch thumb ratios—cannot dynamically adapt to varying spatial or density contexts (`Compact`, `Normal`, `Large`).
2. **Merged Concerns**: Component appearance structs combine layout dimensions and color palettes in the same theme-resolved contract. This forces theme implementations (like `RadixTheme`) to duplicate layout calculations (e.g., width/height scaling), making it difficult to write richer custom themes without duplicating structural scaling logic.

**Interim state (switch).** The codebase currently resolves geometry inside `SwitchAppearance` via `switch_track_metrics` in both `controls/switch/theme.rs` and `theme/radix/switch.rs`. Gallery size variants (`Sm` / `Md` / `Lg`) work, but layout math runs during theme resolution and is duplicated across theme backends. The target architecture below removes geometry from theme traits entirely.

To solve this, the architecture separates **Structural Layout Scales** (managed by the SDK layout rules) from **Visual Theme Palettes** (managed by exchangeable themes).

---

## 2. Solutions

### A. Strict Decoupling of Layout and Style
Rather than resolving a merged struct from a theme trait, the Template combines two distinct inputs:
1. **Sizing/Geometry Scale (`ElementScale`)**: A per-control layout configuration (e.g. `SwitchScale`, `CheckboxScale`) resolved directly by the SDK template from the active `ControlSize`, structural layout formulas, and—where needed—vector stroke/inset constraints.
2. **Visual Palette (`ElementPalette`)**: A state-dependent set of coloring, shading, borders, and shadows resolved from the active theme trait (e.g. `SwitchPalette`, `CheckboxPalette`).

```
┌────────────────────────────────────────────────────────┐
│                     Control Layer                      │
│     (State Management, Event Handlers, Data Model)     │
└───────────────────────────┬────────────────────────────┘
                            │ Passes Size/InteractionState
                            ▼
┌────────────────────────────────────────────────────────┐
│                   Template Layer                       │
│  1. Computes layout geometry via ElementScale (SDK)    │
│  2. Queries visual look via theme palette (Theme)      │
│  3. Renders elements combining both contracts          │
└────────────────────────────────────────────────────────┘
```

### B. Math-Driven Scale Formulas
Rather than hardcoding sizing tokens (e.g., setting a thumb size manually to `14.0`), layouts compute scales dynamically based on the control height and container mass. This guarantees proportional spacing and stroke weights across different density levels.

Box controls (Switch, Button) express everything as ratios of `control_height`. Vector-bearing controls (Checkbox, Accordion, Tree) add explicit `icon_stroke_width` and `icon_inset` fields—see section 4.

### C. Coordinated Typography Scale
To avoid baseline drift when scaling text mathematically, the SDK layout engine maps each `ControlSize` to a discrete, human-curated typography token:
* **Compact / Sm**: Maps to `typography.text.caption` (tight, metadata-weight).
* **Normal / Md**: Maps to `typography.text.label` (standard interface-weight).
* **Large / Lg**: Maps to `typography.text.body` (prominent content-weight).

### D. Fallback Matrix Processing
Because not all controls support every density level (e.g., a Slider does not have a meaningful `Compact` scale), the engine uses a trait-based resolution matrix to fall back to the nearest supported variant (e.g., falling back to `Normal` if `Compact` is requested but not implemented).

### E. Pixel Snapping for Crisp Layout
Math-driven scale formulas produce fractional logical bounds (e.g. `ControlSize::Sm` with `control_height = 28` yields track height ≈ `17.1px`, thumb ≈ `14px`). On GPU-backed canvases and especially at 2× display scale, fractional dimensions cause anti-aliasing blur on tracks, thumbs, and borders.

Snap **layout dimensions** to physical pixel boundaries before passing them to GPUI `px()` helpers. Do not snap typography sizes, colors, or theme tokens. Hairline borders (`border_width: 1.0`) may need separate rules later (snapping a 0.5 logical border to one physical pixel is a different concern than snapping control height).

Shared utility (SDK-owned, reused by checkbox/radio/button-family scales):

```rust
// theme/layout.rs (or controls/layout.rs)

/// Rounds a logical length to the nearest physical pixel at the given display scale.
pub fn snap_to_pixel(value: f32, scale_factor: f32) -> f32 {
    (value * scale_factor).round() / scale_factor
}
```

`scale_factor` comes from `window.scale_factor()` at template render time—not from `MetricTokens`. GPUI already exposes display scale on `Window`; conflating it with theme metrics would tie layout to the wrong abstraction.

### F. Cached Layout Resolution
Template `render` runs whenever GPUI rebuilds the element tree—not on every paint frame—but layout math still repeats unnecessarily today:

- **Focused-probe paths** resolve appearance twice (default + focused adorner probe) even though geometry is identical.
- **Virtual lists** (`ListView`) and gallery panes render many controls sharing the same `ControlSize`.
- **Merged appearance structs** force themes (including Radix) to recompute `(42/36)` ratios inside color resolution.

`SwitchScale::compute` is pure in `(ControlSize, MetricTokens, scale_factor)` and does not depend on hover, pressed, or on/off state. Cache it.

To prevent recalculations, layout metrics are computed once per cache key and stored in a global registry on the GPUI application context (`cx`). The cache is invalidated when global `ThemeTokens` / `MetricTokens` change (theme swap, CSS reload)—not on interaction state changes.

**Cache key.** `(ControlSize, scale_factor, TypeId)` per control scale type (e.g. `SwitchScale`). Include `scale_factor` because pixel snapping (section 2E) makes layout depend on the active display scale. Do **not** put `scale_factor` on `MetricTokens`; metrics are theme-level, scale factor is viewport/device state from `Window`.

**Interim fallback.** Before `use_cached_layout` lands, a static `OnceLock<[SwitchScale; 3]>` keyed by theme epoch is acceptable for switch-only work. The context hook is the long-term shape shared by checkbox, radio, and button-family scales.

---

## 3. Reference Implementation: Switch (Box Layout)

Below is a reference model for **box-layout** controls—Switch first—demonstrating how to separate geometry from stateful visual rendering.

### Decoupled Switch Layout Scale (SDK-owned)
```rust
// controls/switch/layout.rs

/// Composite layout metrics for a Switch control, resolved independently of visual themes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchScale {
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
    pub label_baseline_shift: f32, // Visual alignment nudge for typography
}

impl SwitchScale {
    /// Computes contextual layout proportions based on MetricTokens and display scale.
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let control_height = metrics.control_height(size);

        Self {
            width: snap_to_pixel(control_height * (42.0 / 36.0), scale_factor),
            height: snap_to_pixel(control_height * (22.0 / 36.0), scale_factor),
            thumb_size: snap_to_pixel(control_height * 0.5, scale_factor),
            padding: snap_to_pixel((control_height * (2.0 / 36.0)).max(1.0), scale_factor),
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            radius: metrics.radius.pill, // pill radius may stay fractional
            label_baseline_shift: match size {
                ControlSize::Sm => 0.5,
                ControlSize::Md => 1.0,
                ControlSize::Lg => 1.5,
            },
        }
    }
}
```

Ratios `(42/36)`, `(22/36)`, and `0.5` are anchored to the `Md` reference height (`36px`). They replace hardcoded literals like `width: 42.0, height: 22.0` currently duplicated in default and Radix switch themes.

### GPUI Layout Cache Extension
A global cache registry attached to the active `ThemeTokens` in GPUI handles storing and invalidating calculations:

```rust
// theme/cache.rs

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutCacheKey {
    pub size: ControlSize,
    pub scale_factor_bits: u32, // f32::to_bits(scale_factor)
}

pub trait LumaLayoutCacheExt {
    /// Fetches a cached layout scale, computing it only on a cache miss.
    fn use_cached_layout<S, F>(&self, key: LayoutCacheKey, compute: F) -> S
    where
        S: Clone + Send + Sync + 'static,
        F: FnOnce(&MetricTokens) -> S;
}

// Inside the GPUI global state setup:
// LayoutCache holds a TypeMap keyed by (LayoutCacheKey, TypeId).
// When ThemeTokens are set/updated, LayoutCache is cleared.
```

Do **not** compute layout on every render pass:

```rust
// Instead of recomputing inside theme.resolve or inline in the template:
let scale = cx.use_cached_layout(
    LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
    |metrics| SwitchScale::compute(model.size, metrics, scale_factor),
);
```

### Visual Palette and Theme Contract (Theme-owned)
```rust
// controls/switch/theme.rs

pub trait SwitchTheme: Send + Sync {
    /// Resolves the visual styling of a Switch, independent of spatial sizing
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchPalette;
}

#[derive(Clone, Debug)]
pub struct SwitchPalette {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub label_font_family: SharedString,
}
```

### Template Render Implementation (with Cache Hook)
```rust
// controls/switch/template.rs

impl ButtonTemplate<bool> for ThemedSwitchTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let scale_factor = window.scale_factor();

        // 1. Resolve visual styling from theme (SwitchTheme) — no size parameter
        let palette = self.theme.resolve(model.data, model.state);

        // 2. Resolve structural geometry via cache (pure in size + metrics + scale_factor)
        let scale = cx.use_cached_layout(
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| SwitchScale::compute(model.size, metrics, scale_factor),
        );

        // 3. Compute dynamic offsets using sizing scale (snap offsets derived from snapped dims)
        let thumb_left = if model.data {
            scale.width - scale.thumb_size - scale.padding
        } else {
            scale.padding
        };
        let thumb_top = snap_to_pixel(((scale.height - scale.thumb_size) * 0.5 - 1.0).max(0.0), scale_factor);

        let thumb = div()
            .id(format!("{}-thumb", model.id))
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .size(px(scale.thumb_size))
            .bg(palette.thumb_background)
            .border_1()
            .border_color(palette.thumb_border)
            .rounded(px(scale.radius))
            .shadow(palette.thumb_shadow.clone());

        let track_visual = div()
            .id(format!("{}-track", model.id))
            .relative()
            .w(px(scale.width))
            .h(px(scale.height))
            .bg(palette.track_background)
            .border_1()
            .border_color(palette.track_border)
            .rounded(px(scale.radius))
            .child(thumb);

        let mut track = div().relative().child(track_visual);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(palette.adorner, scale.radius) {
            track = track.child(adorner);
        }

        // 4. Assemble with visual line baseline shift
        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .gap(px(scale.gap))
            .text_color(palette.label_color)
            .text_size(px(palette.label_typography.size))
            .line_height(px(palette.label_typography.line_height))
            .font_family(palette.label_font_family.clone())
            .font_weight(palette.label_typography.weight)
            .rounded(px(scale.radius))
            .child(track)
            .child(
                div()
                    // Baseline vertical nudge shifts baseline relative to switch track
                    .mt(px(scale.label_baseline_shift)) 
                    .child((model.content)(model, cx))
            );

        if model.state.disabled {
            root = root.opacity(0.56);
        } else {
            root = root.cursor_pointer();
        }

        self.apply_modifiers(root, model)
    }
}
```

---

## 4. Vector Path Scaling for Complex Controls

For controls like the **Switch**, layout scaling is straightforward: modify widths, heights, padding, and offsets inside a rectangular bounding box. The `SwitchScale` contract is sufficient.

Complex components—**Checkbox checkmarks**, **Accordion chevrons**, **Tree Node expansion icons**, **Radio dots**, progress arcs—require the same scale/palette split but expand the layout contract to describe **vector geometry**, not just box dimensions.

### A. Simple vs Vector Controls

| Category | Examples | Scale contract carries |
|---|---|---|
| **Box layout** | Switch, TextField, Button | width, height, padding, gap, radius |
| **Box + glyph** | Checkbox (today: Lucide font glyph), ListBox checkmark | indicator box **plus** icon size, stroke, inset |
| **Path / chevron** | Accordion, Tree, NavigationSidebar branch | hit target box **plus** path stroke, chevron arm length, rotation pivot inset |
| **Stroke arc** | Progress ring, Slider thumb ring | radius, stroke_width, cap inset |

Themes must never own these numbers. Themes supply colors; SDK `*Scale::compute` supplies geometry.

### B. Treat Vectors as Proportions of the Bounding Box

When scaling complex vectors, define shapes relative to the computed indicator or icon box—not as absolute pixel art at one density.

1. **Bounding box first** — resolve `indicator_size`, `height`, `gap` from `control_height(size)` (same as box controls).
2. **Insets second** — shrink the drawable region inside the box (`icon_inset`) so strokes do not clip against borders.
3. **Stroke third** — set explicit `icon_stroke_width` (or `path_stroke_width`) per `ControlSize`; snap to physical pixels via `snap_to_pixel`.
4. **Path coordinates last** — express SVG/path points as fractions of the inset rect (0.0–1.0), then multiply at render time. Do not bake absolute path data per size tier.

**Interim state (checkbox).** The template today renders a Lucide `Check` font glyph sized by `appearance.checkmark_size`. The target model replaces implicit glyph sizing with an explicit `CheckboxScale` that carries `icon_stroke_width` and `icon_inset`, whether the checkmark is eventually drawn as a stroked path, Lucide icon, or GPUI `PathBuilder`.

### C. Reference: `CheckboxScale`

`ElementScale` structures for vector-bearing controls must include explicit floating-point stroke and inset fields—not just outer dimensions:

```rust
// controls/checkbox/layout.rs

/// Layout metrics for a Checkbox indicator and its checkmark vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxScale {
    /// Outer indicator box (width and height).
    pub indicator_size: f32,
    /// Corner radius of the indicator box.
    pub indicator_radius: f32,
    /// Row height aligning indicator with label.
    pub height: f32,
    /// Gap between indicator and label.
    pub gap: f32,
    /// Label baseline nudge (see section 5C).
    pub label_baseline_shift: f32,
    /// Drawable checkmark size inside the indicator (after inset).
    pub checkmark_size: f32,
    /// Padding between indicator border and checkmark path.
    pub icon_inset: f32,
    /// Stroke width for path-rendered checkmarks (hairline-aware).
    pub icon_stroke_width: f32,
}

impl CheckboxScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let h = metrics.control_height(size);

        // Per-size tables anchor stroke/inset to human-tuned values at Md,
        // while indicator proportions stay math-driven from control_height.
        let (indicator_ratio, indicator_radius, icon_stroke_width, icon_inset) = match size {
            ControlSize::Sm => (0.45, 2.0, 1.0, 2.0),
            ControlSize::Md => (0.50, 4.0, 1.5, 3.0),
            ControlSize::Lg => (0.55, 6.0, 2.0, 4.0),
        };

        let indicator_size = snap_to_pixel(h * indicator_ratio, scale_factor);
        let icon_inset = snap_to_pixel(icon_inset, scale_factor);
        let checkmark_size = snap_to_pixel((indicator_size - icon_inset * 2.0).max(0.0), scale_factor);

        Self {
            indicator_size,
            indicator_radius: snap_to_pixel(indicator_radius, scale_factor),
            height: snap_to_pixel(h, scale_factor),
            gap: snap_to_pixel(metrics.gap(size), scale_factor),
            label_baseline_shift: match size {
                ControlSize::Sm => 0.5,
                ControlSize::Md => 1.0,
                ControlSize::Lg => 1.5,
            },
            checkmark_size,
            icon_inset,
            icon_stroke_width: snap_to_pixel(icon_stroke_width, scale_factor),
        }
    }
}
```

Per-size `match` arms are appropriate when stroke weights and corner radii need discrete tuning (avoid sub-1px strokes at `Sm`). Indicator **proportions** remain formula-driven from `control_height`; only stroke/inset/radius tiers are tabulated.

### D. Template Usage (Checkmark)

```rust
// controls/checkbox/template.rs (target)

let scale = cx.use_cached_layout(
    LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
    |metrics| CheckboxScale::compute(model.size, metrics, scale_factor),
);
let palette = self.theme.resolve(model.data, model.state);

let indicator = div()
    .size(px(scale.indicator_size))
    .rounded(px(scale.indicator_radius))
    .bg(palette.indicator_background)
    .border_1()
    .border_color(palette.indicator_border)
    .child(render_checkmark_path(
        model.data,
        scale.checkmark_size,
        scale.icon_inset,
        scale.icon_stroke_width,
        palette.checkmark_color,
    ));
```

`render_checkmark_path` draws inside the inset rect. For a Lucide fallback, map `checkmark_size` to font size and treat `icon_stroke_width` as implicit (font weight), but keep the scale fields so a path renderer can drop in without theme changes.

### E. Other Vector Scale Sketches

Same pattern extends to future controls without merging into one mega-struct:

```rust
// Accordion / collapsible section chevron
pub struct ChevronScale {
    pub hit_size: f32,           // square touch target
    pub arm_length: f32,         // chevron leg length
    pub stroke_width: f32,
    pub inset: f32,              // padding inside hit box
}

// Tree node expand/collapse
pub struct TreeDisclosureScale {
    pub hit_size: f32,
    pub stroke_width: f32,
    pub inset: f32,
    pub branch_gap: f32,         // space before node label
}
```

Each implements `compute(size, metrics, scale_factor)` and caches via `use_cached_layout`. Path templates read normalized coordinates `(inset + t * drawable)` where `drawable = hit_size - 2 * inset`.

### F. Stroke Snapping Rules

| Value | Rule |
|---|---|
| `icon_stroke_width` | Snap with `snap_to_pixel`; enforce minimum 1 physical pixel at `Sm` |
| `icon_inset` | Snap; must leave `checkmark_size > 0` |
| Path points | Compute from snapped box/inset, not pre-snapped normalized coords |
| Font glyphs | Use `checkmark_size` only; stroke field reserved for path migration |

---

## 5. Potential Issues & Resolutions

### A. Eager Layout Re-Evaluation (Performance)
* **Potential Issue**: If layout is resolved inside `theme.resolve` on every element rebuild—or twice per control for focused-probe adorner paths—GPUI runs the same math formulas and rebuilds layout objects unnecessarily. Today, merged `SwitchAppearance` structs and duplicated Radix `switch_track_metrics` calls exhibit this pattern.
* **Resolution**: Keep layout calculation entirely separate from theme color resolution. Templates evaluate cached geometry (`SwitchScale`) via `use_cached_layout`, keyed by `(ControlSize, scale_factor, TypeId)`. Interaction state (hover/press/focus) affects only `SwitchPalette`, not scale.

### B. Subpixel Rendering & Blur (Visuals)
* **Potential Issue**: Math-driven calculations often produce fractional pixel bounds (e.g. track height `17.1px` at `Sm`). In a GPU-accelerated canvas like GPUI, this causes anti-aliasing artifacts—blurry tracks and thumbs, especially on 2× displays.
* **Resolution**: Apply `snap_to_pixel(value, window.scale_factor())` to layout dimensions inside `SwitchScale::compute` before they reach `px()`. Source scale factor from `Window`, not `MetricTokens`. See section 2E.

| Snap | Do not snap |
|---|---|
| width, height, thumb_size, padding, gap | typography sizes, colors, shadows |
| derived offsets (thumb_left, thumb_top) | pill radius (may stay fractional) |
| | hairline border widths (separate rules TBD) |

### C. Typography Baseline & Cap-Height Misalignment (Alignment)
* **Potential Issue**: Centering labels next to control indicators via flexbox alignment (`items_center`) aligns with the center of the line box. Because of character ascenders and descenders (e.g., `g`, `p`, `y`), text visually appears sagged or off-center relative to geometric indicators.
* **Resolution**:
  1. **Tight Line-Heights**: Enforce line-heights matching the font size closely (e.g., font size `13px` with line-height `14px` or `16px`) for inline control labels to eliminate outer text padding.
  2. **Baseline Shift parameter**: Implement a `label_baseline_shift` nudge parameter in the scale contracts. Use it inside the template layer to nudge the label container.

### D. Vector Stroke Clipping at Small Sizes
* **Potential Issue**: At `ControlSize::Sm`, a fixed `icon_inset` plus `border_1` can leave zero or negative drawable area for path checkmarks and chevrons. Strokes drawn at sub-1px widths look faint or disappear on 1× displays.
* **Resolution**: Compute `checkmark_size = (indicator_size - 2 * icon_inset).max(0.0)` in `CheckboxScale::compute` and clamp `icon_stroke_width` to at least one physical pixel after snapping. Prefer per-size stroke/inset tables (section 4C) over pure ratios when discrete tiers produce better craft.

---

## 6. Suggested Rollout

Implement in small, reviewable steps:

1. **`SwitchScale` + `snap_to_pixel`**: Add `controls/switch/layout.rs` and shared `snap_to_pixel`. Refactor switch template to compute scale from `window.scale_factor()`. Strip width/height/thumb/padding/gap from `SwitchAppearance`; introduce `SwitchPalette` on the theme trait.
2. **Radix cleanup**: Delete duplicated `switch_track_metrics` from `theme/radix/switch.rs`; Radix themes resolve colors only.
3. **`use_cached_layout`**: Add `theme/cache.rs` with `LayoutCacheKey` and invalidation on theme swap. Wire switch template to the cache hook.
4. **`CheckboxScale` + vector fields**: Introduce `controls/checkbox/layout.rs` with `icon_stroke_width` and `icon_inset`. Strip indicator/checkmark dimensions from `CheckboxAppearance`. Migrate checkmark rendering to consume scale; keep Lucide glyph as interim backend.
5. **Radio + path controls**: Apply `RadioScale` (dot diameter, ring width). Add `ChevronScale` / `TreeDisclosureScale` when accordion and tree controls land.
6. **Generalize**: Reuse `snap_to_pixel`, cache infrastructure, and scale/palette split across remaining controls.

Gallery size-variant panes (button, icon button, toggle, checkbox, switch, radio) serve as the visual regression surface for steps 1–6.
