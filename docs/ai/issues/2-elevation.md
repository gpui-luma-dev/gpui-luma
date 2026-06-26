# Issue #2: Elevation and Box-Shadow Integration

## Description
While the Shadcn CSS theme catalogs (e.g., `jarvis.css`, `retro-arcade.css`) define a complete typographic and visual spacing system, they also export custom box-shadow custom properties (`--shadow-xs` through `--shadow-2xl`). 

Currently, only a few surface elements (like `card`, `selector_items_panel`, and `floating_menu`) support shadows, and they do so by hardcoding the lookup category (e.g., `ShadcnShadow::Default` or native fallbacks). Standard interactive controls in the SDK—such as buttons, switches, and textfields—lack first-class, dynamic shadow mapping. In addition, rendering shadows inside clipped containers in GPUI requires calculating outer bounds offsets to prevent visual edge clipping.

To solve this, we will model elevation and box-shadow styling as a first-class feature across the SDK layout system and look-shadcn stylesheet rule-matcher, drawing on the successful proof-of-concept prototype `shadow_button`.

**Input elevation policy:** Textfield/textarea elevation ships on the **Filled** variant only. **Surface** (transparent light-mode fill) and **Soft** (muted, borderless) do not get elevation rules — see §4 Transparent-Fill Seam.

---

## Proposed Solution

### 1. Unified Elevation Model in the SDK
We will integrate shadow support into the core look schemas of SDK controls, upgrading existing shadow-using controls first and integrating shadows into the base button template:
* **Extend Control Look**: Add `shadow: Option<Vec<BoxShadow>>` to control look definitions (such as `ButtonFamilyLook` and `TextFieldLook`).
* **Update Control Templates**: Modify template render paths (like `ButtonTemplate`) to draw the resolved shadows.
* **Upgrade Existing Shadow Controls**: Update existing controls currently drawing hardcoded or manual shadows to resolve them dynamically via look-shadcn catalog metric rules and stylesheet configurations (`style.toml`). These include:
  * **Floating/Popup surfaces**: `floating_menu`, `combobox`, `selector_panel`, `search_selector`, and `autocomplete`.
  * **Containers**: `card` and `selection_panel`.
  * **Control thumbs**: `switch` and `slider` (updating thumb shadow resolutions).
* **Integrate into Button Base**: Add shadow capabilities directly to the actual `button` base templates and models inside `look-shadcn` and the SDK. This will automatically propagate shadow rendering support to all derived interactive controls (`button`, `toggle`, `checkbox`, `radio_button`).

### 2. Stylesheet Rule Resolution
We will extend look-shadcn's stylesheet rule-matcher (`style.toml` parser) to resolve shadow tokens dynamically:
* **Parse TOML Shadow properties**: In [stylesheet/config.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/stylesheet/config.rs), ensure that the metrics and styling rules parse the `shadow` property (e.g. `shadow = "shadow-sm"` or `shadow = "none"`).
* **Map to Resolved Metrics**: Include shadows in the resolved metrics configurations (like `ResolvedButtonMetrics`) by calling `look.shadow(ShadcnShadow)` on the parsed token key.

### 2b. Textfield Elevation (Filled Variant Only) — **Shipped**

First input-family elevation. Surface/Soft are explicitly excluded (§4).

| Piece | Location | Notes |
|-------|----------|-------|
| Look field | `TextFieldLook.shadow`, `TextFieldPalette.shadow` | SDK `textfield/theme.rs` |
| Template paint | `#*-control` `.shadow(...)` when enabled | `textfield/template.rs` — textfield live entity uses template |
| Stylesheet parser | `TextfieldElevationRule`, `find_textfield_elevation_rule` | `stylesheet/config.rs`, `stylesheet/mod.rs` |
| TOML rules | `[[textfield.elevation_rules]]` | `filled` → `shadow-xs`; `surface` / `soft` → `none` |
| Color + fill | `ShadcnTextFieldStyle::Filled` | `background` opaque fill |
| Resolution | `textfield_elevation_shadow()` | `look-shadcn/controls/textfield.rs` — skipped when disabled |
| Builder API | `.filled(&look)` | `ShadcnTextFieldExt` |

**Textarea:** Shares textfield tokens via `textarea_palette()` → same elevation rules apply to **Filled** only. Live textarea paints shadow in `textarea/control.rs` (custom render path, not template).

**Usage:**
```rust
look.textfield("email").filled(&look).placeholder("…").spawn(cx);
```

**Do not** add elevation rules for Surface or Soft until §4 compositing is solved or product accepts opaque Surface fill.

### 3. Bounding Projection Padding (No-Clipping Helper)
In GPUI, parent elements that apply `overflow_hidden()` or sit inside tight layout bounds will clip shadows that extend beyond the child's container. 
To resolve this:
* We will move the prototype's `ShadowProjectionInsets` layout math into a shared utility inside the SDK (`crates/sdk/src/theme/layout.rs` or `adorner.rs`):
  ```rust
  #[derive(Clone, Copy, Debug, Default)]
  pub struct ShadowProjectionInsets {
      pub top: f32,
      pub right: f32,
      pub bottom: f32,
      pub left: f32,
  }

  impl ShadowProjectionInsets {
      pub fn compute(offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
          let reach = (blur.max(0.0) + spread).max(0.0);
          Self {
              top: (reach - offset_y).ceil().max(0.0),
              right: (reach + offset_x).ceil().max(0.0),
              bottom: (reach + offset_y).ceil().max(0.0),
              left: (reach - offset_x).ceil().max(0.0),
          }
      }
  }
  ```
* Standard templates will query this helper and automatically apply outer padding/margins to their root wrapper elements to guarantee that shadows render completely without clipping.

### 4. Element Styling Extensions (`ShadcnElementExt`)
To simplify applying parsed shadows to arbitrary layout panels in application screens (like gallery pages or theme-studio panes), we will add a helper method to `ShadcnElementExt` in [ext.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/ext.rs):
```rust
fn shadow_cn(self, role: ShadcnShadow) -> Self {
    with_active_look(|look| {
        if let Some(look) = look {
            self.shadow(look.shadow(role))
        } else {
            self
        }
    })
}
```

## GPUI Rendering Limitations & Blast Minimization Plan

GPUI's vector rendering pipeline has several constraints regarding shadows, clip paths, and subpixel coordinates. Based on lessons from the prototype, we expect the following three critical rendering limitations:

### 1. The Box-Model Seam (Layout Displacement vs. Absolute Drawing)
* **The Problem**: If a control (e.g. a `36px` height button) automatically adds `5px` top and bottom `ShadowProjectionInsets` padding to its root container to prevent shadow clipping, the control's logical layout height increases to `46px`. When used inside flex grids, toolbar button groups, or header lines, this extra layout padding will break vertical alignment and distort spacing.
* **Blast Minimization**:
  * By default, shadows **must** be drawn using absolute-positioned backing elements that extend *outside* the control's bounds (e.g., `.top(px(-inset))`) **without** adding layout padding to the control's root container. This keeps the control's physical bounds flat and maintains standard flex/grid alignments.
  * Make projection padding **opt-in / context-aware**: only apply layout padding (via a wrapper or an explicit `.with_projection_padding(true)` builder flag) when a control is placed inside a container that has clipping enabled.

### 2. The Overflow Seam (Parent Clip-Paths Cutting Off Shadows)
* **The Problem**: If a parent container applies `overflow_hidden()` (common in lists, sliders, scroll views, and cards with rounded corners), GPUI clips all absolute-positioned child elements at the parent's boundary. Backing shadows that extend beyond the child's container will be sliced cleanly at the clipping edge.
* **Blast Minimization**:
  * Disable shadows entirely for controls rendered inside nested scroll items or compact list rows.
  * Provide an SDK helper container or custom wrappers that selectively keep layout containers overflow-visible, separating clipping boundaries from shadow layers where layout allows.

### 3. The Subpixel Rendering Seam (Anti-Aliasing Artifacts & Radian Mismatches)
* **The Problem**: GPUI's box shadow shader renders rounded rectangle shadows. If the corner radius of the absolute-positioned shadow backing container does not *exactly* equal the radius of the front element, or if subpixel float rounding fails, anti-aliased jagged edges or background bleeds will occur.
* **Blast Minimization**:
  * The rendering template **must** enforce that the corner radius applied to the absolute shadow layer scales dynamically with the front-element's radius (e.g., `radius_override` or look radius).
  * Always snap calculated pixel offsets cleanly via `snap_to_pixel(val, scale_factor)` to ensure the shadow backing coordinates align exactly with subpixel boundaries.

### 4. The Transparent-Fill Seam (Surface / Soft — Blocked)

* **The Problem**: shadcn **Surface** inputs use `bg-transparent` (light mode). Applying `shadow-xs` / `shadow-sm` to that chrome does not read as a drop shadow beneath the field — it appears as a dark band at the **top inner edge** of the control interior.
* **Diagnosis**: GPUI paints `box_shadow` **before** the background quad (`style.rs` → `paint_shadows` then conditional `paint_quad`). When the control fill is fully transparent, no opaque quad is drawn afterward, so shadow blur is visible **inside** the element bounds, not only below it. Opaque controls (buttons, checkbox/radio indicators, switch track, **Filled** textfield) do not hit this because the post-shadow background quad occludes interior bleed.
* **What did not help** (Surface elevation attempt — reverted):
  * Applying `ShadowProjectionInsets` padding on the shadow wrapper — especially **top** inset — gave upward blur a visible slot and worsened the top-edge artifact; bottom-only padding did not fix interior bleed.
  * Moving shadow to a focus-ring wrapper or inner `#-control` div — same paint order on the same transparent element.
* **Unblocked path (shipped):** **Filled** variant — opaque `background` fill + `[[textfield.elevation_rules]]` `shadow-xs`. Use `.filled(&look)` where drop shadow is needed; keep **Surface** as default for shadcn Input parity.
* **Still blocked:** Elevation on **Surface** and **Soft** without changing fill or GPUI compositing.
* **Likely fix directions** (not implemented for Surface):
  * Separate opaque backing plate behind transparent chrome; or
  * GPUI-level exterior-only shadow compositing on transparent elements; or
  * Non-transparent Surface fill (deviates from shadcn light-mode Input).

### Addendum: Selection Trigger Chrome (Blocked — outline/ghost templates needed)

* **Context**: Popup menu trigger styling was fixed by introducing `PopupMenuTriggerStyle::Outline` (default) and `Ghost`, each delegating to the same `button_palette()` / `button_elevation_shadow()` path as command buttons. That removed the old hybrid (ghost fill/hover tokens + always-on `border` in the template).
* **Current problem**: The **selection control family** still uses the same incorrect hybrid pattern popup menu had, and several variants compose a **textfield** for the trigger/input chrome:
  * **`selector`**: `resolve_ghost_trigger_colors()` for fill/foreground + hardcoded `border` token — reads as outline but behaves like ghost on hover.
  * **`combobox`**, **`search_selector`**, **`autocomplete`**: textfield **Surface** chrome for the editable or display trigger, plus floating-menu/selector panel for the dropdown. Trigger coloring is textfield-driven, not command-button-driven.
* **Why this blocks elevation work here**: Read-only select triggers should use **command button outline** elevation (`[[button.elevation_rules]]`), not textfield rules. Editable combobox inputs may use **Filled** textfield elevation once product chooses that chrome — not Surface.
* **Required follow-up** (before selector/combobox/search_selector/autocomplete elevation):
  * Add a **trigger style** to selection controls (at minimum **`Outline` default**; optionally **`Ghost`** where product calls for it), mirroring popup menu.
  * Refactor trigger templates so outline/ghost resolve through **command button** tokens (border optional, shadow from button elevation rules, button box metrics) — not `resolve_ghost_trigger_*` and not naked textfield surface rules.
  * For combobox/autocomplete/search_selector, decide explicitly whether the trigger is **button-like** (read-only select: chevron + label in outline chrome) vs **input-like** (editable combobox: **Filled** textfield interior, or a dedicated composite template).
  * Gallery/inspector state matrices should show outline vs ghost trigger previews per control, same as popup menu.
* **Status**: Trigger **background fill** shipped (`ShadcnTextFieldStyle::Input` → `--background`). **Shadow/elevation** on selectors is the next step after visual verification.

---

## Tasks

### Phase 1: SDK Shared Layout & Look Updates (`crates/sdk`)
- [ ] Implement `ShadowProjectionInsets` and associated bounding helpers inside `crates/sdk/src/theme/layout.rs`.
- [ ] Add shadow configuration fields to `ButtonFamilyLook` (and other control look structs).
- [ ] Update standard templates (like `ButtonTemplate`) to draw configured shadows using an absolute-positioned backing layer (matching the prototype's `render_shadow` structure).

### Phase 2: Downstream look-shadcn Resolution (`crates/look-shadcn`)
- [ ] Update the `ButtonMetricsRule` parser in `look-shadcn` to resolve the parsed `shadow` string token against `ShadcnLook::shadow(ShadcnShadow)`.
- [ ] Implement `ShadcnElementExt::shadow_cn` inside `ext.rs` to allow fluent container shadow chains (e.g. `div().shadow_cn(ShadcnShadow::Lg)`).
- [ ] Register default shadow rules for outline and primary default controls inside `crates/look-shadcn/assets/style.toml`.

### Phase 2b: Textfield Filled Elevation — **Done**
- [x] SDK: `shadow` on `TextFieldLook` / `TextFieldPalette`; template applies `.shadow()` on `#*-control` when enabled.
- [x] look-shadcn: `TextfieldElevationRule`, `find_textfield_elevation_rule`, `textfield_elevation_shadow()`.
- [x] TOML: `[[textfield.elevation_rules]]` — `filled` → `shadow-xs`; `surface` / `soft` → `none`.
- [x] `ShadcnTextFieldStyle::Filled` color rules (`background` fill) + `ShadcnTextFieldExt::filled()`.
- [x] Gallery textfield inspector + Filled state preview row.
- [x] Unit tests: filled palette resolves opaque fill + non-empty shadow; surface/soft resolve no shadow.
- [ ] Manual verify: Filled shadow reads as exterior drop shadow (not inner bleed) on page bg and on card.
- [ ] Manual verify: shadow not clipped in typical form layouts (watch §2 Overflow Seam).
- [ ] **Not in scope:** Surface / Soft elevation.

### Phase 3: Upgrade Shadow-Using Controls & Refactor Button Base
- [ ] **Selection trigger chrome (prerequisite)**: Add outline default (+ optional ghost) trigger templates to `selector`, `combobox`, `search_selector`, and `autocomplete` — delegate to command button palette/elevation, not ghost+border hybrid or naked textfield surface rules. See addendum § “Selection Trigger Chrome”.
- [ ] Upgrade existing shadow-using controls first to resolve their shadows dynamically via look-shadcn catalog metric rules and stylesheet configurations (`style.toml`):
  - [ ] Floating/Popup surfaces: `floating_menu`, `combobox`, `selector_panel`, `search_selector`, and `autocomplete`.
  - [ ] Containers: `card` and `selection_panel`.
  - [ ] Control thumbs: `switch` and `slider`.
- [ ] Add shadow configuration directly to the `button` base templates/models inside `look-shadcn` and the SDK to automatically propagate shadow support to all derived controls (`button`, `toggle`, `checkbox`, `radio_button`).
- [ ] Keep the prototype `shadow_button` page in the `prototypes/` folder as an isolated testing/tuning harness, rather than migrating it, to let developers continue to slide and tweak offsets in isolation.
- [ ] Refactor dashboard previews and cards in `apps/theme-studio` and `apps/gallery` to use the new `.shadow_cn(...)` builder methods instead of hardcoded shadow parameters.
- [ ] Verify that dark-mode and light-mode color transitions for shadows render correctly under both verification apps.

---

## Acceptance Criteria
- SDK controls correctly paint shadows defined in the `style.toml` stylesheet rules (e.g. outline button receives the default theme shadow).
- **Filled textfield** paints `shadow-xs` from `[[textfield.elevation_rules]]` with exterior drop shadow (no inner bleed); **Surface** textfield does not paint elevation.
- Shadow glow edges do not clip when children are contained inside bounded layouts.
- Swapping themes (e.g. to a theme with high shadow-blur offsets) updates the shadow projections and offsets immediately in Theme Studio.
- The project builds cleanly with `cargo check` and compiles without warnings.
