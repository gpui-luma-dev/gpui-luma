# Use Compact HSLA for Luma Studio Color Displays

## Summary

Replace user-visible RGB-family color specifications in Luma Studio with compact HSLA notation,
and make HSLA the ordinary internal color representation at the app/theme/SDK boundaries.

The target display format is:

```text
hsla(210 50% 40% / 0.75)
```

This issue is about both color presentation and the ordinary color representation used by the Luma
Studio app and its theme-facing code. It is not a request to remove explicit RGB interpolation
demos, channel-specific color models, or to introduce OKLCH as a runtime color pipeline.

OKLCH is deferred as the primary color model. GPUI's practical rendering boundary is still
sRGB-oriented, and introducing OKLCH as the default would expand the scope into color-space
conversion, interpolation, gamut handling, picker behavior, and display semantics. HSLA is the
compact, readable notation and ordinary internal representation to standardize on for this pass.

## Problem

Luma Studio currently presents color values through several formats:

- compact HSLA in some color exposition helpers;
- hexadecimal values in color readouts and inspectors;
- RGB/RGBA values in the shadow-button prototype;
- RGB/RGBA labels in color slider demonstrations;
- RGB-backed `ColorSpecification` models for channel controls and interpolation;
- HSL, HSV, Lab, and OKLCH values in color-space demonstrations.

The result is inconsistent. A user inspecting the same `gpui::Hsla` value may see different
representations depending on which page or prototype is active. Some displays also expose the
implementation's conversion to RGB rather than the representation used by the active theme and
GPUI look boundary.

The app needs one compact, predictable color readout format for ordinary displayed color values,
and one ordinary color type for values crossing between app state, theme resolution, and GPUI
painting.

## Goals

- Use compact HSLA notation for ordinary color values displayed by Luma Studio.
- Use `gpui::Hsla` for ordinary color values stored in Luma Studio app state and passed through
  theme-facing APIs.
- Keep RGB/RGBA conversion at explicit boundaries: color-space demonstrations, compatibility
  parsers, or the final conversion performed by GPUI's rendering pipeline.
- Standardize rounding and alpha formatting in one reusable formatter.
- Remove RGB/RGBA/hex color values from ordinary readout rows, status text, helper text, and
  preview summaries where the value is presented as a color specification.
- Update visible color input guidance to use HSLA notation.
- Keep ordinary color values represented internally as `gpui::Hsla`; retain typed alternate color
  specifications only where they are the subject of an explicit color-space control or algorithm.
- Preserve RGB interpolation and RGB channel demonstrations where they are intentionally teaching
  color-space behavior.
- Preserve HSL, HSV, Lab, and OKLCH demos as separate color-space experiments where their labels
  describe the algorithm or control being demonstrated.
- Make color display formatting reusable by the color field, slider, composition, shadow-button,
  palette/inspector, and theme-usage surfaces.
- Defer OKLCH adoption without blocking future addition of an explicit color-space display layer.
- Keep OKLCH available as a future advanced color-space tool or palette-generation algorithm, with
  results converted back to `gpui::Hsla` at the SDK/GPUI boundary.

## Non-goals

- Do not change GPUI's `Hsla` type or existing SDK Hsla-facing theme contracts.
- Do not claim that HSLA is the only color format supported by GPUI. GPUI also exposes RGBA and RGB
  conversion helpers; the objective is to use Hsla as the ordinary application/theme/painting
  boundary, not to remove those lower-level capabilities.
- Do not change the mathematical behavior of RGB interpolation.
- Do not remove `RgbaSpec`, `ColorInterpolation::Rgb`, or RGB channel delegates from SDK color
  controls.
- Do not convert all color-space demos to HSLA. A demo labeled `RGB`, `HSV`, `Lab`, or `OKLCH` may
  continue to use that label when it identifies the control's actual color-space algorithm.
- Do not introduce OKLCH rendering, OKLCH gamut mapping, or OKLCH persistence.
- Do not make OKLCH the default SDK color model, default color picker, or default developer-facing
  notation.
- Do not change Shadcn CSS parsing or token semantics.
- Do not redesign Luma Studio's color controls beyond their displayed notation and related input
  guidance.

## Internal color representation objective

This issue has an internal representation objective in addition to the display objective.

`gpui::Hsla` is a first-class GPUI color type, not merely a Luma Studio formatting convention:

- GPUI's styled APIs accept `Hsla` for text color, backgrounds, borders, fills, highlights, and
  related paint inputs.
- GPUI defines the `Hsla` struct and `hsla(...)` constructor in its color module.
- GPUI provides conversion between `Hsla` and `Rgba`; the rendering path can perform that
  conversion when needed.
- The SDK's theme and look structures already use `Hsla` pervasively for control backgrounds,
  foregrounds, borders, focus rings, shadows, and other visual values.

Therefore, the ordinary color pipeline should be:

```text
theme/app input
  → gpui::Hsla
  → SDK/look theme values
  → GPUI styling and paint APIs
```

RGB/RGBA should remain only at explicit boundaries:

- an RGB interpolation or channel-control demonstration;
- a typed `RgbaSpec`/RGB color-space model whose purpose is to expose that model;
- compatibility parsing for an existing user input format;
- conversion performed by GPUI or a deliberate color-space algorithm.

This avoids repeatedly converting ordinary theme colors to RGB for storage or display and then
converting them back to Hsla for GPUI painting.

The objective is not to make HSLA a more capable rendering space than it is. HSLA remains a
practical representation built around GPUI's existing color API and the expectations of developer
users. Developers are more likely to benefit from familiar CSS-like HSL notation, predictable
theme-token debugging, and copyable values than from a perceptually uniform but less familiar
OKLCH picker.

OKLCH can be added later in a deliberately bounded role:

- an optional advanced color-space experiment;
- a palette-generation or lightness-ramp algorithm;
- accessibility or contrast candidate generation;
- a designer-oriented tool that explicitly handles gamut and conversion behavior.

Those features should produce ordinary `gpui::Hsla` values for the SDK and should not require most
developers to understand OKLCH channels or out-of-gamut behavior.

## Display contract

### Canonical ordinary color display

Any ordinary displayed `Hsla` color value should use:

```text
hsla(<hue-degrees> <saturation>% <lightness>% / <alpha>)
```

Examples:

```text
hsla(0 100% 50% / 1)
hsla(210 50% 40% / 0.75)
hsla(0 0% 50% / 0.5)
```

Formatting rules:

- Hue is expressed in degrees and rounded to the existing compact whole-number policy.
- Saturation and lightness are expressed as percentages and rounded to the existing compact
  whole-number policy unless a specific diagnostic surface requires more precision.
- Alpha is clamped to `0..1`, rounded to at most two decimal places, and omits unnecessary zeroes.
- Hue should remain normalized to the expected display range rather than exposing raw GPUI
  normalized storage.
- The formatter should be deterministic and safe for values at the edges of the valid range.

The existing `format_compact_hsla` helpers are close to this contract and should become the
canonical implementation instead of remaining duplicated in multiple modules.

### Hexadecimal display

Hexadecimal is an sRGB/RGB-family representation. If the goal is to eliminate RGB-family color
specifications from the app display, ordinary `Hex` readout rows should be replaced by the
canonical HSLA row rather than displayed alongside it.

If a later product requirement needs copyable web/CSS export, that should be an explicit export
surface with its own format choice. It should not make Hex the default inspection representation.

### RGB algorithm labels

The following are not ordinary color readouts and should not be mechanically renamed:

- `RGB` interpolation, when it demonstrates interpolation in RGB space;
- `RGBA` channel delegates, when they demonstrate the `RgbaSpec` model;
- RGB/HSL/HSV/Lab/OKLCH comparison cards, when the label identifies the algorithm under test.

Those controls may still show their current channel names because removing the labels would make
the color-space demonstrations inaccurate. Their selected color summaries should use compact HSLA
when the summary is a general color value rather than a channel-specific diagnostic.

## Current implementation findings

The app already has several compact HSLA implementations:

- `color_exposition_common.rs` provides `format_compact_hsla` for shared color exposition;
- `color_slider.rs` has a local duplicate;
- `shadow_button/button.rs` has a local duplicate of HSLA and RGB conversion helpers;
- `panels/theme_usage.rs` has another local compact HSLA formatter.

RGB-family display and guidance still exists in several places:

- shadow-button color help displays an RGB triplet for the theme default and current value;
- shadow-button input guidance asks for RGB or hex values;
- shadow-button CSS output is formatted as `rgba(...)`;
- color exposition and inspector surfaces still show Hex rows;
- some sample and prototype surfaces use literal `rgb(...)` colors for visual decoration;
- color slider demos intentionally construct `RgbaSpec` and RGB interpolation controls.

The implementation must distinguish user-visible color specifications from literal colors used to
paint a sample. A literal `rgb(...)` used only to create a decorative sample is not necessarily a
display-format issue, though it should be reviewed for consistency with the broader lookless/app
composition rules.

## Prep work

### 1. Build a display inventory

Search all `apps/luma-studio/src` modules for:

- `rgb(` and `rgba(` strings;
- `format_rgb`, `rgb_triplet`, `hsla_to_rgb8`, and hex formatters;
- `format_compact_hsla` definitions and call sites;
- labels and helper text containing `RGB`, `RGBA`, `Hex`, or `#hex`;
- direct `Hsla` interpolation into user-visible strings;
- inspector rows that render color values.

Classify each occurrence as one of:

1. **Ordinary color display** — must use compact HSLA.
2. **Color input guidance** — should teach/accept the chosen canonical notation.
3. **Algorithm or channel demonstration** — preserve the RGB-family label where technically
   meaningful, but use HSLA for general selected-color summaries.
4. **Paint-only sample literal** — not a display format, but review for app-level styling policy.
5. **Parsing/compatibility code** — not changed unless the implementation explicitly chooses to
   migrate input acceptance.

This classification prevents a presentation change from accidentally deleting the purpose of the
RGB interpolation and RGBA channel demos.

### 2. Build an internal representation inventory

In addition to display strings, inspect Luma Studio state and helper APIs for values that are
converted to RGB and immediately converted back to `Hsla`, or that store ordinary theme colors as
RGB triplets or hex integers.

Classify those occurrences as:

1. **Ordinary app/theme color** — migrate storage and handoff to `gpui::Hsla`.
2. **Explicit RGB/RGBA color-space model** — retain the typed model and convert to Hsla only at the
   GPUI/theme boundary.
3. **Parser compatibility** — retain only if compatibility is intentionally supported; normalize
   the parsed result to Hsla immediately.
4. **Paint-only literal** — review separately; do not convert merely for representation purity.

The migration should eliminate unnecessary RGB detours in ordinary color state without weakening
the color-space demonstrations.

### 3. Establish one formatter owner

Select a generic color-formatting module that does not live under the Cards feature. Existing
generic color exposition code should not depend on `content_tabs::cards::common` merely to format
colors.

The formatter should expose a small API such as:

```rust
pub(crate) fn format_compact_hsla(color: Hsla) -> String;
```

If Hex remains needed for an explicit export or compatibility surface, keep it as a separately
named formatter rather than treating it as the default color display.

### 4. Define precision and edge-case behavior

Before replacing call sites, confirm behavior for:

- hue values near `0` and `360` degrees;
- grayscale colors with zero saturation;
- transparent colors;
- alpha values of `0`, `1`, and fractional values;
- values slightly outside valid ranges due to calculations;
- colors produced by RGB, HSV, Lab, and OKLCH conversion paths.

The formatter should clamp only for presentation. It must not mutate the underlying color.

### 5. Decide input compatibility

The request concerns app display, but the shadow-button prototype currently accepts RGB and hex
input. Decide whether the first implementation should:

- only change the visible placeholder/help and output while retaining RGB/hex parsing; or
- make HSLA the canonical input and parser format as well, with optional compatibility parsing.

The recommended initial choice is to make HSLA the canonical visible input format while retaining
existing RGB/hex parsing only if it is inexpensive and clearly documented as compatibility. The
stored value and all output should still be HSLA.

## Implementation plan

### Phase 1: Add the canonical formatter

- Consolidate the duplicated compact HSLA implementations.
- Define the canonical notation and precision in one module.
- Add unit tests for hue, saturation, lightness, alpha, grayscale, transparent, and boundary
  values.
- Ensure no formatter exposes normalized `0..1` hue/saturation/lightness values directly.

### Phase 2: Migrate shared color exposition

- Update color field, color slider, color arc, color ring, color picker, and composition readouts.
- Replace ordinary Hex/HSLA pairs with the canonical HSLA readout where the objective is to remove
  RGB-family display formats.
- Update event summaries that currently print a color specification in RGB-family notation.
- Keep channel-specific labels for controls that intentionally demonstrate RGB, HSL, HSV, Lab, or
  OKLCH behavior.
- Ensure compact HSLA text fits the existing detail rows and narrow composition cards.

### Phase 3: Migrate shadow-button display and guidance

- Replace the shadow-button theme-default and current-value RGB triplet readouts with compact
  HSLA.
- Replace `rgba(...)` CSS-like display output with `hsla(...)` notation for the displayed color
  specification, while preserving the box-shadow numeric fields.
- Update the color input placeholder/help text to show the canonical HSLA form.
- Decide and document whether RGB/hex parsing remains as compatibility input.
- Update invalid-input messages to describe the accepted syntax accurately.
- Keep conversion to GPUI `Hsla` as the internal result.

### Phase 4: Normalize ordinary internal color state

- Identify ordinary Luma Studio fields, models, and helper return values that store colors as RGB
  triplets, packed RGB integers, or Hex strings.
- Change those ordinary color values to `gpui::Hsla` where they cross app, theme, or GPUI styling
  boundaries.
- Keep `RgbaSpec` and other alternate specifications local to the color-space demos that need
  them.
- Convert parser results to Hsla immediately after parsing.
- Remove unnecessary `Hsla → RGB → Hsla` round trips in ordinary display and theme paths.
- Ensure grouping, equality, overrides, and theme synchronization use the typed Hsla value rather
  than a formatted RGB/Hex string.

### Phase 5: Migrate inspectors and theme surfaces

- Replace ordinary Hex value rendering in control inspectors with compact HSLA.
- Replace ordinary Hex value rendering in palette/theme-related displays if those surfaces remain
  in Luma Studio.
- Keep CSS token names and provenance metadata separate from the value formatter. A row may show
  both a token source such as `--primary` and its HSLA value.
- Ensure theme usage tables continue to group equivalent colors consistently after formatting
  changes. Grouping should use the underlying `Hsla` value, not the rounded display string.

### Phase 6: Review literal RGB sample colors

- Review direct `rgb(...)` literals in Luma Studio.
- Do not automatically rewrite every literal: distinguish paint-only sample data from user-facing
  color specifications.
- Where a literal is app chrome or interactive styling, route it through the appropriate active
  look/theme boundary instead of merely converting its syntax to HSLA.
- Where a literal is a deliberate color-space example, preserve the example and document why.

### Phase 7: Verify and document the deferred OKLCH boundary

- Confirm that no new OKLCH conversion or rendering dependency was introduced.
- Keep existing OKLCH demos functional where they already exist.
- Document that OKLCH remains an explicit color-space demo rather than the canonical display
  notation.
- Identify future requirements for OKLCH: conversion accuracy, interpolation space, gamut mapping,
  GPUI painting behavior, picker ergonomics, and export semantics.
- Keep any future OKLCH work behind an explicit advanced-tool or palette-generation boundary.
- Convert OKLCH results to `gpui::Hsla` before they enter ordinary SDK/theme state or GPUI styling
  APIs.

## Acceptance criteria

### Formatting

- Ordinary Luma Studio color readouts use compact HSLA notation.
- Ordinary Luma Studio color state and theme-facing values use `gpui::Hsla`.
- No ordinary color value display uses `rgb(...)`, `rgba(...)`, RGB triplets, or Hex as its default
  representation.
- Compact HSLA formatting is implemented once and reused across color exposition and inspection
  surfaces.
- Alpha is displayed compactly and consistently.
- Formatter tests cover normalized and boundary values.

### Color controls

- RGB interpolation and RGBA channel demos still work and remain correctly labeled.
- Alternate RGB/RGBA specifications convert to Hsla at the ordinary GPUI/theme boundary.
- HSL, HSV, Lab, and existing OKLCH demos still work.
- OKLCH is not required for ordinary SDK color authoring, storage, or rendering.
- Any future OKLCH algorithm has a clear conversion boundary back to `gpui::Hsla`.
- General selected-color summaries from those demos use compact HSLA where appropriate.
- The shadow-button prototype displays and explains colors using HSLA.
- Any retained RGB/hex input compatibility is accurately described in the UI.

### Architecture

- The change aligns ordinary app/theme color storage with GPUI's first-class `Hsla` color APIs
  without changing GPUI rendering behavior.
- The change does not introduce Shadcn-specific color formatting into the SDK.
- Paint-only sample literals are not confused with display-format requirements.
- Theme provenance and CSS token names remain separate from color-value formatting.
- No OKLCH runtime pipeline is introduced.

### Verification

- Run the relevant Luma Studio tests.
- Manually inspect color field, slider, arc, ring, picker, composition, shadow-button, inspector,
  palette, and theme-usage surfaces.
- Exercise opaque, transparent, grayscale, highly saturated, and hue-wrap colors.
- Verify narrow cards and monospace detail rows do not truncate or overflow compact HSLA values.
- Run `cargo fmt`, `cargo clippy`, and relevant tests when implementation begins.

## Risks and mitigations

### Removing meaningful RGB documentation

**Risk:** A broad search-and-replace removes RGB labels from controls whose purpose is to compare
color-space behavior.

**Mitigation:** Classify occurrences before editing. Preserve algorithm/channel labels and change
only general color-value readouts.

### Formatter drift

**Risk:** Multiple local HSLA formatters continue to diverge in rounding or alpha behavior.

**Mitigation:** Consolidate to one formatter and add direct unit tests.

### Confusing display format with parser format

**Risk:** Changing visible HSLA output accidentally breaks existing RGB/hex input behavior.

**Mitigation:** Treat display and parsing as separate contracts. Decide compatibility explicitly and
  test retained input formats.

### Rounded display values used for logic

**Risk:** Grouping, equality, or theme usage logic begins relying on rounded HSLA strings.

**Mitigation:** Use underlying typed color values for logic and use formatted HSLA only at the UI
  boundary.

### Incorrect claim about color-space support

**Risk:** Adopting HSLA as the display and ordinary internal format is interpreted as a claim that
HSLA is perceptually uniform, or that OKLCH is unnecessary for every future tool.

**Mitigation:** Document HSLA as the current GPUI-native, developer-facing representation. Keep
optional OKLCH algorithms and designer-oriented tooling separate, with explicit conversion back to
HSLA at the SDK boundary.

## Definition of done

Luma Studio presents ordinary color values consistently as compact HSLA, with one tested formatter
used across color exposition, inspector, theme, and prototype surfaces. Ordinary app and theme
color values use `gpui::Hsla`, while RGB-family algorithms and channel demonstrations remain
functional and accurately labeled. OKLCH remains deferred as an optional advanced color-space or
palette-generation capability rather than becoming the default developer-facing color model.
