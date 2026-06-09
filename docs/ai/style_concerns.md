# `style.toml` Migration Concerns & Rust Retention Boundary

> [!NOTE]
> **Status:** Architecture Design Document  
> **Related:** [`style.md`](style.md), [`style.toml`](style.toml)  
> **Scope:** What `style.toml` replaces vs. what remains in `crates/look-shadcn/src/controls/*`

---

## Summary

`style.toml` plus shared loader/resolver code will **significantly simplify** control appearance processing by eliminating duplicated rule definitions (`declare_look_table!` matrices, parallel `_from_palette` / `_from_catalog` paths, per-control color structs).

The goal is **zero duplicated rule definitions**, not zero Rust in appearance functions. Control files become thin shells whose job is to translate state, invoke the stylesheet, and hand resolved values to GPUI paint APIs.

---

## The Boundary

```text
style.toml owns:     WHAT token / metric applies WHEN (selector → value)
Rust retains:        HOW the control interprets SDK state into selector keys,
                     constructs GPUI specs, and applies runtime layout math
```

This boundary is **fundamentally sound**. It preserves look-agnosticism by preventing Rust enum types (`ButtonFamilyRole`, `TextFieldState`) from leaking into the TOML parser. Rust acts as the translator; the stylesheet engine stays type-agnostic.

### What moves out of `controls/*`

| Today (per control) | After migration |
|---|---|
| Markdown token-mapping docs at file top | Data lives in `style.toml` |
| Per-control `*ColorTable` structs + `fallback()` | Generic resolved-field bag or shared types |
| `declare_look_table!` matrices (often 20–80 lines) | `[[component.color_rules]]` in TOML |
| Imperative `_from_palette` re-implementations | Removed (single resolution path) |
| Style tweaks requiring recompile | Edit TOML; reload at runtime (Theme Studio) |

### Target pipeline (thin shell)

```text
state.to_selector_map()
  → stylesheet.query(component, selectors) → resolved fields
  → build AdornerSpec / palette from resolved values
  → compose box model  →  SDK palette struct
```

---

## Concern Categories

Each concern below covers: **current state**, **evaluation**, and **forward-looking recommendation**.

---

### 1. Input Translation (Control Semantics → Selectors)

**Current state:** Each control manually maps runtime state to simple selector keys before querying the stylesheet.

Examples:

- **Button toggle** — unselected toggles query `style = "outline"`; `selected` extracted from `ButtonFamilyRole::Toggle`.
- **Switch** — passes `on: bool` and `disabled`, not `InteractionLayer` (shadcn Switch has no hover surface).
- **Text field** — passes `enabled`, `invalid`, `focus_visible` as separate axes.
- **Navigation sidebar** — routes to `container`, `branch`, or `item` sub-schemas.

**Evaluation:** **Correct.** The stylesheet parser must not know about SDK control APIs.

**Recommendation:** Standardize translation via a shared trait instead of bespoke glue per control:

```rust
pub trait AsSelectorState {
    fn to_selector_map(&self) -> SelectorMap;
}

// Uniform query path:
stylesheet.query(component_name, state.to_selector_map())
```

Each control implements `AsSelectorState` once; appearance functions stop repeating manual string/key assembly.

**Stays in Rust permanently:** Mapping SDK types to selector keys. The stylesheet never sees `ButtonFamilyRole` directly.

---

### 2. Derived Fields (Completing the Palette)

**Current state:** When a color rule omits a field, Rust applies hardcoded geometric defaults:

| Control | Current Rust fallback |
|---|---|
| Button | Ghost → transparent border; primary/secondary/outline → border = background |
| Checkbox | Checked + enabled → `indicator_border` = `indicator_background`; else `border` token |
| Switch | On + enabled → `track_border` = `track_background`; thumb border mirrors thumb fill |

**Evaluation:** **Partially short-sighted.** Hardcoded derivations limit theme flexibility. A theme designer may want a primary button with a contrasting accent border, or a checked checkbox with a transparent indicator border.

**Recommendation:** Support **field references** inside `style.toml` so themes override derivations explicitly. Rust provides safe layout fallbacks only when a field is omitted entirely.

```toml
[[button.color_rules]]
style = "primary"
layer = "default"
background = "primary"
foreground = "primary-foreground"
border = "@background"   # explicit: border follows background

[[button.color_rules]]
style = "primary"
layer = "default"
background = "primary"
foreground = "primary-foreground"
border = "ring"            # explicit override: contrasting border
```

```toml
[[switch.color_rules]]
on = true
disabled = false
track_background = "@action_default"
track_border = "@track_background"
thumb_background = "@action_foreground"
thumb_border = "@thumb_background"
```

**Resolution order:**

1. Rule specifies a token or `@field` reference → resolve and use it.
2. Rule omits the field → Rust applies a documented safe default (e.g. transparent for ghost borders).
3. Never hardcode theme-specific choices in Rust when the stylesheet can express them.

**Stays in Rust:** Fallback logic for omitted fields; conditional gating that depends on control semantics (e.g. checkbox border derivation only when `checked && !disabled`).

---

### 3. Focus Rings and Adorners

**Current state:** Focus geometry is hardcoded in `focus.rs` and per-control appearance functions. Ghost buttons use inset placement; others use oversize with `border_width + focus.width`.

**Evaluation:** **Needs separation.** Painting focus rings is GPUI-specific, but **visual styling** (thickness, distance, color, placement) is theme-dependent. A retro-pixel theme may want a thick square outer border; a material theme may want a thin inset ring.

**Recommendation:** Move focus **parameters** to `style.toml`; Rust still constructs `AdornerSpec` from resolved values.

```toml
[button.focus.default]
placement = "oversize"
thickness = "metrics.focus.width"
distance = "calc(metrics.border_width.default + metrics.focus.width)"
color = "ring"

[button.focus.ghost]
placement = "inset"
thickness = "metrics.focus.width"
distance = "metrics.border_width.default"
color = "ring"

[navigation_sidebar.item.focus]
color = "first(sidebar-ring, ring)"
```

**Stays in Rust:**

- Constructing `AdornerSpec::FocusRing(FocusRingAdornerSpec { ... })`
- Behavioral gating (`focus_visible` vs `focused`, `enabled && focus_visible` for text fields)
- Attaching adorners to palette structs

---

### 4. Layout Composition (Box Model)

**Current state:** Role-based metric overrides live in `compose_button_family_appearance` — icon buttons get zero padding and pill radius regardless of TOML metrics.

**Evaluation:** **Correct that composition stays in Rust**, but override rules should be formalized in the stylesheet rather than hardcoded in the SDK compositor.

**Recommendation:** Model role overrides in metrics blocks:

```toml
[button.metrics.md]
height = "metrics.control.md"
padding_horizontal = 16.0
corner_radius = "radius"
font_size = 14.0

[button.metrics.md.icon_override]
padding_horizontal = 0.0
padding_vertical = 0.0
aspect_ratio = "square"
corner_radius = "radius.pill"
```

Rust reads base metrics + applicable override (matched by role selector), then passes the merged scale to `compose_button_family_appearance`. The compositor becomes a pure geometry merge, not a policy holder.

Navigation sidebar layout constants (`height`, `padding_x`, `icon_size`) migrate the same way — into `[navigation_sidebar.item.metrics]` rather than literals in Rust.

**Stays in Rust permanently:**

- `StandardBoxScale::compute` and scale-factor application
- Pixel snapping and high-DPI alignment
- Typography baseline offset math (font size + line height + control height)
- `compose_button_family_appearance` as geometry merge (inputs come from TOML, logic stays generic)

See [`style.md` §4](style.md#4-the-complexity-of-metrics-the-box-model-seams) for `calc()`, rem conversion, and typographic centering seams.

---

### 5. Shadows, Typography, and Unmodeled Properties

**Current state:** Shadows and typography slots are hardcoded in Rust.

**Shadows** — `elevation.rs` maps theme mode to SDK `LumaElevation` presets:

```rust
pub(crate) fn menu_shadow(theme_mode: ThemeMode) -> Vec<BoxShadow> {
    match theme_mode {
        ThemeMode::Light => LumaElevation::light().menu.to_box_shadows(),
        ThemeMode::Dark => LumaElevation::dark().menu.to_box_shadows(),
    }
}
```

A look skin (e.g. *Retro Arcade*) cannot remove menu shadows or customize blur/offset without code changes.

**Typography** — controls lock to scaffold slots and families:

```rust
typography: typography.text.label,
font_family: typography.font.sans.family.clone().into(),
```

A theme cannot switch button labels to a serif family or a caption-weight slot without code changes.

**Evaluation:** **Both should move to TOML.** Shadows and typography are theme decisions, not control behavior.

**Recommendation — shadows:** Resolve via `ResolveValue<Vec<BoxShadow>>` from CSS catalog tokens or TOML shadow definitions:

```toml
[[floating_menu.surface.color_rules]]
background = "first(popover,card)"
shadow = "shadow-lg"    # resolves --shadow-lg from CSS catalog
```

Or explicit multi-layer definitions when the catalog lacks them:

```toml
[tokens.shadows]
shadow-menu = ["rgba(0,0,0,0.08)", 0.0, 4.0, 12.0, 0.0]
```

**Recommendation — typography:** Metrics blocks declare slot and family:

```toml
[button.metrics.md]
font_slot = "label"        # maps to LumaTextStyle slot in typography catalog
font_family = "font-sans"  # CSS token reference
font_size = 14.0
font_weight = "medium"
```

```rust
let slot = metrics.font_slot.as_deref().unwrap_or("label");
let base = ctx.typography().get_slot(slot);
ButtonFamilyPalette {
    typography: base.with_size(metrics.font_size).with_weight(metrics.font_weight),
    font_family: ctx.typography().resolve_family(&metrics.font_family),
    // ...
}
```

**Stays in Rust permanently:**

- `ResolveValue<T>` implementations (color, f32, shadow, typography)
- Runtime pixel snapping and baseline centering math
- GPUI canvas/shadow application at paint time

---

### 6. Dual Resolution Paths (Temporary)

**Current state:** Many controls maintain parallel `*_from_palette` and `*_from_catalog` implementations that re-implement the same rules.

**Evaluation:** **Migration artifact, not architecture.** Removed when TOML loader lands.

---

## Retention Summary

| Concern | Initial migration | Target (forward-looking) |
|---|---|---|
| Selector remapping | Manual per control | `AsSelectorState` trait |
| Border / field fallbacks | Hardcoded in Rust | `@field` references in TOML; Rust safe defaults when omitted |
| Focus adorners | Hardcoded geometry | Focus params in TOML; Rust builds `AdornerSpec` |
| Icon / role metric overrides | Hardcoded in compositor | `[metrics.*.role_override]` in TOML |
| Shadows | Hardcoded in `elevation.rs` | `ResolveValue<Vec<BoxShadow>>` from catalog/TOML |
| Typography slots | Hardcoded scaffold picks | `font_slot`, `font_family`, `font_weight` in metrics |
| Pixel snapping / baseline | Rust runtime | Rust runtime (permanent) |
| SDK type → selector translation | Rust | Rust (permanent) |
| GPUI spec construction | Rust | Rust (permanent) |
| Control interaction model | Rust | Rust (permanent) |

### Per-control sketch (target state)

| Control | Rust retains |
|---|---|
| Button | `AsSelectorState` for toggle/icon; focus gating; compositor call |
| Checkbox | Selector translation; conditional border gating when field omitted |
| Switch | Selector translation (`on`, not layer); thumb shadow query |
| Text field | `focus_visible` gating; invalid/enabled selector axes |
| Navigation sidebar | Sub-part routing; merged metrics + focus params from TOML |
| Split view | Near-pure query — reference thin shell |
| Tabs navigation | Selector axes (`active`, `focused`); indicator from TOML rules |

---

## What Disappears vs. What Gets Thinner

```text
TODAY (typical control file):
  docs + ColorTable struct + fallback()
  + declare_look_table! { 20–80 lines }
  + _from_palette { re-implements rules imperatively }
  + _from_catalog { query table + assemble }
  + hardcoded focus/shadow/typography/border derivations

TARGET (thin shell):
  impl AsSelectorState { ... }
  + stylesheet.query(component, selectors)
  + build palette from resolved fields (minimal fallbacks)
  + attach adorners from resolved focus block
```

The **large deletion** is duplicated rule data and hardcoded theme choices. The **persistent shell** is state translation, GPUI spec construction, and runtime layout math.

---

## Non-Goals

The migration does **not** aim to:

- Move control **behavior** (events, focus traversal, keyboard handling) into TOML
- Replace SDK palette/appearance **types** with dynamic maps
- Remove the inspect/provenance path — provenance attaches to the shared resolver and cites matching TOML rules
- Eliminate `LookResolver` token syntax — it moves into the shared engine (`input/50`, `first(...)`, `@action_layer`, `@field`)

---

## Success Criteria

The migration is successful when:

1. Every color/metric rule in `declare_look_table!` has a corresponding entry in `style.toml` (already largely true — see [`style.toml`](style.toml)).
2. No control file contains an imperative re-implementation of the same rules in `_from_palette`.
3. Style changes for mapped properties require editing `style.toml`, not touching 25 Rust files.
4. Remaining Rust in each `controls/*.rs` appearance function is a **thin shell** — state translation, spec construction, runtime math — readable in one screenful.
5. Gallery/Theme Studio inspectors cite the matching TOML rule (rule index or selector) as provenance.
6. Theme designers can override borders, focus geometry, shadows, typography, and role metrics without Rust changes.

---

## See Also

- [`style.md`](style.md) — 3-tier architecture, schema proposal, button bootstrap spec
- [`style.toml`](style.toml) — unified mapping configuration (18 control families)
- [`architecture.md`](architecture.md) — SDK vs look-shadcn layering
- `crates/look-shadcn/src/provenance.rs` — resolved color/metric provenance for inspect path
- `crates/look-shadcn/src/focus.rs` — shared focus adorner helpers (params migrate to TOML)
- `crates/look-shadcn/src/elevation.rs` — hardcoded shadows (migrate to TOML/catalog resolution)
