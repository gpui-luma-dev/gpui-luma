# Color Viz Next Steps

## Goal

Prepare `apps/color-viz` for broader color-space support without growing more `gpui::Hsla`-centric helper code.

The next likely feature direction is support for multiple editable color spaces such as:

- `rgb`
- `hsl`
- `hsv`
- `oklch`

That means the app should be organized so color math is not tied to one display format or one GPUI-facing type.

## Main Direction

Use the `palette` crate as the primary color engine.

`gpui::Hsla` should be treated as a boundary type for:

- painting into GPUI controls
- talking to existing GPUI color widgets
- handing colors to preview/render code that already expects GPUI values

It should not be the app's internal source of truth for conversion, interpolation, parsing, or formatting.

## Recommended Structure

### 1. Add an app-local color abstraction

Create a small color-model layer for color-viz rather than passing `gpui::Hsla` everywhere.

Possible shape:

```rust
enum ColorValue {
    Srgb(...),
    Hsl(...),
    Hsv(...),
    Oklch(...),
}
```

This layer should own:

- conversion between supported color spaces
- conversion to `gpui::Hsla`
- formatting for UI and CSS
- parsing from user-facing text fields
- access to per-space components for future editors

### 2. Keep gradient stops color-space agnostic

Gradient stops should mean:

- `position`
- `color value`

Not:

- `position + hex`
- `position + hsla`

Formatting to hex or CSS strings should stay in the presentation layer.

### 3. Separate three concerns currently mixed together

The current `gradient_builder/color.rs` is doing several jobs at once:

- parsing and formatting
- color-space conversion
- interpolation

Those should be split before more color spaces are added.

Suggested separation:

- `color/model.rs`
  Color value types and conversions
- `color/format.rs`
  Hex/CSS/user-facing formatting
- `color/parse.rs`
  User input parsing
- `color/interpolate.rs`
  Interpolation policies

## Important Design Rule

Do not conflate:

- the color space used to store or edit a color
- the color space used to interpolate a gradient

Those are separate decisions.

For example:

- a stop may be edited as `oklch`
- the gradient may still interpolate in `srgb`, `hsl`, or `oklch`

That separation will matter as soon as the app exposes interpolation choices.

## What To Avoid

Avoid extending the current pattern of:

- `parse_* -> gpui::Hsla`
- `format_* <- gpui::Hsla`
- ad hoc conversion helpers around `gpui::Hsla`

That approach will get harder to maintain once `rgb`, `hsl`, and `oklch` all exist side by side.

Also avoid forcing all color spaces into one flat generic component schema too early. Their semantics differ enough that a thin typed layer per color space is cleaner.

## Practical Near-Term Plan

1. Keep `palette` as the math/conversion core.
2. Introduce one app-local color model above it.
3. Push `gpui::Hsla` to the rendering/control boundary.
4. Split formatting/parsing/interpolation out of the current helper file.
5. Add new color spaces on top of that layer instead of adding more direct `Hsla` helpers.

## Why This Matters

If color-viz is going to grow into a real gradient exploration tool, the app needs to be:

- palette-first for correctness and reuse
- additive rather than rewrite-heavy
- color-space agnostic at the stop/model level

That gives a cleaner path to `rgb`, `hsl`, `oklch`, and later interpolation-mode support without another structural rewrite.

## Additional Next Step

Add support for copying the generated CSS spec out of the app and pasting/importing a CSS gradient spec back into the app.

Add support for full `linear-gradient(...)` angles rather than only the current cardinal-angle presets.

In the app UI, change the current angle control over to an angle slider so the preview and generated CSS can be driven across the full linear-gradient angle range.

Add a light/dark theme switch in the app title bar, consistent with the other apps in the workspace.
