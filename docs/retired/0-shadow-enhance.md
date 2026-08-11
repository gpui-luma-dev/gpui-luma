# Fix Note: Shadow Token Model and Luma Studio Round-Trip

**Area:** `crates/look-shadcn/src/{look,shadow,tokens}.rs`, `crates/look-shadcn/assets/tweakcn/*.css`, `apps/luma-studio/src/studio/{overrides,theme_sidebar/panels/other}.rs`  
**Status:** Open — composite override propagation, parts modeling, and **parts ↔ ladder regeneration** are still TODO.  
**Owns:** Luma Studio Other → Shadow coherence with the catalog ladder (this note only).  
**Related:** [2-elevation.md](../retired/2-elevation.md) — control paint + projection (**done**, retired 2026-07-19).

---

## Boundary (read first)

| Topic | Status | Owner |
|-------|--------|--------|
| Control elevation resolve/paint (`elevation_rules`, slots, Primary textfield, etc.) | **Done** | [2-elevation.md](../retired/2-elevation.md) |
| Parts → regenerate full `--shadow-2xs`…`--2xl` on Other-tab edit | **Open** | **This note** |
| Outline / Surface textfield elevation | **Won't do** | Product: Primary-only elevation |
| Absolute-positioned shadow backing | **Unrelated leftover** | Only if clip returns after elevation slots; not ladder work |
| Full manual bleed/clip QA matrix | **Unrelated leftover** | Optional polish; not blocking |

Do **not** mix ladder regeneration with control paint/clip follow-ups.

---

## Summary

Tweakcn themes (e.g. `academic.css`, `bubblegum.css`) export a **two-layer shadow system** in `:root` / `.dark`:

1. **Decomposed parts** — `shadow-x`, `shadow-y`, `shadow-blur`, `shadow-spread`, `shadow-opacity`, `shadow-color`
2. **Elevation ladder** — `shadow-2xs` … `shadow-2xl`, plus base `shadow`

look-shadcn already **parses** all of these into the mode catalog and resolves the ladder via `ShadcnShadow` + `look.shadow()`. Controls resolve elevation through `style.toml` rules that reference catalog keys (`shadow-xs`, `shadow-md`, etc.).

Luma Studio **Other → Shadow** edits a **single composite** shadow (color, opacity, blur, spread, offset). That override must stay consistent with the full ladder so SDK controls and cards respond together.

The gap: we do not yet model parts and ladder as a **unified, regenerating system**. The current override path updates only the base `shadow` token; it neither updates the decomposed parts nor propagates the change to the ladder.

---

## What Tweakcn CSS Contains (reference: `academic.css`)

### Parsed by look-shadcn (`:root` / `.dark`)

| Token group | Examples | Used today |
|-------------|----------|------------|
| Parts | `--shadow-x: 3px`, `--shadow-color: #000000`, … | Stored in catalog; **no Rust API** |
| Ladder | `--shadow-2xs` … `--shadow-2xl`, `--shadow` | Catalog + `look.shadow(ShadcnShadow::*)` |
| Base metrics | `--radius`, `--spacing` | `look.radius()`, metrics scaffold |

### Not parsed (Tailwind bridge only)

The `@theme inline` block maps `--radius-sm: calc(var(--radius) - 4px)` and `--shadow-xs: var(--shadow-xs)` for web CSS. GPUI does not need these; real values already live in `:root`.

---

## Current Architecture

### Symbolic API (look-shadcn)

| Rust enum | Catalog / computation | Resolved type |
|-----------|----------------------|---------------|
| `ShadcnShadow` | `shadow-2xs` … `shadow-2xl`, `shadow` | `Vec<BoxShadow>` via `look.shadow(role)` |
| `ShadcnRadius` | computed from `--radius` (−4 / −2 / 0 / +4 px) | `f32` via `look.radius(role)` |

`ShadcnShadow` and `ShadcnRadius` do not yet expose `ALL` / `css_name()` like `ShadcnToken`.

### Luma Studio Other tab

- Sliders → `ThemeShadowOverride` (one box-shadow layer)
- Saved as `"shadow"` in `StudioOverrides::token_overrides()`
- Load reads first layer of `--shadow` via `default_shadow_override(look)`
- **Does not** read or write `shadow-x` … `shadow-color` catalog keys directly

### Override application (current behavior)

When `"shadow"` is overridden, `apply_token_overrides` replaces only the `shadow` catalog key. It does not update the decomposed parts or the ladder keys (`shadow-2xs` … `shadow-2xl`).

**Effect:** cards and other consumers of `shadow` pick up Luma Studio edits. Controls configured with `shadow-xs`, `shadow-sm`, or `shadow-md` continue using their previous ladder values.  
**Result:** the base shadow and elevation ladder can become inconsistent; the ladder is not flattened in the current implementation.

### Consumers

| Consumer | Resolution path |
|----------|-----------------|
| Cards / dashboard chrome | `look.shadow(...)` / `shadow_cn` |
| Buttons (outline), toggles, choices | `style.toml` elevation → catalog key → `parse_shadow_token` |
| Floating menu, Primary textfield/textarea | `shadow-md` / `shadow-xs` via stylesheet rules |
| App one-off elevated primary | `ButtonBuilder::with_look` + `look.shadow(...)` (no look change) |

Control paint / projection is done ([2-elevation.md](../retired/2-elevation.md)). This note owns **ladder parts ↔ Luma Studio round-trip only**.

---

## Problem Statement

**Full support** requires three layers to stay in sync:

```
shadow parts  →  shadow ladder (2xs…2xl)  →  look.shadow() / controls
       ↑                                              ↓
              Luma Studio Other tab
```

Today:

- Parts exist in catalog but are disconnected from Other tab and from ladder regeneration.
- Ladder exists in catalog but Luma Studio override does not update it, leaving the base shadow and ladder inconsistent.
- `ShadcnShadow` is complete symbolically; decomposed parts have no symbolic type.

---

## Proposed Solution

### 1. Shadow token module (`look-shadcn`)

Add a structured model (new module or extend `shadow.rs`):

```rust
// Conceptual — names TBD
pub struct ShadcnShadowParts {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub opacity: f32,
    pub color: Hsla,
}

impl ShadcnShadow {
    pub const ALL: [Self; N] = [ ... ];
    pub fn css_name(self) -> &'static str;  // "shadow-md", etc.
}
```

**`ShadcnLook` API:**

| Method | Purpose |
|--------|---------|
| `shadow_parts(&self) -> ShadcnShadowParts` | Read from catalog parts, or derive from `--shadow` first layer |
| `shadow_level(&self, role: ShadcnShadow) -> Vec<BoxShadow>` | Alias / rename of today’s `shadow()` |
| `shadow_ladder(&self) -> impl Iterator<(ShadcnShadow, &str)>` | Raw CSS strings for inspect / export |
| `shadow_ladder_css(&self) -> BTreeMap<String, String>` | All keys for override write-back |

### 2. Ladder regeneration

Tweakcn bakes differentiated ladders in `:root` (e.g. academic: same 3px offset, opacity and second-layer offsets vary by level). Luma Studio edits should **regenerate** the ladder, not copy one string everywhere.

**On theme load**

- Snapshot baseline ladder from catalog (optional: store per-level ratios relative to `--shadow`).

**On parts or base-`shadow` edit**

- Regenerate all `shadow-*` catalog strings using tweakcn-like rules, e.g.:
  - `2xs` / `xs`: single layer, reduced opacity
  - `sm` … `xl`: two layers; second-layer offset/blur scales with level
  - `2xl`: stronger primary-layer opacity

Rules can be derived from fixture themes (`academic.css`, `bubblegum.css`, `native.css`) and tested for stable output.

**Replace** flat fan-out in `apply_string_overrides_to_catalog` with `regenerate_shadow_ladder(parts) → write all keys + parts`.

### 3. Override application strategy

| Override input | Action |
|----------------|--------|
| `shadow-x` … `shadow-color` | Update parts → regenerate ladder → write all `shadow-*` keys |
| `shadow` only (current Other tab) | Parse layer → update parts → regenerate ladder |
| Per-level `shadow-md` (future) | Update that key; optionally reconcile parts from default level |

Fan-out of one identical string remains a **fallback** only if regeneration is unavailable.

### 4. Luma Studio (thin layer)

**Load (Other → Shadow):**

- Prefer `look.shadow_parts()` when catalog has decomposed keys
- Else parse `--shadow` first layer (current `default_shadow_override` behavior)

**Save:**

- Write parts (or composite `shadow`) to overrides
- look-shadcn regenerates full ladder on `apply_token_overrides`

**Optional later:** read-only ladder preview (xs / md / xl) in Other or Design Tokens panel.

Luma Studio should **not** own generation logic — it belongs in look-shadcn so gallery and apps benefit.

### 5. Radius (same pattern, smaller scope)

Radius scale is already computed via `ShadcnRadius` from `--radius`. For parity:

- `look.radius_ladder() -> { Sm, Md, Lg, Xl }` as px values (materialized)
- On `--radius` override, recompute; optionally write `radius-sm` … into catalog only if CSS export is added

No need to parse `@theme` calc aliases.

### 6. App helpers (optional, look-shadcn)

`ShadcnButtonElevationExt` — wrap `with_look` + `look.shadow(...)` for one-off elevated buttons (primary has no default elevation in `style.toml`).

---

## Implementation Order

1. `ShadcnShadow::ALL` + `css_name()`; `shadow_ladder()` read API
2. `ShadcnShadowParts` + read from catalog / derive from `--shadow`
3. Ladder generator + unit tests against `academic.css`, `bubblegum.css` fixtures
4. Replace flat fan-out with regeneration on override apply
5. Luma Studio: load/save through parts + regeneration
6. Keep [2-elevation.md](../retired/2-elevation.md) cross-links current; note in `docs/architecture.md` if the ladder model becomes a public contract

---

## Tests

| Test | Assert |
|------|--------|
| Fixture load | All nine ladder keys present in catalog after `from_built_in_theme("academic")` |
| Parts read | `shadow_parts()` matches `:root` `shadow-x` … `shadow-color` |
| Level parse | `look.shadow(Md)` ≠ empty; md differs from xs on unloaded academic |
| Regenerate | After parts edit, ladder keys differ by level; md ≠ xs |
| Override round-trip | Luma Studio–style single-layer edit updates parts + full ladder |
| Control resolution | Outline button elevation still resolves `shadow-xs` (or configured rule) from updated catalog |

---

## Explicit Non-Goals

- Parsing `@theme inline` self-referential aliases
- Storing only `--shadow` and dropping the ladder (controls use `shadow-xs`, `shadow-md`, …)
- Moving generation logic into Luma Studio or SDK
- Per-level shadow editors in Luma Studio v1 (regeneration from one editor is enough initially)
- Outline / Surface textfield elevation (Primary-only; won't add)
- Absolute shadow backing or bleed/clip QA (unrelated leftovers from elevation; revisit only if clipping appears)

---

## Progress Log

| Date | Change |
|------|--------|
| 2026-08-02 | **Revalidated:** the documented fan-out is not present in the current implementation. Composite edits update only `shadow`; ladder propagation and parts-based regeneration remain open. |
| 2026-07-19 | **Status clarified:** elevation paint ([2-elevation](../retired/2-elevation.md)) retired. This note remains open for ladder regen only. Documented unrelated leftovers (absolute backing, clip QA) and won't-do Outline/Surface elevation. |
| 2026-07-08 | **Shipped:** `apply_token_overrides` fan-out — `"shadow"` override copied to all `shadow-*` catalog keys so SDK controls respond to Other tab (cards already used `--shadow`). |
| 2026-07-08 | **Identified:** Fan-out flattens tweakcn ladder; parts (`shadow-x` …) not wired to Other tab; no `ShadcnShadowParts` model. |
| 2026-07-08 | **Doc:** This note created. |

---

## References

- Example full theme: `crates/look-shadcn/assets/tweakcn/academic.css` (`:root` lines 42–55)
- Shadow parse: `crates/look-shadcn/src/shadow.rs`
- Symbolic shadow roles: `crates/look-shadcn/src/tokens.rs` (`ShadcnShadow`)
- Look resolution: `crates/look-shadcn/src/look.rs` (`shadow()`, `apply_token_overrides`)
- Luma Studio shadow UI: `apps/luma-studio/src/studio/theme_sidebar/panels/other.rs`
- Overrides: `apps/luma-studio/src/studio/overrides.rs` (`ThemeShadowOverride`, `token_overrides`)
- Gallery tuning prototype: `apps/gallery/src/gallery/panes/prototypes/shadow_button/`
