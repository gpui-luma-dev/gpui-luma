# shadcn / tweakcn CSS variables → control parts

Reference for mapping tweakcn/shadcn `:root` / `.dark` custom properties to GPUI-Luma control appearance. Radix UI documents component behavior, not this token layer; shadcn implies it via Tailwind classes (`bg-primary`, `border-input`) but does not tabulate it.

**Living source of truth in code:** `crates/sdk/src/theme/radix/usage.rs` (`all_radix_theme_usages()`). Gallery **Theme Usage** and per-pane **Theme Parts** sidebars render the same data against the active tweakcn CSS.

**Related:**

| Doc / path | Role |
|---|---|
| [`plan-theme.md`](plan-theme.md) | Theme roadmap and gallery wiring |
| [`fundamental-templates.md`](fundamental-templates.md) | Appearance-only contract, CSS import path |
| `apps/gallery/tweakcn/*.css` | Sample tweakcn theme exports |
| `crates/sdk/src/theme/radix/*.rs` | Per-control resolver tables (module doc comments) |

---

## Mental model

shadcn themes define **semantic slots**, not literal “border color” or “background color” per control. Each slot is reused across many components.

| Slot | Typical role |
|------|----------------|
| `--background` / `--foreground` | Page surface and default text |
| `--card` / `--card-foreground` | Raised panels; outline button fill |
| `--popover` / `--popover-foreground` | Dropdowns, menus, floating surfaces |
| `--primary` / `--primary-foreground` | Brand actions; selected / checked states |
| `--secondary` / `--secondary-foreground` | Secondary actions |
| `--muted` / `--muted-foreground` | Disabled fills, subtle surfaces, placeholder text |
| `--accent` / `--accent-foreground` | Hover highlights (ghost buttons, menu rows) |
| `--destructive` / `--destructive-foreground` | Danger actions (not wired in gallery yet) |
| `--border` | Generic borders, dividers, scrollbar thumb |
| `--input` | Form control borders; switch off-track |
| `--ring` | Focus rings |
| `--sidebar-*` | App sidebar only |

### `--border` vs `--input`

These are the most commonly confused pair:

- **`--input`** — editable control chrome: text field border, checkbox/radio ring, listbox border, switch off-track.
- **`--border`** — structural borders: panels, menus, switch track outline, slider track.

Many tweakcn exports set both to the same HSL value, which makes visual validation hard. When they alias, only the resolver (or Theme Parts sidebar) tells you which token drives a part.

---

## Token glossary

| CSS var | Used for |
|---------|----------|
| `--background` | App/page bg; text field bg; selector trigger bg; switch thumb (off); slider thumb |
| `--foreground` | Body text; outline/ghost button label |
| `--card` | Panel bg fallback; outline button bg (default) |
| `--card-foreground` | Text on card surfaces |
| `--popover` | Menu / dropdown / panel surfaces |
| `--popover-foreground` | Text and icons on popover surfaces |
| `--primary` | Primary button fill; checked checkbox; selected radio; switch on-track; slider fill; active tab; progress fill; text selection |
| `--primary-foreground` | Text/icon on primary surfaces; switch on-thumb |
| `--secondary` | Secondary button fill |
| `--secondary-foreground` | Text on secondary button |
| `--muted` | Disabled fills; control group bg; progress track; ghost/outline pressed |
| `--muted-foreground` | Placeholders; disabled text; sidebar section labels |
| `--accent` | Hover bg (ghost button, menu items, listbox rows) |
| `--accent-foreground` | Hover text on accent bg |
| `--border` | Generic borders; switch track border; slider track; menu borders |
| `--input` | Input borders; unchecked checkbox/radio ring; listbox border; switch off-track |
| `--ring` | Focus rings (most focusable controls) |
| `--sidebar` | Sidebar container bg |
| `--sidebar-foreground` | Sidebar item text |
| `--sidebar-primary` | Selected sidebar item bg |
| `--sidebar-primary-foreground` | Selected sidebar item text |
| `--sidebar-accent` | Sidebar item hover bg |
| `--sidebar-accent-foreground` | Sidebar item hover text |
| `--sidebar-border` | Sidebar container border |
| `--sidebar-ring` | Sidebar focus ring |

---

## Buttons

Applies to **Button**, **Icon Button**, and **Toggle**.

| Style | Background | Foreground | Border | Hover bg | Focus |
|-------|------------|------------|--------|----------|-------|
| **Primary** | `--primary` | `--primary-foreground` | `--primary` | darkened `--primary` | `--ring` |
| **Secondary** | `--secondary` | `--secondary-foreground` | `--secondary` | darkened `--secondary` | `--ring` |
| **Outline** | `--card` → `--background` | `--foreground` | `--border` | `--accent` → `--muted` | `--ring` |
| **Ghost** | transparent | `--foreground` | — | `--accent` → `--muted` | `--ring` |
| **Disabled** (all) | `--muted` | `--muted-foreground` | — | — | — |

Filled roles use `{style}` / `{style}-foreground` (e.g. primary → `--primary`, `--primary-foreground`). Outline and ghost label tokens resolve to `--foreground`.

---

## Form inputs

**TextField**, **TextArea**

| Part | CSS var |
|------|---------|
| Background | `--background` |
| Border | `--input` |
| Text | `--foreground` |
| Placeholder | `--muted-foreground` |
| Text selection | `--primary` |
| Focus ring | `--ring` |
| Ghost variant hover | `--accent` |

---

## Checkbox

| State | Fill / ring | Mark |
|-------|-------------|------|
| Unchecked | `--input` border | — |
| Checked | `--primary` (or `--secondary` for secondary style) | `--primary-foreground` |
| Disabled | `--muted` | `--muted-foreground` |
| Focus | `--ring` | — |

---

## Radio button

| State | Ring | Dot |
|-------|------|-----|
| Unselected | `--input` border | — |
| Selected | `--primary` fill | `--primary-foreground` |
| Disabled | `--muted` | `--muted-foreground` |
| Focus | `--ring` | — |

---

## Switch

| State | Track bg | Track border | Thumb bg | Thumb border |
|-------|----------|--------------|----------|--------------|
| **Off** | `--input` | `--border` | `--background` | `--border` |
| **On** | `--primary` or `--secondary` | same as track | `--primary-foreground` | same as thumb |
| **Disabled** | `--muted` | `--border` | `--muted-foreground` | `--muted` |
| **Focus** | — | — | — | adorner: `--ring` |

On-state track uses the control style token (`primary` / `secondary`); thumb uses matching `{style}-foreground`.

---

## Slider

| Part | CSS var |
|------|---------|
| Track (rail) | `--border` |
| Fill | `--primary` |
| Thumb body | `--background` → `--card` |
| Thumb border / ring | `--primary` |
| Focus ring | `--ring` |

Track uses `--border` rather than `--muted` because many tweakcn light themes set `--muted` near white, which disappears on card panels.

---

## Scrollbar

| Part | CSS var |
|------|---------|
| Track | transparent / `--background` |
| Thumb | `--border` |

---

## Floating surfaces

Shared pattern for **Floating Menu**, **ComboBox** panel, **Selector** panel, **Context Menu**, **Selection Panel**, **Autocomplete** panel.

| Part | CSS var | Fallback chain |
|------|---------|----------------|
| Surface bg | `--popover` | → `--card` → `--background` |
| Surface text | `--popover-foreground` | → `--foreground` |
| Border | `--border` | — |
| Item hover bg | `--accent` | → `--muted` |
| Item hover text | `--accent-foreground` | → `--foreground` |
| Disabled item | `--muted-foreground` | — |

**Popup Menu** trigger follows the ghost button pattern (`--accent` on hover).

**Selector / ComboBox** trigger follows the text field pattern (`--background` + `--input` border).

---

## ListBox

| Part | CSS var |
|------|---------|
| List bg | `--background` |
| List border | `--input` |
| Row hover | `--accent` |
| Focus ring | `--ring` |

---

## ListView

| Part | CSS var |
|------|---------|
| List bg | `--background` |
| List border | `--input` |
| Header bg | `--muted` |
| Header label | `--muted-foreground` |
| Row hover / keyboard active | `--muted` |
| Selected row | `--muted` |
| Row divider | `--border` |

Row hover uses `--muted` (not `--accent`) so table highlights stay neutral in tweakcn themes where accent is a chart/highlight color.

**GPUI clipping:** `overflow_hidden` is rectangular only. The list shell uses `controls::rounded_shell::rounded_bordered_panel` (`rounded` + `bg` + `border` on one node). Header strips need `.rounded_tl` / `.rounded_tr` on the same node as `header_background`, or corners show square fills past the border.

---

## Control group / toggle group

| Part | CSS var |
|------|---------|
| Container bg | `--muted` |
| Container border | `--border` |

---

## Tabs navigation

| Part | CSS var |
|------|---------|
| Inactive label | `--foreground` |
| Active label | `--primary` |
| Active indicator | `--primary` |
| Disabled label | `--muted-foreground` |
| Focus ring | `--ring` |

---

## Navigation sidebar

| Part | CSS var | Fallback |
|------|---------|----------|
| Container bg | `--sidebar` | → `--card` → `--background` |
| Item text | `--sidebar-foreground` | — |
| Item hover bg | `--sidebar-accent` | — |
| Selected item bg | `--sidebar-primary` | — |
| Selected item text | `--sidebar-primary-foreground` | — |
| Container border | `--sidebar-border` | — |
| Focus ring | `--sidebar-ring` | — |
| Section labels | `--muted-foreground` | — |

---

## Progress

| Part | CSS var |
|------|---------|
| Track | `--muted` |
| Fill | `--primary` |

---

## App chrome

Gallery shell and page chrome (not a single control).

| Part | CSS var |
|------|---------|
| Page bg | `--background` |
| Body text | `--foreground` |
| Muted / secondary text | `--muted-foreground` |
| Panel bg | `--card` → `--background` |
| Dividers | `--border` |

---

## Non-color tokens

Present in tweakcn CSS exports; partial GPUI wiring today.

| CSS var | shadcn meaning | GPUI status |
|---------|----------------|-------------|
| `--radius` | Border radius scale | Partially via metrics |
| `--font-sans` / `--font-mono` | Typography | Font loading (e.g. jarvis) |
| `--shadow-*` | Elevation | Not fully wired |
| `--chart-*` | Charts only | Unused in controls |
| `--spacing` | Tailwind spacing | Unused in controls |

---

## Fallback chains

When a token does not seem to apply, check resolver fallbacks in `crates/sdk/src/theme/radix/resolve.rs`:

```
popover bg:     popover → card → background
popover fg:     popover-foreground → foreground
outline btn bg: card → background
hover accent:   accent → muted
panel bg:       card → background
sidebar bg:     sidebar → card → background
slider thumb:   background → card
```

If `--popover`, `--card`, and `--background` share the same value, surfaces look identical regardless of which token the resolver chose.

---

## Where to look in the repo

| Need | Location |
|------|----------|
| Full usage registry | `crates/sdk/src/theme/radix/usage.rs` |
| Palette → token mapping | `crates/sdk/src/theme/radix/palette.rs` |
| Shared resolve helpers | `crates/sdk/src/theme/radix/resolve.rs` |
| Per-control tables | Module doc comments in `crates/sdk/src/theme/radix/{button,checkbox,switch,textfield,...}.rs` |
| Sample CSS themes | `apps/gallery/tweakcn/*.css` |
| Live introspection | Gallery → **Theme Usage** pane; per-control **Theme Parts** sidebar |

---

## External docs (what they do *not* cover)

| Source | Documents |
|--------|-----------|
| [radix-ui.com](https://radix-ui.com) | Component behavior, a11y, unstyled parts — no CSS var mapping |
| [ui.shadcn.com](https://ui.shadcn.com) | Component source with Tailwind classes — vars implied, not tabulated |
| [tweakcn.com](https://tweakcn.com) | Visual editor for `:root` / `.dark` — no per-control legend |

The CSS var → control part contract lives in application code. This doc and `usage.rs` are the reference shadcn does not publish.
