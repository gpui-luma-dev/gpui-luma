# Inspector — Step 1 shipped, Step 2 plan

**Status:** Step 1 implemented in gallery (button pane). Step 2+ planned here.

**Related:**

- [`inspector.md`](inspector.md) — original Theme Studio design (mostly pre-implementation)
- [`introspection.md`](introspection.md) — `declare_look_table!`, offline matrices
- [`look-theme.md`](look-theme.md) — `ShadcnLook`, catalog loading
- Gallery theme file: [`apps/gallery/tweakcn/retro-arcade.css`](../../apps/gallery/tweakcn/retro-arcade.css)

---

## Step 1 — Done (button color inspector)

### What shipped

Gallery **Command (Text)** button pane includes a live inspector:

- **Tree (left):** style → state → field (`background`, `foreground`, `border`, `focus ring`)
- **Detail (right):** large swatch, field name, CSS key, provenance, color value table (HSLA / HEX / RGBA / HSL) with copy
- **Resizable split** between tree and detail (`ResizablePanels`)
- **Selection** drives detail panel; Primary → default expanded on load

### look-shadcn API

| Piece | Location |
|-------|----------|
| `ResolvedColor`, `ColorSource` | `crates/look-shadcn/src/provenance.rs` |
| `format_inspect_css_key` | Catalog-honest keys (`--primary`, not synthetic `--primary-hover`) |
| `format_inspect_provenance` | Algorithmic / inherits / catalog state labels |
| `inspect_button_color_palette` | `crates/look-shadcn/src/controls/button.rs` |
| `ButtonInspectPalette` | background, foreground, border, optional focus_ring |

### Gallery UI

| Piece | Location |
|-------|----------|
| Tree builder + template | `apps/gallery/src/gallery/panes/button/inspector_tree.rs` |
| Detail entity | `apps/gallery/src/gallery/panes/button/inspector_detail.rs` |
| Shell (split + selection) | `apps/gallery/src/gallery/panes/button/inspector_shell.rs` |
| Pane wiring | `apps/gallery/src/gallery/panes/button/pane.rs` |
| Mono fallback | `apps/gallery/src/assets/fonts.rs` (`gallery_mono_font`) |

### Design choices locked in

1. **Inspector shows resolved paint truth**, not only grep targets in the CSS file.
2. **CSS keys** use tokens that exist in the loaded theme; state comes from the tree branch label.
3. **Provenance** is a separate line when non-obvious (algorithmic hover, inherited border).
4. **Typography:** body size for labels; caption size for mono lines; platform mono font until theme fonts are embedded.
5. **SDK scaffold values belong in the inspector** when they are what actually paints (see Step 2).

### Known gaps (acceptable for step 1)

- `docs/ai/inspector.md` still reads “design only” — update when convenient.
- Theme Parts sidebar still used on other gallery panes.
- No count badges / style accent dots on tree branches (mockup polish).
- `retro-arcade` `--font-mono` (Space Mono) not embedded; Menlo/Consolas fallback.
- `format_inspect_provenance` exported but only provenance line in UI uses it indirectly via field data.

---

## Step 2 — Button metrics inspector (recommended next)

### Goal

Answer: *“For this style + state + size, what layout numbers does the button paint, and where did each come from?”*

Mirror step 1: same tree/detail/split shell; add **metrics leaves** (or a **Layout** group under each state).

### Include SDK defaults now — do not wait

Show **all resolved layout fields** with honest provenance, including values **not** in `retro-arcade.css`:

| Field | Typical Md value | Provenance (today) |
|-------|------------------|-------------------|
| `height` | 36px | `scaffold · MetricTokens.control.md.control_height` |
| `padding_x` | 14px | `scaffold · …padding_x` |
| `padding_y` | 8px | `scaffold · …padding_y` |
| `gap` | 8px | `scaffold · …gap` |
| `radius` | 2px (retro-arcade) | `catalog · --radius` → `md = radius − 2px` |
| `border_width` | 1px | `scaffold · border_width.default` |
| `focus_ring_width` | 1px | `scaffold · focus.width` |
| `focus_ring_offset` | 1–2px | `derived · border_width + focus.width` (ghost: inset) |
| Icon `radius` | pill | `scaffold · radius.pill` |

**Do not** invent CSS tokens (`--button-padding`). **Do** label scaffold rows clearly so authors know what is not in the theme file yet.

When `--spacing` is wired into `metrics_from_catalog`, flip provenance for padding/gap from `scaffold` to `catalog · --spacing` without changing the UI shape.

### retro-arcade.css — metrics-relevant tokens

**Wired today**

- `--radius` (`0.25rem` → 4px base) → per-size corner radius via `metrics_from_catalog`
- `--font-sans` → button `font_family` (typography; step 3 or sibling branch)

**In file, not wired to button metrics**

- `--spacing` (`0.25rem`) — natural knob for padding/gap; currently ignored
- `--tracking-normal` — not applied
- `--radius-sm/md/lg/xl` in `@theme inline` — code recomputes from `--radius` instead of reading these keys
- `--shadow-*` — parseable via `ShadcnLook::shadow()`; buttons do not paint shadow

**Color tokens that interact with metrics UI**

- `--ring` — focus ring **color** (step 1); width/offset are scaffold
- `--border` — outline **color** (step 1); width is scaffold

### Proposed look-shadcn API

```text
inspect_button_metrics(mode, theme_mode, style, role, size, state)
  → ButtonInspectMetrics {
      height, padding_x, padding_y, gap, radius,
      border_width, focus_ring_width, focus_ring_offset, …
    }
  each field: ResolvedMetric { value_px, source: MetricSource }
```

`MetricSource` variants (parallel to `ColorSource`):

- `CssVar { token }` — e.g. `--radius`
- `Derived { note }` — e.g. `md radius = --radius − 2px`, `focus offset = border + focus.width`
- `Scaffold { path }` — e.g. `MetricTokens.control.md.padding_x`
- `Constant { label }` — e.g. `radius.pill` for icon buttons

Reuse existing resolution path: `button_appearance` → `StandardBoxScale::compute` + `compose_button_family_appearance` + focus adorner specs in `button_palette`.

### Gallery UI

- Extend `InspectTreeData` with metric leaves (or `Metric(InspectMetricData)`).
- Detail panel: same three-line row pattern — name / source / `14px` (no HSLA table unless useful).
- Optional: toggle or branch filter **Colors** vs **Layout** if the tree gets noisy.

### Tests

- `inspect_button_metrics` matches `button_appearance` fields for Primary/default/Md.
- retro-arcade catalog: radius from `--radius`, padding from scaffold.
- Focused ghost vs primary: different `focus_ring_offset` derivation.

---

## Step 3 — Complete button inspector

- **Typography branch:** `font_family` (`--font-sans`), label size/weight/line-height (scaffold), mono honesty note.
- **Template-only layers:** disabled opacity, adorner placement — `MetricSource::Derived` / `Template`.
- **Mockup polish:** branch count badges, style accent circles on Primary/Secondary.
- **Embed theme fonts** (e.g. Outfit, Space Mono) or document fallback in inspector.

---

## Step 4 — Replicate pattern

| Control | Priority | Notes |
|---------|----------|-------|
| Checkbox / Switch | High | Look tables exist; thumb/track metrics |
| Text field | High | Border/ring colors + height/padding |
| Tree view | Medium | Already custom template in gallery |

Extract shared shell when copying the second pane:

- Resizable tree + detail
- Selection → detail entity
- `gallery_pane_with_inspector` width (~620px)

---

## Step 5 — Theme Studio

Move inspector from gallery proof to Theme Studio ([`inspector.md`](inspector.md) target):

- Inspect live panels (Payments, etc.), not only button demo.
- `ColorSource::Override` / `MetricSource::Override` for sidebar edits.
- Optional catalog line numbers for grep-to-file.

---

## Step 6 — Replace static Theme Parts

Per gallery pane: inspector slot → deprecate dense Theme Parts sidebar. Keep sparse catalog callout where the CSS file is incomplete.

---

## Step 7 — Offline / CLI (optional)

From [`introspection.md`](introspection.md): dump color + metrics matrices for all states; diff themes in CI. Shares types with live inspector.

---

## Doc hygiene (when touching docs)

- [ ] Update [`inspector.md`](inspector.md) status: gallery button color inspector shipped; link here.
- [ ] Update [`module-map.md`](module-map.md) with inspector file paths if missing.
- [ ] Align [`introspection.md`](introspection.md) with `declare_look_table!` as implemented (not only proposed).

---

## Suggested implementation order

```text
1. MetricSource + inspect_button_metrics + tests
2. Gallery tree/detail rows for layout fields
3. Typography branch on button pane
4. Wire --spacing in metrics_from_catalog (provenance flip only)
5. Checkbox or TextField color inspector (clone shell)
6. Theme Studio wiring
7. CLI state-table generator
```
