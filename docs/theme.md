# GPUI-Luma Theme Strategy

This document defines the next theming direction for GPUI-Luma.

The priority is to complete Luma's own theme model before importing external theme ecosystems. The
SDK needs a clear native contract for colors, radii, spacing, shadows, typography, and component
recipes. Once that contract is complete and validated in the gallery, external formats such as
shadcn/tweakcn CSS can be mapped into it deliberately.

Do not use shadcn/tweakcn as the design source of truth yet. Those themes are useful source material,
but their token names describe the shadcn component system, not Luma's SDK and templates.

## Strategy

Build theming in this order:

```text
Luma semantic theme model
  -> Luma component recipes
  -> resolved SDK appearance structs
  -> templates
  -> gallery validation
  -> Luma TOML interchange format
  -> optional importers for external ecosystems
```

Avoid this order:

```text
external theme names
  -> guessed Luma token mapping
  -> ad hoc component fixes
```

The goal is to know what Luma means first. Importers can then translate into Luma instead of
teaching Luma to imitate another toolkit's vocabulary.

## Design Goals

Luma theming should provide:

- a complete semantic palette for application chrome, surfaces, controls, states, and data accents,
- a complete metric model for radius, spacing, sizing, border widths, and focus affordances,
- shadow/elevation tokens for menus, popovers, panels, thumbs, dialogs, and other raised surfaces,
- typography tokens for font families, font sizes, line heights, weights, and label styles,
- explicit component recipes that say how each SDK theme resolver uses the semantic model,
- a native TOML interchange format that mirrors Luma's model rather than any imported ecosystem,
- public theme and template APIs that let users replace appearance without forking control behavior.

The gallery is the proving ground. It should exercise one native Luma theme with enough visual
contrast to verify every role before importer work resumes.

## Theme Layers

The theme stack should have explicit layers:

```text
LumaTheme
  -> LumaPalette
  -> LumaMetrics
  -> LumaTypography
  -> LumaElevation
  -> ComponentRecipe set
  -> control-specific Appearance structs
  -> templates
```

Templates should consume resolved appearance structs only. They should not know whether a color came
from a TOML file, Rust code, a future shadcn importer, or a hand-built theme pack.

Theme resolvers own semantic decisions such as "checked switch track uses primary action" or "menu
surface uses floating surface." Templates own GPUI structure and final style application.

## Semantic Palette

Luma should define palette roles around how Luma uses color.

Recommended first palette:

```text
app.background
app.foreground
app.muted_foreground

surface.panel.background
surface.panel.foreground
surface.panel.border

surface.floating.background
surface.floating.foreground
surface.floating.border

surface.subtle.background
surface.subtle.foreground

action.primary.background
action.primary.foreground
action.primary.hover_background
action.primary.pressed_background

action.ghost.background
action.ghost.foreground
action.ghost.hover_background
action.ghost.pressed_background

action.danger.background
action.danger.foreground
action.danger.hover_background
action.danger.pressed_background

state.hover.background
state.hover.foreground
state.pressed.background
state.selected.background
state.selected.foreground
state.disabled.background
state.disabled.foreground

form.input.background
form.input.foreground
form.input.border
form.input.placeholder

focus.ring
border.default
border.strong

navigation.background
navigation.foreground
navigation.muted_foreground
navigation.hover_background
navigation.selected_background
navigation.selected_foreground
navigation.border
navigation.focus_ring

data.accent_1
data.accent_2
data.accent_3
data.accent_4
data.accent_5
```

This shape intentionally does not use `background`, `foreground`, and `primary` by themselves. Those
names are too broad for SDK-level decisions. Luma roles should say where a value is intended to be
used.

## Metrics

Metrics should cover more than today's control height and padding.

Recommended first metric set:

```text
spacing.0
spacing.1
spacing.2
spacing.3
spacing.4
spacing.5
spacing.6

radius.none
radius.sm
radius.md
radius.lg
radius.xl
radius.pill

border.width.hairline
border.width.default
border.width.strong

focus.width
focus.offset

control.sm.height
control.sm.padding_x
control.sm.padding_y
control.sm.gap

control.md.height
control.md.padding_x
control.md.padding_y
control.md.gap

control.lg.height
control.lg.padding_x
control.lg.padding_y
control.lg.gap
```

Pill controls should be explicit. A switch, slider thumb, scrollbar thumb, or progress track may use
`radius.pill` even when ordinary controls use `radius.md`. That prevents a zero-radius theme from
accidentally breaking controls that are semantically pill-shaped.

## Shadows And Elevation

Shadow tokens should describe surface purpose, not only size.

Recommended first elevation set:

```text
shadow.none
shadow.control
shadow.thumb
shadow.menu
shadow.popover
shadow.panel
shadow.dialog
```

Each shadow token should be a list of layers:

```text
color
offset_x
offset_y
blur
spread
```

The default theme can alias several roles to the same shadow, but the theme model should not force
menus, switch thumbs, panels, and dialogs to share one hardcoded `shadow_sm()`.

Templates should not call fixed shadow helpers for themed surfaces. They should receive shadow layers
through appearance structs.

## Typography

Typography is part of theming and should be modeled directly.

Recommended first typography set:

```text
font.sans.family
font.mono.family
font.serif.family

text.body.size
text.body.line_height
text.body.weight

text.label.size
text.label.line_height
text.label.weight

text.caption.size
text.caption.line_height
text.caption.weight

text.title.size
text.title.line_height
text.title.weight

text.code.size
text.code.line_height
text.code.weight
```

Controls should use typography roles through recipes. For example, buttons and toggles likely use
`text.label`; page copy uses `text.body`; navigation section headers use `text.caption`; code-like
gallery themes can set all font families to mono without every template hardcoding that choice.

## Component Recipes

Every SDK theme resolver needs a documented recipe. This is the central missing layer.

Initial recipe table:

```text
Button default
  background: action.ghost.background
  foreground: action.ghost.foreground
  hover: action.ghost.hover_background
  pressed: action.ghost.pressed_background
  border: border.default
  radius: radius.md
  typography: text.label
  focus: focus.ring

Button primary
  background: action.primary.background
  foreground: action.primary.foreground
  hover: action.primary.hover_background
  pressed: action.primary.pressed_background
  border: action.primary.background
  radius: radius.md
  typography: text.label
  focus: focus.ring

Button destructive
  background: action.danger.background
  foreground: action.danger.foreground
  hover: action.danger.hover_background
  pressed: action.danger.pressed_background
  border: action.danger.background
  radius: radius.md
  typography: text.label
  focus: focus.ring

Icon button
  same color recipe as button variant
  radius: radius.pill or radius.md, decided by role
  size: control size height

Toggle button unselected
  same as button default

Toggle button selected
  background: state.selected.background
  foreground: state.selected.foreground
  hover: action.primary.hover_background or state.hover.background
  pressed: action.primary.pressed_background or state.pressed.background

Toggle group list
  background: surface.subtle.background
  border: border.default
  radius: radius.md

Toggle group item selected
  background: state.selected.background
  foreground: state.selected.foreground

Checkbox unchecked
  indicator background: form.input.background
  indicator border: form.input.border
  label: app.foreground
  radius: radius.sm

Checkbox checked
  indicator background: action.primary.background
  indicator foreground: action.primary.foreground
  indicator border: action.primary.background

Radio unchecked
  indicator background: form.input.background
  indicator border: form.input.border
  label: app.foreground

Radio selected
  indicator background: action.primary.background
  indicator foreground: action.primary.foreground
  indicator border: action.primary.background

Switch unchecked
  track background: form.input.background or surface.subtle.background
  track border: form.input.border
  thumb background: surface.panel.background
  thumb border: border.default
  thumb shadow: shadow.thumb
  radius: radius.pill

Switch checked
  track background: action.primary.background
  track border: action.primary.background
  thumb background: action.primary.foreground or surface.panel.background
  thumb border: transparent or action.primary.foreground
  thumb shadow: shadow.thumb
  radius: radius.pill

Slider
  track background: surface.subtle.background
  fill background: action.primary.background
  thumb background: surface.panel.background
  thumb border: action.primary.background
  thumb shadow: shadow.thumb
  radius: radius.pill

Scrollbar
  track background: transparent or surface.subtle.background
  thumb background: border.default or state.hover.background
  thumb hover: state.hover.background
  radius: radius.pill

Progress
  track background: surface.subtle.background
  fill background: action.primary.background
  radius: radius.pill

Popup menu trigger
  background: action.ghost.background
  foreground: action.ghost.foreground
  border: border.default
  radius: radius.md

Popup menu surface
  background: surface.floating.background
  foreground: surface.floating.foreground
  border: surface.floating.border
  shadow: shadow.menu
  radius: radius.lg

Popup menu item hover
  background: state.hover.background
  foreground: state.hover.foreground
  radius: radius.sm

Context menu surface
  same as popup menu surface

Tabs list
  background: surface.subtle.background
  border: border.default
  radius: radius.md

Tab active
  background: surface.panel.background or state.selected.background
  foreground: app.foreground or state.selected.foreground

Navigation sidebar
  background: navigation.background
  foreground: navigation.foreground
  section label: navigation.muted_foreground
  hover row: navigation.hover_background
  selected row: navigation.selected_background
  selected text: navigation.selected_foreground
  border: navigation.border
  focus: navigation.focus_ring
```

This table is not final, but it is the right place to make disagreements visible. If a control looks
wrong, inspect the recipe first, then the theme data, then the template.

## TOML Interchange Format

Use TOML for native Luma theme files. Do not use JSON.

TOML is easier to read and maintain for hand-authored themes, supports comments, and maps cleanly to
sectioned semantic data.

Draft shape:

```toml
name = "Luma Test"
version = 1

[light.app]
background = "hsla(0 0% 98% / 1)"
foreground = "hsla(0 0% 20% / 1)"
muted_foreground = "hsla(0 0% 55% / 1)"

[light.surface.panel]
background = "hsla(0 0% 100% / 1)"
foreground = "hsla(0 0% 20% / 1)"
border = "hsla(0 0% 88% / 1)"

[light.surface.floating]
background = "hsla(0 0% 100% / 1)"
foreground = "hsla(0 0% 20% / 1)"
border = "hsla(0 0% 88% / 1)"

[light.surface.subtle]
background = "hsla(0 0% 94% / 1)"
foreground = "hsla(0 0% 20% / 1)"

[light.action.primary]
background = "hsla(35 85% 42% / 1)"
foreground = "hsla(60 8% 98% / 1)"
hover_background = "hsla(35 85% 36% / 1)"
pressed_background = "hsla(35 85% 30% / 1)"

[light.action.ghost]
background = "hsla(0 0% 96% / 1)"
foreground = "hsla(0 0% 20% / 1)"
hover_background = "hsla(0 0% 92% / 1)"
pressed_background = "hsla(0 0% 88% / 1)"

[light.action.danger]
background = "hsla(0 74% 50% / 1)"
foreground = "hsla(0 0% 98% / 1)"
hover_background = "hsla(0 74% 44% / 1)"
pressed_background = "hsla(0 74% 38% / 1)"

[light.state.hover]
background = "hsla(0 0% 94% / 1)"
foreground = "hsla(0 0% 20% / 1)"

[light.state.pressed]
background = "hsla(0 0% 90% / 1)"

[light.state.selected]
background = "hsla(35 85% 42% / 1)"
foreground = "hsla(60 8% 98% / 1)"

[light.state.disabled]
background = "hsla(0 0% 92% / 1)"
foreground = "hsla(0 0% 60% / 1)"

[light.form.input]
background = "hsla(0 0% 100% / 1)"
foreground = "hsla(0 0% 20% / 1)"
border = "hsla(0 0% 80% / 1)"
placeholder = "hsla(0 0% 60% / 1)"

[light.focus]
ring = "hsla(35 85% 42% / 1)"

[light.border]
default = "hsla(0 0% 88% / 1)"
strong = "hsla(0 0% 70% / 1)"

[light.navigation]
background = "hsla(0 0% 98% / 1)"
foreground = "hsla(0 0% 20% / 1)"
muted_foreground = "hsla(0 0% 55% / 1)"
hover_background = "hsla(0 0% 94% / 1)"
selected_background = "hsla(35 85% 42% / 1)"
selected_foreground = "hsla(60 8% 98% / 1)"
border = "hsla(0 0% 88% / 1)"
focus_ring = "hsla(35 85% 42% / 1)"

[light.radius]
none = 0
sm = 3
md = 6
lg = 8
xl = 12
pill = 999

[light.spacing]
xs = 4
sm = 6
md = 8
lg = 12
xl = 16

[light.border_width]
hairline = 0.5
default = 1
strong = 2

[[light.shadow.menu]]
color = "hsla(0 0% 0% / 0.10)"
offset_x = 0
offset_y = 1
blur = 3
spread = 0

[[light.shadow.menu]]
color = "hsla(0 0% 0% / 0.10)"
offset_x = 0
offset_y = 1
blur = 2
spread = -1

[light.shadow]
control = []
thumb = [{ color = "hsla(0 0% 0% / 0.12)", offset_x = 0, offset_y = 1, blur = 2, spread = 0 }]
popover = [] # omitted here for brevity
panel = []
dialog = []

[light.font.sans]
family = "Inter, ui-sans-serif, sans-serif, system-ui"

[light.font.mono]
family = "Geist Mono, ui-monospace, monospace"

[light.text.body]
size = 14
line_height = 20
weight = 400

[light.text.label]
size = 13
line_height = 18
weight = 500

[light.text.caption]
size = 11
line_height = 14
weight = 500

[light.text.title]
size = 20
line_height = 28
weight = 600
```

The dark mode should mirror the same shape under `[dark.*]`. A loader can support inheritance later,
but the first complete format should require both modes to avoid ambiguous fallbacks.

## Native Theme Implementation Plan

1. Define Rust structs for `LumaTheme`, `LumaThemeMode`, palette, metrics, typography, and elevation.
2. Build one complete native Luma theme in Rust.
3. Build a gallery TOML fixture that matches the native Rust theme.
4. Implement TOML loading into the same Rust theme structs.
5. Update SDK default theme resolvers to use component recipes rather than the current small
   `ThemeTokens` bag.
6. Extend appearance structs where templates still hardcode radius, shadow, typography, or color.
7. Make the gallery render from the native Luma theme.
8. Add visual validation pages that expose every semantic role.
9. Only after this is stable, write importers from shadcn/tweakcn CSS into the Luma TOML/Rust model.

## Template Rules

Templates should:

- consume resolved appearance structs,
- apply provided colors, radii, shadows, sizes, and typography,
- attach control-supplied handlers,
- avoid hardcoded visual constants when the theme model provides a role.

Templates should not:

- parse theme files,
- inspect shadcn/tweakcn names,
- choose between semantic palette roles,
- emit semantic events,
- implement state matrices that belong in theme resolvers.

## Validation Rules

The native TOML loader should return errors for:

- missing required light-mode fields,
- missing required dark-mode fields,
- unsupported color syntax,
- invalid numeric units or negative values where not allowed,
- non-finite numeric values,
- unknown component recipe references,
- shadow layers with invalid color or metric values,
- typography roles with missing family, size, line height, or weight.

The loader may warn for:

- very low contrast between foreground and background pairs,
- unusably small control sizes,
- radius values that are inconsistent with their scale,
- shadows with fully transparent layers,
- unknown fields reserved for future format versions.

## External Importers

External importers are future work.

When shadcn/tweakcn import resumes, it should be implemented as:

```text
shadcn/tweakcn CSS
  -> imported shadcn palette
  -> explicit shadcn-to-Luma mapping
  -> LumaTheme
```

It should not map directly into control appearance structs. It should not teach templates about
external token names. The importer should be tested against the Luma theme contract, not against
visual guesses from isolated controls.

## Open Questions

- Should selected navigation use `navigation.selected_*`, `state.selected_*`, or
  `action.primary_*` by default?
- Should switch thumbs use `action.primary.foreground` or `surface.panel.background` when checked?
- Should default buttons be ghost-filled, panel-filled, or outline-style?
- Which controls are semantically pill-shaped and should always use `radius.pill`?
- How many shadow roles does the SDK need before themes become too granular?
- Should component recipes be fully configurable in TOML, or should the first TOML format only define
  semantic inputs and use SDK-owned recipes?
- Should fonts be mode-specific, or shared across modes with optional overrides?
- When should the default theme move from SDK internals into a separate theme pack?
