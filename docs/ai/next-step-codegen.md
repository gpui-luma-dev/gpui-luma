# Next step: sdk-theme codegen (data-driven appearance)

**Status:** Proposed — not implemented. Documents the target architecture for moving **color and choice policy** out of hand-written Rust `match` blocks and into a build-validated **sdk-theme** data file.

**Scope:** Appearance matrices, per-control variant support, default choice weights, and build-time validation/code generation. Templates stay responsible for **structure** (layout, geometry, interaction wiring); themes stay responsible for **values and policy**.

**Not in scope here:** Palette import from tweakcn ([`next-step-theme-import.md`](next-step-theme-import.md) — superseded), active theme wiring ([`next-step-theme.md`](next-step-theme.md) — superseded), or adding new ladder rungs to the public API (`docs/ai/next-step-variants.md`). Those docs remain separate; codegen consumes their outputs.

---

## Problem today

After the variant-ladder and choice-control work (toggle, switch, checkbox, radio), appearance policy is spread across many `*Theme::resolve` implementations as large `match` expressions that mostly **swap token paths** based on `(control state, ButtonKind, interaction layer)`.

Examples of policy currently encoded in Rust rather than data:

- Checkbox / switch / radio: checked/on + `ButtonKind::Standard` vs `Prominent` → `action.standard.*` vs `action.prominent.*`
- Button-family toggle: unselected segments always resolve as `Subtle`; selected weight comes from `model.kind`
- Per-control defaults: everything inherits `ButtonKind::Standard` from `ButtonBuilder` unless `.kind(...)` is set

This works for the gallery and Astrovista, but it is a **code smell**: the matrices are style sheets written as Rust. Adding a control state or changing a default requires touching multiple resolver files and risks drift between checkbox, switch, radio, and button-family paths.

### Interim product convention (until codegen lands)

Live with the current code using this rule:

- **`ButtonKind::Standard` by default** everywhere it applies (forms, groups, toolbars, intro panels).
- **`ButtonKind::Prominent` only when explicitly needed** (primary emphasis, gallery matrix rows, deliberate demos).
- **Astrovista** exercises the full ladder (coral prominent, navy standard, subtle outlines); **Jarvis** stresses typography and borders more than dramatic choice-state color contrast — same policy can work on both; palette TOML supplies the hues.

Explicit `.kind(...)` remains the override for matrices, introduction panels, and one-offs.

---

## Target architecture

Separate three concerns that are mixed today:

| Layer | Owns | Example |
|---|---|---|
| **SDK (Rust)** | Control APIs, interaction, template **structure**, closed **variant enum** | `ButtonKind`, thumb position, focus adorners |
| **sdk-theme (data)** | Which variants each control supports, **defaults**, appearance **matrices** (state × variant → token path) | `checkbox.checked + prominent → action.prominent.background` |
| **App theme TOML** | **Palette values** (HSL) + optional policy overrides | Astrovista `action.standard.background = "hsl(...)"` |
| **Instance API** | Force a variant when theme default is wrong for one control | `.kind(ButtonKind::Prominent)` |

```text
sdk-theme.toml (embedded in SDK, validated at build)
  ├── global ladder: prominent | standard | subtle | ghost
  ├── per-control: supported variants, default weights, pairing rules
  └── appearance matrix: (control, slot, state, variant, layer) → token path

default-theme.toml + imported themes (Astrovista, Jarvis, …)
  └── palette only — HSL values for action.*, form.*, state.*, …

Runtime
  effective_variant = instance.kind ?? theme.choice.<control> ?? sdk-theme default
  appearance[slot]  = resolve_token(matrix entry, active ThemeTokens)
  template            = draw using appearance fields (no color match)
```

Policy like “toggle unselected uses Subtle” belongs in **sdk-theme**, not in `DefaultButtonFamilyTheme` Rust long term.

---

## Analogy: attributes + reflection (other languages)

In ecosystems with attributes/reflection, controls declare supported variants (A, B, C) and a **data file** maps control × state × variant → aesthetic properties; the **build** proves the file matches control capabilities.

Rust has no runtime reflection for this. The equivalent is:

1. **Declarative matrix** in `sdk-theme.toml`
2. **`build.rs` validation** — unknown variant names, invalid token paths, or unsupported control/state combos fail the build
3. **Optional codegen** — emit lookup tables or thin resolver glue so Rust stops duplicating matrices by hand

The developer experience goal: **A, B, C remain typed enum variants** (`ButtonKind::Standard`, etc.) at the call site, while **which variant applies by default** and **which token fills which appearance field** live in data tied to the SDK build.

---

## sdk-theme.toml (conceptual shape)

Two sections: **choice policy** (which ladder rung when) and **appearance matrix** (which token fills which slot).

### Global ladder

Variant names in TOML are **strings that must match the SDK ladder** — TOML configures known rungs; it does not invent new ones without an SDK release.

```toml
[ladder]
variants = ["prominent", "standard", "subtle", "ghost"]
```

### Per-control choice policy

```toml
[controls.checkbox]
selected_variants = ["standard", "prominent"]
default_selected = "standard"   # or "prominent" per product/theme override later

[controls.switch]
on_variants = ["standard", "prominent"]
default_on = "standard"

[controls.radio]
selected_variants = ["standard", "prominent"]
default_selected = "standard"

[controls.toggle]
selected_variants = ["standard", "prominent"]
default_selected = "standard"
unselected = "subtle"           # pairing rule (today hardcoded in button-family Rust)

[controls.button_family.toggle_segment]
# same pairing semantics as toggle in control groups
unselected = "subtle"
```

App theme TOML may optionally override `[light.choice.*]` / `[dark.choice.*]` for brand-specific defaults without changing sdk-theme structure.

### Appearance matrix (per control)

Declare **slots** (maps to `*Appearance` struct fields) and **entries** (state + variant + interaction layer → token path on `ThemeTokens`):

```toml
[controls.checkbox.slots]
indicator_background = "CheckboxAppearance.indicator_background"
indicator_border     = "CheckboxAppearance.indicator_border"
checkmark_color      = "CheckboxAppearance.checkmark_color"

# Conceptual — exact schema TBD at implementation
[[controls.checkbox.matrix]]
state = "checked"
variant = "standard"
layer = "default"
indicator_background = "action.standard.background"
checkmark_color      = "action.standard.foreground"

[[controls.checkbox.matrix]]
state = "checked"
variant = "prominent"
layer = "default"
indicator_background = "action.prominent.background"
checkmark_color      = "action.prominent.foreground"
```

Same pattern extends to switch, radio, button-family (toggle segments), and eventually plain buttons — one engine, many control tables.

---

## Codegen patterns (implementation options)

Three patterns, in recommended order:

### Pattern 1 — Hand-written enum, codegen validates + emits lookup tables (first milestone)

- Keep `ButtonKind` / `ButtonVariant` as manually maintained Rust enums (see `docs/ai/next-step-variants.md`).
- `build.rs` reads `sdk-theme.toml` and:
  - validates every variant string ∈ ladder
  - validates every token path exists on `ThemeTokens`
  - validates every matrix row references a supported control state
- Emits `generated/checkbox_matrix.rs` (or similar): static tables used by `CheckboxTheme::resolve` instead of hand-written `match` arms.

**API unchanged:** `checkbox::new(...).kind(ButtonKind::Prominent)`.

### Pattern 2 — Codegen emits enum + tables from sdk-theme (single source of truth)

- Global ladder defined only in `sdk-theme.toml`.
- Build emits `ButtonKind` and per-control `SUPPORTED_VARIANTS` constants.
- SDK releases that add `destructive` update the ladder in one file, regenerate Rust.

**Trade-off:** stronger consistency; generated public API requires stable codegen and review discipline.

### Pattern 3 — Fully generic runtime resolver (later)

- One Rust engine: `(control_id, slot, state, variant, layer) → token path → Hsla`.
- All control-specific `*Theme::resolve` thin wrappers or removed.
- Matrices live entirely in sdk-theme; build validates completeness (every required cell filled).

**Trade-off:** least duplicated Rust; highest investment in schema and debugging tooling.

---

## Resolution order (instance → theme → sdk-theme)

When implemented, effective variant for choice controls should resolve in this order:

1. **Explicit instance** — `.kind(ButtonKind::…)` on the builder (gallery matrices, intro overrides).
2. **App theme policy** — optional `[light.choice.checkbox]` / `[dark.choice.checkbox]` in imported theme TOML.
3. **sdk-theme default** — per-control `default_selected` / `default_on` / pairing rules.
4. **SDK fallback** — hard-coded safety default (likely `standard`) if data is missing (should not happen when build validation passes).

Open design question for implementation: whether “omit `.kind()`” means **inherit theme default** vs **Standard**. Today they are the same because the builder always sets `Standard`. Theme-level defaults require distinguishing **unset** from **explicit Standard** on the model, or a dedicated “inherit” sentinel in the API.

---

## Build-time guarantees

Codegen should fail the SDK build when:

| Check | Why |
|---|---|
| Unknown variant string in sdk-theme | Prevents typos (`"prominant"`) |
| Token path not on `ThemeTokens` | Prevents dead matrix entries |
| Matrix row for unsupported control/state | Keeps data aligned with control capabilities |
| Missing required matrix cell | Prevents incomplete appearance at runtime |
| `ThemeUsage` / gallery matrix docs drift from sdk-theme | Optional: generate `ThemePartUsage` snippets from the same file |

This is how **sdk-theme stays tied to the build** without runtime reflection.

---

## What stays in Rust (not codegen)

Even with full matrices in data, Rust remains responsible for:

- **Template structure** — switch thumb travel, radio dot geometry, checkbox checkmark glyph, button border width
- **Interaction** — hover, press, focus, disabled, group selection semantics
- **Layout policy** — icon vs text padding, control-group chrome (may have its own sdk-theme section later)
- **New ladder rungs** — adding `Destructive` requires enum + palette slot + lexicon + sdk-theme row (not TOML-only extensibility)

Codegen removes **color-swapping match smell**; it does not replace the control framework.

---

## Relationship to existing theme pipeline

```text
tweakcn CSS  →  catalog  →  palette (lexicon)  →  theme.toml (colors)
                                                      ↑
sdk-theme.toml (behavior + matrices) ─────────────────┘
         ↓ build.rs
    generated Rust tables + validation
         ↓
    *Theme::resolve reads matrix + active ThemeTokens
```

| Artifact | Role after codegen |
|---|---|
| `lexicon.toml` | Import: catalog → Luma **palette paths** (colors) |
| `default-theme.toml` | Base **palette** + metrics + typography |
| `sdk-theme.toml` | **Behavior + appearance matrices** (new) |
| `tweakcn-astrovista.toml` etc. | Palette overrides; optional choice policy overrides |
| Hand-written `*Theme::resolve` | Shrinks to generic resolve or generated glue |

See also: `docs/ai/theme-lexicon.md`, `docs/ai/next-step-theme-import.md`.

---

## Phased rollout (suggested)

| Phase | Deliverable |
|---|---|
| **0 (now)** | Standard default + explicit Prominent; document convention; stop adding new policy to templates |
| **1** | Author `sdk-theme.toml` for **one** control (checkbox); `build.rs` validates; codegen emits lookup; replace checkbox `match` arms |
| **2** | Migrate switch, radio, button-family toggle pairing to same file + engine |
| **3** | Optional app-theme `[*.choice.*]` overrides; unset vs explicit `.kind()` API decision |
| **4** | Generate `ThemeUsage` / theme matrix docs from sdk-theme; consider Pattern 2 for ladder enum |

---

## Success criteria

When this work is done:

1. Changing checkbox checked appearance for `prominent` vs `standard` is an **sdk-theme edit**, not a Rust diff across `checkbox/theme.rs`.
2. Toggle unselected → `subtle` is **data policy**, not a special case in `button_family/theme.rs`.
3. `cargo build -p gpui-luma` fails if sdk-theme references invalid variants or token paths.
4. Public API still exposes **`ButtonKind`** (or generated equivalent) for explicit overrides and gallery matrices.
5. Astrovista and Jarvis differ by **palette TOML** (and optional choice overrides), not forked resolver logic.

---

## Related code and docs

| Topic | Path |
|---|---|
| Variant ladder | `docs/ai/next-step-variants.md`, `crates/sdk/src/controls/button_family/theme.rs` |
| Choice themes (interim match blocks) | `crates/sdk/src/controls/{checkbox,switch,radio_button}/theme.rs` |
| Toggle pairing (interim) | `crates/sdk/src/controls/button_family/theme.rs` |
| Active palette resolve | `docs/ai/next-step-theme.md`, `crates/sdk/src/theme/pack.rs` |
| Import / lexicon | `docs/ai/theme-lexicon.md`, `crates/sdk/src/theme/lexicon.toml` |
| Gallery variant matrices | `apps/gallery/src/gallery/panes/{toggle,switch,checkbox,radio_button}/pane.rs` |

---

## Explicitly elsewhere

- Hot-reloading sdk-theme at runtime
- User-defined variant names beyond the SDK ladder without a release
- Replacing GPUI template code with generated UI markup
- Per-theme codegen (matrices are SDK-scoped; themes override palette and optional policy only)
