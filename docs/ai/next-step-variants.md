# Next step: variant ladder (more button weights)

**Status:** Tabled — concept agreed, **not implemented** in SDK or TOML yet.

**Scope:** Luma public API names for action **visual weight**, plus theme slots to back them. This is separate from **active theme wiring** (`docs/ai/next-step-theme.md`) and **CSS import** (`docs/ai/next-step-theme-import.md`).

---

## Problem today

Luma exposes three button weights in API and theme:

```text
ButtonKind / ButtonVariant:  Standard | Ghost | Prominent
Theme slots:                 action.standard | action.ghost | action.prominent
```

shadcn / tweakcn exports **six** button recipes with different semantics:

| shadcn | Typical look |
|---|---|
| **default** | Solid primary CTA |
| **destructive** | Solid red CTA (out of scope for Luma weight ladder — deferred) |
| **secondary** | Filled muted/secondary tone (often navy or grey fill — *not* outline) |
| **outline** | Transparent/card fill + visible border |
| **ghost** | Transparent, hover tint only |
| **link** | Text-only (no chrome) |

Radix Themes goes further with recipe matrices (`solid`, `soft`, `surface`, `outline`, `ghost`, …) reused across Button, Badge, Callout, Select trigger, etc.

Trying to map both **outline Cancel** and **filled navy secondary** onto Luma **Standard** produces inconsistent gallery behavior — the mismatch is semantic, not a broken theme file.

---

## Agreed target ladder (tabled)

Use **visual weight** names in the Luma API — not Radix/shadcn variant strings:

```text
Prominent  →  Subtle  →  Standard  →  Ghost
(loudest)      (filled,          (bordered      (quietest)
                secondary tone)     neutral)
```

### Confirmed shadcn import mapping (concept)

| Luma (API) | shadcn recipe | Astrovista example |
|---|---|---|
| **Prominent** | `primary` / `default` | Coral “Upgrade Plan” |
| **Subtle** | `secondary` (filled) | Navy filled chips / selective icon toggles |
| **Standard** | `outline` | White card + grey border “Cancel” |
| **Ghost** | `ghost` | Transparent, hover tint only |

**Link** and **destructive** are out of scope for the four-weight ladder — add later when product needs them.

---

## Radix Themes cross-reference (planning)

Radix encodes **chrome recipe × color × contrast**. Luma only needs the **weight ladder** in public API; hues live in `theme.toml`.

Closest Radix → Luma weight mapping (Button / IconButton):

| Radix recipe | Closest Luma weight |
|---|---|
| `solid` + accent color | Prominent |
| `soft` / filled secondary surface | Subtle |
| `surface` / `outline` | Standard |
| `ghost`, `ghost-offset` | Ghost |

Other Radix components (Badge, Callout, Select trigger) can reuse the same weight resolver; they are not separate Luma variant enums unless product needs demand it.

**Checkbox, Switch, TextField** in Radix have no variant prop — styling comes from theme + context. Luma already follows that pattern.

---

## SDK changes (when implemented)

### 1. Public API

```rust
pub enum ButtonKind {
    Prominent,
    Subtle,    // new
    Standard,
    Ghost,
}
```

Today: `ButtonKind` has Standard | Ghost | Prominent only (`crates/sdk/src/controls/button_family/theme.rs`).

### 2. Theme TOML

Add full role under `[*.palette.action.subtle]`:

```toml
[light.palette.action.subtle]
background = "…"      # typically from CSS --secondary
foreground = "…"
hover_background = "…"
pressed_background = "…"
border = "…"
```

### 3. Resolvers

- `ButtonFamilyTheme::resolve` — map `ButtonVariant::Subtle` to `action.subtle.*`.
- Toggle selected state: default mapping stays **Prominent** or `state.selected` — not Subtle unless explicitly designed.
- Icon toolbars / toggle groups: unselected → Standard or Ghost; selected → Prominent (existing gallery pattern).

### 4. Re-map `action.standard` for imported themes

For shadcn **outline** imports, **`action.standard`** should be the bordered neutral recipe (card background + border), **not** the filled `--secondary` navy.

**`action.subtle`** owns the filled secondary tone (`--secondary` in Astrovista).

This corrects an earlier mistake where `--secondary` was mapped to `action.standard`, making Standard buttons look like shadcn Secondary instead of Outline.

### 5. Gallery

- Button / Icon Button panes: add Subtle column to variant demos when API lands.
- Template matrix panes: add Subtle row.

---

## Theme import interaction

When converting tweakcn CSS (`docs/ai/next-step-theme-import.md`):

| CSS variable | Luma slot (after variant work) |
|---|---|
| `--primary` | `action.prominent.*` |
| `--secondary` | `action.subtle.*` |
| `--background` / `--card` + `--border` | `action.standard.*` (outline recipe) |
| `--accent` | `state.hover.*` (and dark-mode hover variants) |
| `--destructive` | `form.input.invalid_border` only (no action role yet) |
| `--muted` | `surface.subtle.*`, `state.disabled.*` |

Derive hover/pressed for prominent and subtle when CSS exports only base pairs (+/- lightness steps, typically 4–9% L).

---

## Toggle / selection semantics (boundaries)

- **Subtle** is filled but de-emphasized vs Prominent — not the same as Ghost hover-only.
- **Standard** is the neutral bordered default — Cancel, secondary actions that need a box.
- **Prominent** remains the single primary CTA weight per surface unless product adds multi-accent rules later.

Open questions (defer until implementation):

- Should navigation selected item use `navigation.selected_*`, `state.selected_*`, or `action.prominent_*`?
- Should checked switch thumb use `action.prominent.foreground` or `surface.panel.background`?
- Pill-shaped controls: always `radius.pill` vs per-variant metric?

See also `docs/theme.md` open questions.

---

## Verification (when implemented)

1. Astrovista import: Prominent = coral, Subtle = navy fill, Standard = white+border Cancel, Ghost = transparent.
2. white-orange import: secondary orange maps to Subtle, not Standard.
3. Gallery Button pane shows four weights.
4. `cargo test -p gpui-luma` theme tests cover `action.subtle` parse + resolve.
5. No public API exposes shadcn names (`outline`, `secondary`, etc.).

---

## Explicitly out of scope (for this doc)

- Active theme / default template wiring
- CSS import CLI tooling
- Runtime theme hot-reload
- Per-control mini-styling in TOML
- Full Radix recipe matrix port (soft/surface/classic naming in API)
- Destructive / semantic danger styling (deferred to a future design)
