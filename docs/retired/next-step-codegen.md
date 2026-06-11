# Next step: appearance codegen

**Status:** Proposed — not implemented.

**Scope:** Appearance matrices, per-control variant support, default choice weights, and build-time validation/code generation. Templates stay responsible for structure (layout, geometry, interaction wiring); themes stay responsible for values and policy.

---

## The actual problem

The hardest appearance constructor in the tree is **`theme/radix/button.rs`**, not the choice controls in `properties/`. It builds `ButtonFamilyAppearance` from four coupled axes:

| Axis | Values | Effect |
|---|---|---|
| **Style** | `Primary`, `Secondary`, `Outline`, `Ghost` | Selects `RadixPalette::action(style)` |
| **Role** | `Text`, `Icon`, `Toggle { selected }` | Metrics; toggle unselected **rewrites style → Outline** |
| **Selection** | toggle selected or not | Background/foreground take different palette branches when selected |
| **Interaction** | default / hovered / pressed / disabled | Layer on fill colors |

That policy is two large `match` blocks plus inline role logic:

```rust
// Style override — pairing rule, not visible in a simple token table
let style = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
    RadixButtonStyle::Outline
} else {
    style
};

// foreground: 7 arms on (style, selected, disabled)
// button_background: 19 arms on (style, selected, layer)
```

Simpler controls (checkbox, switch, radio) mostly call shared primitives (`resolve_action_layer`, `outline`) with one style axis. **Button is the reference implementation** — if we can represent button appearance declaratively, the rest compose from the same resolver vocabulary.

Today the matrix lives only as Rust matches. `usage.rs` documents button tokens separately and still drifts. There is no validated data layer between “what we intend” and “what `button_appearance` does”.

**This is not a palette-import problem.** Colors come from tweakcn CSS → `RadixPalette`. No `lexicon.toml`, no `default-theme.toml`, no theme TOML stems. Codegen targets **how `ButtonFamilyAppearance` (and siblings) get constructed**, not where palette values are stored.

---

## What we want instead

A **declarative appearance matrix** for button-family: for each colored field on `ButtonFamilyAppearance`, declare the token/palette source for each `(style, role, selected, layer)` cell — including **style rewrite rules** and **selection branches**.

```text
tweakcn CSS  →  RadixPalette (runtime)
                      ↓
button appearance matrix (build-time data)
                      ↓
generated lookups  →  ButtonFamilyAppearance
                      ↓
ButtonTemplate / toggle segments  →  paint only
```

Checkbox, switch, and radio matrices should **reuse the same style/action primitives** defined for button, not be piloted first.

### Split responsibilities

| Concern | Owner | Button example |
|---|---|---|
| **Color slot policy** | Appearance matrix | `background` when toggle selected + primary style + hovered → `primary.hover` |
| **Style rewrite / pairing** | Matrix `rewrite` or `when.role` rules | toggle unselected → effective style `outline` |
| **Selection semantics** | Matrix rows keyed on `selected` | selected toggle + non-secondary → `palette.selected_background` |
| **Resolver primitives** | Rust (`palette.action`, layer hover/press) | Map matrix `source` strings to `RadixActionRole` fields |
| **Metrics & role layout** | Rust | icon pill radius, padding_x/y by `ButtonFamilyRole` |
| **Focus adorner geometry** | Rust | ghost inset vs oversize ring — placement is structure, color is `focus_ring` |
| **GPUI interaction** | Control templates | hit targets, toggle selection — no tokens |

---

## Button matrix (conceptual shape)

Pilot file: button-family appearance recipe — **not** a theme pack.

```toml
[controls.button_family]
styles = ["primary", "secondary", "outline", "ghost"]

# Effective style after role rules
[[controls.button_family.rewrite]]
when = { role = "toggle", selected = false }
effective_style = "outline"

[controls.button_family.slots.background]
[[controls.button_family.slots.background.when]]
disabled = true
source = "palette:disabled_background"

[[controls.button_family.slots.background.when]]
selected = true
style = "secondary"
layer = "default"
source = "action:background"       # secondary action role

[[controls.button_family.slots.background.when]]
selected = true
style = "secondary"
layer = "hovered"
source = "action:hover_background"

# … pressed, other layers …

[[controls.button_family.slots.background.when]]
selected = true
# style != secondary — uses selection palette, not action(style)
source = "palette:selected_background"

[[controls.button_family.slots.background.when]]
selected = false
style = "primary"                  # or any non-rewrite style
layer = "default"
source = "action:background"

[controls.button_family.slots.foreground]
[[controls.button_family.slots.foreground.when]]
disabled = true
source = "palette:disabled_foreground"

[[controls.button_family.slots.foreground.when]]
selected = true
style = "secondary"
source = "action:foreground"

[[controls.button_family.slots.foreground.when]]
selected = true
source = "palette:selected_foreground"

[[controls.button_family.slots.foreground.when]]
source = "action:foreground"       # all unselected styles

[controls.button_family.slots.border]
source = "action:border"           # from effective style's action role
```

Generated code replaces `button_background(...)` and the foreground `match`. A thin `assemble_button_appearance(...)` keeps metrics, adorner placement, and the style-rewrite prelude (or that prelude is also generated from `rewrite` rules).

---

## Why button first

1. **Highest match complexity** — 19 + 7 arms before counting role/style rewrite.
2. **Hub for the style ladder** — `RadixButtonStyle`, `palette.action(style)`, and `properties/action.rs` token pairs all exist because of button.
3. **Downstream consumers** — toggle templates, `button_family_theme`, gallery button pane, and choice controls that inherit `{style}` semantics.
4. **Proves the hard cases** — style rewrite, selection palette vs action role, interaction layers on filled roles.

Once button is data-driven, checkbox/switch/radio matrices become short compositions of the same `action:*` and `outline` sources.

---

## Codegen / validation (build time)

| Check | Why |
|---|---|
| Every slot name ∈ `ButtonFamilyAppearance` | No typos |
| Every `when` clause uses known `(style, role, selected, layer)` combos | Matches `ButtonFamilyRole` capabilities |
| Every `source` references a valid palette field or action primitive | Dead paths fail at build |
| Rewrite rules cover toggle unselected (and any future pairing) | No hidden Rust overrides |
| Full grid covered for `(effective_style, selected, layer)` on background/foreground | No runtime fallback arms |

**Emit:**

1. Static lookup tables indexed by compressed state key, **or**
2. Generated `match` with identical semantics to today's `button.rs` but never hand-edited.

Optional later: derive `usage.rs` button parts from the same matrix.

No runtime TOML. Palette still loaded from CSS only.

---

## Design & DX Recommendations

### 1. Developer Experience (DX) & Debugging
- **Clean Generated Code**: The build script should emit human-readable Rust `match` blocks rather than opaque binary tables, with comments pointing back to the corresponding TOML source/rules to support standard Rust IDE navigation and debugging.
- **Cargo Rebuild Tracking**: The `build.rs` script must register dependencies via `cargo:rerun-if-changed=path/to/matrix.toml` so edits to the declarative rules trigger immediate recompilations.

### 2. Schema Optimization & Redundancy
- **Defaults and Mixins**: To avoid repeating boilerplate (e.g. `disabled = true -> source = "palette:disabled_background"`) across every slot of every control, the schema should support a shared `[defaults]` block or wildcard (`_`) fallbacks.

### 3. Deterministic Rewrite Ordering
- **Sequential Evaluation**: Rewrite rules (e.g. converting unselected toggles to `outline` style) must be evaluated sequentially in the order they are defined to establish the final `effective_style` before slot evaluation.

---



## What stays hand-written in Rust

- `RadixPalette::action`, catalog → palette conversion
- Button **metrics** by role and size — height, padding, radius, typography
- Focus **adorner placement** (inset vs oversize) — geometry policy
- `RadixTheme` loading, mode toggle
- Control templates — paint `ButtonFamilyAppearance` fields only

---

## Rollout

| Phase | Deliverable |
|---|---|
| **0 (now)** | Treat `button.rs` matches as the spec; stop adding arms by hand |
| **1** | Appearance matrix + codegen for **button-family**; delete `button_background` / foreground matches |
| **2** | Express checkbox, switch, radio as compositions of button action primitives |
| **3** | Remaining `properties/*` controls (slider, textfield, navigation_sidebar, …) |
| **4** | Generate or verify `usage.rs` from matrices |

---

## Key paths (today)

| Role | Path |
|---|---|
| **Pilot — appearance construction** | `crates/sdk/src/theme/radix/button.rs` |
| Style enum + palette action lookup | `crates/sdk/src/theme/radix/palette.rs` |
| CSS token pairs for catalog path | `crates/sdk/src/theme/radix/action.rs` |
| `ButtonFamilyAppearance` definition | `crates/sdk/src/controls/button_family/theme.rs` |
| Theme factories / toggle binding | `crates/sdk/src/theme/radix/templates.rs` |
| CSS palette (runtime) | `apps/gallery/tweakcn/*.css` → `RadixTheme::from_css_path` |
| Simpler controls (follow button) | `crates/sdk/src/theme/radix/{checkbox,switch,radio}.rs` |

---

## Explicitly not this work

- `lexicon.toml`, `default-theme.toml`, imported theme TOML, or palette-import CLI
- Replacing tweakcn CSS as the palette source
- Moving button metrics or focus-ring geometry into data files
- Runtime-reloaded appearance matrices
