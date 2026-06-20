# Fundamental templates + SDK/theme divorce

> **Phase A note (May 2026):** Phase A closed **differently** than originally planned — gallery loads tweakcn CSS directly via `RadixTheme` (no `luma-theme` CLI, no `assets/themes` TOML). This doc remains the design record for optional **Phase B** (SDK/theme crate split). See [`next-steps-finalized.md`](next-steps-finalized.md).

**Status (May 2026):**

| Phase | State | Summary |
|---|---|---|
| **A — Theme import & gallery wiring** | **Closed (Radix/CSS path)** | Direct CSS → `RadixTheme`; TOML import toolchain removed |
| **B — SDK/theme split** | **Next** | `Look`-only SDK; theme in separate crate(s); dev theme for control work |

This doc is the **design record** for Phase B. It subsumes earlier “adapters / recipes” notes and the button `.kind` migration plan.

**Related:**

| Doc | Role |
|---|---|
| [`next-step-theme-import.md`](next-step-theme-import.md) | Phase A — import pipeline, lexicon, gallery assets (maintenance only) |
| [`theme-lexicon.md`](theme-lexicon.md) | Lexicon grammar + CLI usage |
| [`next-step-variants.md`](next-step-variants.md) | Historical four-weight ladder — **public naming to move out of SDK** |
| [`next-step-codegen.md`](next-step-codegen.md) | Optional codegen — after theme split |

---

## Decision: close current theme work, then split

**Close Phase A** — the import toolchain works for its purpose:

- Parses tweakcn/shadcn CSS (`:root` / `.dark`) into a catalog.
- Resolves `lexicon.toml` + `default-theme.toml` into `theme.toml`.
- Validates with `LumaTheme::from_toml_str`.
- Batch: `luma-theme import --source-dir apps/gallery/tweakcn --dest-dir apps/gallery/src/assets/themes`.
- Gallery: `cargo run -p gpui-luma-gallery -- <stem>` loads `apps/gallery/src/assets/themes/<stem>.toml`.

**Do not** keep iterating lexicon hover rules, golden diffs, or per-theme hand-fixes in the **married** SDK+theme model. That work identified the expected issues (derivation mismatch, recipe vs slot confusion, oklch themes, duplicate TOML names). **Phase B** addresses root cause: **the SDK should not own the design system.**

**Next focus:** complete SDK **controls** with **complete `Look`** + **`gpui-luma-theme-dev`**; attach product themes (`shadcn`/tweakcn) as optional crates.

---

## Phase A delivered (reference)

| Artifact | Path |
|---|---|
| Import CLI | `crates/luma-theme` — `import`, `catalog` |
| Lexicon | `crates/sdk/src/theme/lexicon.toml` |
| CSS samples | `apps/gallery/tweakcn/*.css` |
| Imported TOML | `apps/gallery/src/assets/themes/*.toml` |
| Gallery loader | `apps/gallery/src/gallery/theme.rs` (runtime stem → TOML) |

**Known gaps left intentionally for Phase B:**

- Full golden parity with hand-maintained TOML (hover/pressed derivation vs lexicon).
- `shadow_parse` / elevation from CSS shadows.
- Typography scale from tweakcn `rem`.
- In-app theme picker; dedicated **theme studio** app.
- Resolving import bugs by editing `palette.action.prominent` in SDK schema.

---

## North star (Phase B)

```text
gpui-luma (SDK)          →  controls + complete *Look + paint-only templates
gpui-luma-theme-dev      →  ugly, high-contrast functions — finish controls
gpui-luma-theme-*        →  optional product look (shadcn/tweakcn/Radix) — attach when needed
luma-theme (CLI)         →  feeds product theme crates only, not SDK
apps/gallery             →  dev theme by default; product theme for import demos
```

### Core principles

1. **`Look` is the only cross-boundary contract** between theme and SDK.
2. **SDK does not care** where theme data lives (TOML path, embedded str, codegen, hard-coded dev colors).
3. **Theme exposes functions**, not SDK enums — e.g. `theme.primary_button(role, size, state) -> ButtonFamilyLook`.
4. **Templates never read `palette` or `ButtonKind`** — they paint structs only.
5. **Swift/Flutter layering, not Material in the framework** — `ThemeData`-style functions in a separate layer; avoid `LumaThemePack` + default resolvers hiding missing Look fields.

Target call site:

```rust
let look = theme.primary_button(ButtonFamilyRole::Text, ControlSize::Md, state);

Button::new("submit")
    .look(look)   // or .template(themed_button_template(...))
    .spawn(cx);
```

`primary_button` lives in the **theme crate**. Renaming to `new_primary` or `cta` does not touch `gpui-luma`.

### SDK / theme divorce

```text
App (gallery, product)
  → theme.primary_button(...) / theme.checkbox(...)
       → Look
gpui-luma SDK
  → Button::new, interaction state, paint template
```

| Concern | Today (married) | After split |
|---|---|---|
| `theme.toml` / `LumaTheme::from_toml_str` | `gpui-luma` | Theme crate only |
| `palette.action.prominent` | `gpui-luma` `tokens.rs` | Theme storage (opaque to SDK) |
| `ButtonKind` + `DefaultButtonFamilyTheme::resolve` | `gpui-luma` | Theme functions |
| `LumaThemePack` + `*Theme` traits | `gpui-luma` `pack.rs` | Theme layer; optional thin wrappers |
| `set_active_theme_pack` | SDK defaults | App registers theme; SDK requires Look or dev wrapper |
| `luma-theme` import | Targets SDK-shaped TOML | Targets product theme crate storage |

`LumaThemePack` is the marriage symbol — runtime mode + every control resolver in one type inside the SDK.

### Dev theme vs product theme

**`gpui-luma-theme-dev`**

- High-contrast, visually distinct states (hover ≠ pressed ≠ disabled ≠ focused).
- Every `*Look` field populated — exposes SDK gaps, not beauty.
- No TOML/import dependency.
- Default for gallery while building control panes.

**Product theme crates** (optional)

- `gpui-luma-theme-shadcn` or per-family crates (Astrovista, etc.).
- Own `primary_button`, palette, fonts; consume `luma-theme` output.
- Gallery/product: `cargo run ... -- astrovista` only when demoing import.

### What “SDK complete” means

Using **only `DevTheme`**, done when:

1. Every gallery control pane shows all documented interaction states.
2. Dev theme makes each state visibly different.
3. No `palette.` or `ButtonKind` in `template.rs`.
4. Builders use `.look(...)` or theme wrapper templates — no silent SDK resolve.
5. `*Look` structs are stable for product themes to fill later.

**Not required for SDK complete:** import golden parity, coral accuracy, Rajdhani/Outfit embedding.

### Future: theme studio (separate app)

After dev theme + paint-only SDK:

- Fixed preview matrix (four button weights, one field, choice row).
- Inspector: click control → **Look** + **theme function** + palette paths.
- Diff imported vs golden TOML; binding report from `luma-theme`.

Studio inspects **look + provenance**, not labels like “Prominent.”

---

## Button review: where `.kind` lives today

### Flow

`ButtonKind` on builder → `ButtonRenderModel.kind` → `DefaultButtonTemplate` calls `theme.resolve(button_variant(kind), role, size, state)` → large `match` in `DefaultButtonFamilyTheme`.

### Public API

```10:26:crates/sdk/src/controls/button_family/mod.rs
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonKind {
    #[default]
    Standard,
    Subtle,
    Ghost,
    Prominent,
}

pub fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Standard => ButtonVariant::Standard,
        ButtonKind::Subtle => ButtonVariant::Subtle,
        ButtonKind::Ghost => ButtonVariant::Ghost,
        ButtonKind::Prominent => ButtonVariant::Prominent,
    }
}
```

```128:131:crates/sdk/src/controls/command/button/model.rs
    pub fn kind(mut self, kind: ButtonKind) -> Self {
        self.model.kind = kind;
        self
    }
```

### Template (already mostly fundamental)

Painting is fine; **`kind` only exists to call `resolve`:**

```47:49:crates/sdk/src/controls/command/button/template.rs
    fn render(&self, model: &ButtonRenderModel<D>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let look = self.theme.resolve(button_variant(model.kind), model.role, model.size, model.state);
```

`ButtonFamilyLook` is the right output type.

### Policy matrix (leaves SDK in Phase B)

```276:294:crates/sdk/src/controls/button_family/theme.rs
impl ButtonFamilyTheme for DefaultButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyLook {
        // ...
        let variant = if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
            ButtonVariant::Subtle
        } else {
            variant
        };
        // match (variant, selected, layer) → palette.action.* / palette.state.*
```

**Also coupled:** `control_group/button_item_template.rs` (`kind: ButtonKind`), `toggle_button_item_template.rs`, `toggle/mod.rs` modifier resolve, gallery `.kind(...)` panes, `LumaThemePack::button_template()`.

**Choice controls** reuse `ButtonKind` for checked accent only (`checkbox/theme.rs` — `Standard` vs everything else → `prominent`). Same name, different policy.

---

## Button `.kind` → theme layer: migration (4 steps)

Principle: **look-first alongside `kind`**, then **move resolve out of SDK**, then **delete `kind`**. No big-bang.

### Step 1 — SDK: dual path (no caller changes yet)

- Add `look: Option<ButtonFamilyLook>` + builder `.look(...)`.
- `DefaultButtonTemplate`: if `Some(look)` use it; else fallback `resolve(kind, …)`.

**Exit:** One test button can ignore `kind`.

### Step 2 — Theme-dev + gallery (stop calling `.kind`)

- New `gpui-luma-theme-dev` with `primary_button`, `outline_button`, etc. (copy today’s `resolve` bodies initially).
- `themed_button_template(theme, ButtonStyle::Primary)` wrapping paint-only path.

```rust
// Before
Button::new("x").label("Submit").kind(ButtonKind::Prominent).spawn(cx);

// After
Button::new("x")
    .label("Submit")
    .template(themed_button_template(theme.clone(), ButtonStyle::Primary))
    .spawn(cx);
```

`ButtonStyle` is a **theme/app enum**, not `gpui_luma::ButtonKind`.

Migrate: button pane, icon_button, intro panels, toggle matrices first.

**Exit:** No `.kind` in those gallery files.

### Step 3 — SDK: paint-only; policy in theme

- `PaintButtonTemplate` / slim default — no `ButtonFamilyTheme` in SDK.
- `ButtonRenderModel` drops `kind` (or requires look).
- Toggle unselected-outline policy in `theme.toggle_look(...)`, not inside SDK resolve.
- `button_item_template` / `toggle_button_item_template` take theme callbacks, not `ButtonKind`.
- Relocate `LumaThemePack::button_template()` to product theme crate.

**Exit:** No `button_variant(model.kind)` in `crates/sdk/src/controls/command/button`.

### Step 4 — Public API cleanup

- Deprecate/remove `ButtonKind` from `command::button` exports.
- Move `ButtonFamilyTheme`, `DefaultButtonFamilyTheme`, `LumaPalette` action resolution to theme crates.
- Parallel track later: checkbox/switch/radio `kind` on `*Theme` traits.

```text
Step 1  SDK dual path
Step 2  theme-dev + gallery buttons     ← most caller churn
Step 3  SDK paint-only + control_group/toggle
Step 4  remove ButtonKind from public API
```

**Defer:** import lexicon tuning, golden full-file diff, studio app — until Steps 1–3 stable.

---

## Theme layer internal structure (adapters / recipes)

Inside the theme crate (not SDK):

```text
App calls theme.primary_button(...)
  → named functions (product language)
  → optional recipe / slot tables
  → palette + TOML / import / codegen
  → ButtonFamilyLook
  → gpui-luma paint template
```

**Recipe ids** (e.g. `filled-accent`, `outline-neutral`) can replace `ButtonKind` in data and codegen. Lexicon import should target **palette slots**, not SDK enum names. See [`next-step-codegen.md`](next-step-codegen.md).

**Toggle policy (explicit in theme, not SDK surprise):**

```rust
// Sketch
fn toggle_look(theme: &Theme, model: &ButtonRenderModel<bool>) -> ButtonFamilyLook {
    if model.data {
        theme.primary_button(model.role, model.size, model.state)
    } else {
        theme.outline_button(model.role, model.size, model.state)
    }
}
```

---

## Diagnosis (why Prominent/Standard failed as SDK names)

| Layer | Today | Issue |
|---|---|---|
| **Render** | `*Template` + `*Look` | Keep |
| **Resolve** | `*Theme::resolve(..., Kind/Variant)` | Policy in Rust; should be theme functions |
| **Storage** | `palette.action.{prominent,…}` | Product-flavored; belongs in theme crate |
| **App** | `.kind(ButtonKind::…)` | Forces SDK vocabulary for every label |

Renaming shadcn `primary` → `prominent` moved coupling into lexicon, gallery, and choice themes without fixing architecture.

---

## Phase B roadmap (recommended order)

| Order | Work | Notes |
|---|---|---|
| 1 | Audit `*Look` per control vs interaction states | SDK-only; use dev theme to expose gaps |
| 2 | Step 1–2 button migration | Pattern for all controls |
| 3 | `gpui-luma-theme-dev` crate | Gallery defaults here |
| 4 | Replicate to choice controls, text, menus | Same look-first pattern |
| 5 | Extract `LumaTheme` / pack / resolve out of SDK | Product theme crate + `luma-theme` output |
| 6 | Theme studio + import QA | Against stable Look + theme functions |

**Anti-goals for Phase B:**

- More `ButtonKind` variants for shadcn parity.
- More variant `match` in templates.
- Encoding `--primary` in SDK template code.
- Blocking control completion on import color accuracy.

---

## Open questions (Phase B)

1. **Crate layout** — `gpui-luma-theme-dev` + `gpui-luma-theme-shadcn` vs single `gpui-luma-theme` with features?
2. **SDK default when no look** — panic, neutral gray, or require explicit theme template?
3. **`set_active_theme_pack`** — remove from SDK or keep during migration only?
4. **Palette slot rename** — keep `action.prominent` as internal key (Option A) vs neutral `action.a` (Option B)?
5. **Recipe ids** — `&'static str` in theme vs generated enums in app only?
6. **TextFieldVariant** — same smell as `ButtonKind`; button-first or parallel?

---

## Radix catalog + properties (in progress)

Gallery product themes load from **`apps/gallery/tweakcn/*.css`** into `RadixTheme`. The SDK keeps the full token map per mode and resolves controls through declarative property mappings:

| Layer | Path | Role |
|---|---|---|
| CSS catalog | `crates/sdk/src/theme/radix/catalog/` | Parse `:root` / `.dark`; `CssTokenMap::color("input")`; `--radius` / `--font-sans` → metrics & typography |
| Control properties | `crates/sdk/src/theme/radix/` | Switch, checkbox, radio map token names → `*Look` fields (e.g. switch off: track `input`, track border `border`, thumb `background`, thumb border `border`) |
| Cached palette | `crates/sdk/src/theme/radix/palette.rs` | Derived action roles for buttons/chrome; not the only source of truth |
| Theme API | `RadixTheme::token()` / `token_color()` / `catalog()` | Introspect active mode catalog |

**Still scaffolded from native Luma:** control heights, spacing scale, elevation/shadows, button hover derivation (palette `darken()`).

---

## Code references

| Topic | Path |
|---|---|
| Radix catalog | `crates/sdk/src/theme/radix/catalog/` |
| Radix properties | `crates/sdk/src/theme/radix/` |
| Radix theme | `crates/sdk/src/theme/radix/mod.rs` |
| Button kind | `crates/sdk/src/controls/button_family/mod.rs` |
| Resolve matrix | `crates/sdk/src/controls/button_family/theme.rs` |
| Button template | `crates/sdk/src/controls/command/button/template.rs` |
| Button builder | `crates/sdk/src/controls/command/button/model.rs` |
| Pack / live theme | `crates/sdk/src/theme/pack.rs` |
| Control group kind | `crates/sdk/src/controls/control_group/button_item_template.rs` |
| Toggle template | `crates/sdk/src/controls/toggle/mod.rs` |
| Checkbox kind policy | `crates/sdk/src/controls/checkbox/theme.rs` |
| Import CLI | `crates/luma-theme` |
| Gallery theme CLI | `apps/gallery/src/gallery/theme.rs` |

---

## Doc maintenance

- [ ] Update [`next-step-theme-import.md`](next-step-theme-import.md) status row: import tooling **closed for now**; pointer here for split.
- [ ] When Phase B starts: add note to [`next-step-variants.md`](next-step-variants.md) that public ladder moves to theme functions.
- [ ] Add `next-step-theme-studio.md` when studio app is scoped.
- [ ] Add `next-step-theme-dev.md` when dev crate API is sketched.

Comments: edit **Open questions** or Phase B roadmap directly in this file.
