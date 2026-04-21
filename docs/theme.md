# GPUI-Luma Theme Design Notes

This document tracks the path toward data-driven themes for GPUI-Luma. The immediate goal is to make
the gallery consume real theme data without committing the SDK to a final import format too early.
The longer-term goal is easy access to theme ecosystems without closing off custom theme and template
libraries.

## Design Goals

The first goal is easy access to themes. Luma should make it practical to start from existing theme
sources instead of hand-picking palettes. A shadcn-style theme should be usable source material: parse
or convert it once, normalize it into Luma's semantic JSON shape, and build the runtime theme objects
the SDK needs.

The second goal is to keep custom theme and template packs viable. Nothing in control design should
assume the SDK's default visual structure is the only visual structure. A user should be able to
ship:

- a data-backed color and metric theme,
- a Rust theme pack that implements Luma theme traits directly,
- a template pack that replaces GPUI structure for some or all controls,
- a combined theme plus template pack that keeps its own design language.

That means each control needs a rich public face: semantic render models, public template traits,
public themed template constructors, public theme traits, and builder hooks for replacing templates.
Controls should expose enough state for templates to make strong visual decisions, while still
keeping behavior and event emission inside the control.

The third goal is to avoid premature packaging. The SDK themes currently live near the controls on
purpose. While control behavior, render models, templates, and theme appearance contracts are still
being validated, it is easier to confirm a control in one folder than to split the default theme into
a separate "dev theme" package. A separate default theme crate or SDK theme pack may make sense
later, but it is not the next step.

## Current Direction

The SDK keeps the existing separation:

- controls own behavior,
- templates own GPUI structure,
- themes resolve semantic state into appearance,
- tokens are data inputs to those themes.

`ThemeTokens` remains the active token set used by the default control themes. Light and dark mode
support starts one layer above that with `ThemeMode` and `ThemeModes`: a theme can carry both modes,
and the application or future theme pack chooses which mode becomes the active `ThemeTokens`.

That gives the SDK a light/dark model now without forcing every control theme to become mode-aware in
the same change.

The gallery is the proving ground for the next layer. It can carry a `GalleryThemePack`, a selected
mode, imported theme data, and app-chrome tokens before those concepts become SDK API. Once the
integration shape is clear, the generic pieces can migrate into the SDK.

## Why Data-Driven Themes

Picking colors by hand is not the point of this project. There are many existing theme ecosystems,
especially shadcn-style themes, that already define coherent semantic roles such as background,
foreground, primary, muted, accent, border, ring, and sidebar colors.

Luma should be able to consume those themes through a converter:

```text
source theme
  -> normalized JSON theme spec
  -> Luma converter
  -> ThemeModes
  -> active ThemeTokens
  -> ThemePack / TemplatePack
```

The SDK should not need to parse arbitrary CSS as its core theme format. CSS import can be a separate
adapter that writes the normalized JSON shape.

The normalized JSON should be stable and ecosystem-friendly. It should not mirror today's
`ThemeTokens` exactly, because `ThemeTokens` is the current control-theme input, not necessarily the
durable file format.

## First JSON Shape

The first JSON format should describe semantic design tokens, not SDK trait objects and not today's
`ThemeTokens` struct directly.

```json
{
  "name": "shadcn-neutral",
  "source": "shadcn",
  "base_font_px": 16,
  "modes": {
    "light": {
      "background": "hsl(223.8136 0% 100%)",
      "foreground": "hsl(223.8136 0% 3.9388%)",
      "card": "hsl(223.8136 0.0011% 99.3410%)",
      "popover": "hsl(223.8136 0% 100%)",
      "primary": "hsl(223.8136 0% 9.0527%)",
      "primary_foreground": "hsl(223.8136 0.0004% 98.0256%)",
      "secondary": "hsl(223.8136 0% 83.1444%)",
      "secondary_foreground": "hsl(223.8136 0% 3.9388%)",
      "muted": "hsl(223.8136 0.0001% 93.4480%)",
      "muted_foreground": "hsl(223.8136 0% 45.1519%)",
      "accent": "hsl(223.8136 0.0001% 93.4480%)",
      "accent_foreground": "hsl(223.8136 0% 9.0527%)",
      "destructive": "hsl(351.7303 100% 40.5257%)",
      "destructive_foreground": "hsl(223.8136 0% 100%)",
      "border": "hsl(223.8136 0.0001% 93.4480%)",
      "input": "hsl(223.8136 0.0002% 96.0587%)",
      "ring": "hsl(223.8136 0% 63.0163%)",
      "sidebar": "hsl(223.8136 0.0004% 98.0256%)",
      "sidebar_foreground": "hsl(223.8136 0% 3.9388%)",
      "sidebar_accent": "hsl(223.8136 0.0002% 96.0587%)",
      "sidebar_accent_foreground": "hsl(223.8136 0% 9.0527%)",
      "sidebar_border": "hsl(223.8136 0.0001% 89.8161%)",
      "sidebar_ring": "hsl(223.8136 0% 63.0163%)"
    },
    "dark": {
      "background": "hsl(223.8136 0% 9.0527%)",
      "foreground": "hsl(223.8136 0.0004% 98.0256%)"
    }
  },
  "radius": {
    "base": "1rem"
  },
  "spacing": {
    "base": "0.25rem"
  },
  "fonts": {
    "sans": "DM Sans, ui-sans-serif, sans-serif, system-ui",
    "mono": "Geist Mono, ui-monospace, monospace"
  }
}
```

The `dark` object above is intentionally abbreviated. Real theme files should provide the same
required mode fields for light and dark.

## Initial Mapping

The first converter can be opinionated. It should map semantic theme data into the current control
token surface and derive missing interaction states when the source theme does not provide them.

```text
secondary or input       -> surface
accent                   -> surface_hover
muted                    -> surface_pressed
muted                    -> surface_disabled
primary                  -> primary
primary derived states   -> primary_hover, primary_pressed
destructive              -> destructive
destructive derived      -> destructive_hover, destructive_pressed
primary                  -> selected
primary derived states   -> selected_hover, selected_pressed
foreground               -> text
primary_foreground       -> text_inverse
muted_foreground         -> text_disabled
border                   -> border
ring                     -> focus_ring
radius.base              -> control radius scale
spacing.base             -> gap and padding scale
popover                  -> menu surfaces in control-specific themes
sidebar tokens           -> gallery navigation templates
```

Derived hover and pressed colors should be generated by small HSL lightness adjustments at first.
The exact derivation can be replaced later by explicit per-control override fields.

## First Implementation Steps

1. Keep `ThemeTokens` as the control-theme input and use `ThemeModes` to hold light and dark token
   sets.
2. Build a gallery-local `GalleryThemePack` from one selected `ThemeMode`.
3. Thread the selected gallery theme pack through pane construction and apply templates through each
   control builder.
4. Move gallery-specific sidebar templates behind the gallery theme pack so sidebar colors can use
   imported `sidebar_*` tokens.
5. Add a gallery-local JSON spec and converter under `apps/gallery/src/gallery/theme/`.
6. Store supplied shadcn-derived JSON files under `apps/gallery/themes/`.
7. Keep custom structural templates, such as context menu variants, as template-pack experiments
   rather than forcing them into the SDK immediately.
8. Review the friction from applying the gallery theme pack before deciding whether to migrate
   `ThemePack`, `TemplatePack`, both, or neither into the SDK.

## Public Control Surface

Theme and template extensibility depends on the public surface of every control. New controls should
continue to provide:

- a typed builder with `.template(...)`,
- a narrow render model that carries semantic state rather than resolved appearance,
- a public template trait,
- a default themed template,
- a public theme trait when the control has distinct appearance policy,
- appearance structs that are rich enough for a custom template to avoid duplicating control logic,
- handler bundles for composite controls whose templates own child surfaces.

The control should not emit semantic events from templates. Templates can attach control-supplied
handlers, but behavior stays in the control entity.

This is the main guardrail for user-owned theme and template packs: the SDK should not require users
to fork control behavior just to get a different visual language.

## Packaging Notes

Do not split the existing SDK default themes into a separate "dev theme" package yet. The default
theme is still part of control development feedback. Keeping the model, control, template, and nearby
theme resolver easy to inspect helps validate whether a control does what it should.

A future packaging split is still possible. Reasonable future shapes include:

- `gpui-luma`: controls, public traits, default templates, and a default theme pack,
- `gpui-luma-themes`: optional shipped theme packs,
- user crates that export `ThemePack` and `TemplatePack` values,
- tooling that converts source themes into normalized Luma JSON.

The near-term priority is to keep the contracts clean enough that any of those package shapes remain
possible.

## Validation Rules

The converter should return errors for:

- missing required light-mode tokens,
- missing required dark-mode tokens when a dark mode is declared,
- unsupported color syntax,
- invalid numeric units for radius or spacing,
- non-finite metric values.

The converter should normalize or warn for:

- HSL saturation and lightness outside normal percentage bounds,
- unusably small control metrics,
- low contrast between text and surfaces,
- unknown tokens from source ecosystems.

HSL input should accept common web forms such as `hsl(210 10% 96%)`,
`hsl(210deg 10% 96% / 0.8)`, and `hsl(210, 10%, 96%)`. Values should be normalized into GPUI's
`Hsla` representation.

## Open Design Questions

- Does the SDK need a first-class `ThemePack`, a `TemplatePack`, or both?
- Should builders continue to accept per-control templates only, or should there be a broader style
  context?
- Which tokens belong in SDK control themes versus application chrome?
- How much per-control override data is useful before the format starts becoming a CSS engine?
- Should CSS-to-JSON conversion live in the SDK, a tool, or examples only?
- When, if ever, should the SDK default theme move out of the control/theme implementation area?
