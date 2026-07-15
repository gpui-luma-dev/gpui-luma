# Content Only Button Variant Plan

## Problem

Theme Studio titlebar icon actions need normal button behavior without normal button chrome. Disabling only the focus adorner is too narrow because hover, active, selected, border, fill, and focus-space reservation also need to disappear together.

## Direction

Add **Content Only** as a real button variant, not a one-off `content_only_button` control and not a stack of boolean suppressors.

The variant should be available across the button-family factories where the model already supports shared button styling:

- `content_only_button`
- `content_only_icon_button`
- `content_only_checkbox`
- `content_only_toggle`
- `content_only_radio`
- `content_only_switch`

It may be less useful for radio and switch, but the shared model suggests the variant should be consistently available rather than leaving holes in the factory surface.

## Style Guide

Update Theme Studio's style guide to show the variant as **Content Only** alongside the existing button variants. The preview should make clear that this is still an interactive control, just without visible chrome.

Include at least:

- Button
- Icon Button
- Checkbox
- Toggle
- Radio
- Switch

This is not a new preview surface. Thread the variant through the existing button, icon button, toggle, checkbox, radio, and switch matrices. In particular, update the shared style arrays and manual preview `ButtonRenderModel` construction sites so the new render-model field is present everywhere previews instantiate controls directly.

## Visual Contract

### Button, Icon Button, and Toggle
For button-like controls, **Content Only** should:
- render only the control content (label/icon)
- have no background fill, borders, or elevation in any state
- have no hover or active/pressed fills
- have no template adorners and reserve no adorner space (today this means the focus adorner; the seam should remain plural for future adorners)
- keep disabled opacity behavior (e.g., `0.56` opacity)
- keep cursor, focus state, activation, and event behavior

### Checkbox, Radio, and Switch
Since these are state indicators, they must remain visible and functional. **Content Only** for these controls should:
- **Keep Indicator Visible**: Resolve checkmark boxes, radio circles, and switch tracks using standard visible colors. Treat `ContentOnly` as `Primary` for indicator color resolution unless a stronger reason appears, so selected vs. unselected states remain distinguishable.
- **Suppress Adorners & Chrome**: Remove template adorners, reserve no adorner space, and strip any container borders or background fills around the control. The current adorner is focus-only, but the API should stay extensible.

## API Shape

Expose look-layer factories through `ShadcnLookControlExt`, for example:

```rust
look.content_only_icon_button("theme-studio-sidebar-toggle", LucideIcon::PanelLeft)
look.content_only_button("plain-action")
look.content_only_checkbox("plain-checkbox")
```

These should still return normal SDK controls/builders, not raw `div` chrome.

## Implementation Sketch

### Phase 1: SDK Adorner Suppression Prep
1. Add `suppress_adorners: Cell<bool>` to `ButtonRenderModel` in [model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/model.rs).
2. Expose `.without_adorners()` on `ButtonBuilder` in [model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/model.rs) (with passthroughs on `CheckboxBuilder` and `SwitchBuilder`). The name is intentionally plural: the current implementation only has a focus adorner, but the template seam was written to allow additional adorners later. Elevation remains controlled separately by `.without_elevation()`.
3. Update [ButtonTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/template.rs), [CheckboxTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/checkbox/template.rs), [RadioButtonTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/radio_button/template.rs), and [SwitchTemplate](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/switch/template.rs) to check `model.suppress_adorners.get()`. If set, skip rendering adorners and skip adorner layout padding/oversize extent.
4. Update direct `ButtonRenderModel` construction sites in Theme Studio and Gallery previews. Manual previews will not compile until the new field is populated.

### Phase 2: Look-Shadcn Implementation
1. Add a `ContentOnly` button style/variant in [ShadcnButtonStyle](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/button.rs#L54-L60).
2. In [style.toml](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/assets/style.toml), configure `ContentOnly` as a purely color-based style resolving to transparent backgrounds, borders, and fills (no hover/pressed background fills).
3. In [checkbox.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/checkbox.rs), [radio.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/radio.rs), and [switch.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/switch.rs), if the style is `ContentOnly`, resolve the inner indicators using standard `Primary` colors so they remain fully functional, but apply `.without_adorners()`.
4. Update every `ShadcnButtonStyle` match site, including resolver helpers, stylesheet selector keys, inspection helpers, Theme Studio shared style arrays, and style-id helpers.
5. Add factory methods in [ShadcnLookControlExt](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/ext.rs). The factory methods will compose the standard `ContentOnly` style with `.without_adorners()` and `.without_elevation()`.
6. Keep SDK behavior intact: subscriptions, focus handling, activation, disabled state, presenter updates, and typed data all remain normal.
7. Move Theme Studio titlebar icon buttons to `content_only_icon_button`.
8. Update the Theme Studio style guide previews to include **Content Only**.

## Rollout Notes

- This is a medium cross-cutting change because `ShadcnButtonStyle` is a closed enum used by style resolution, stylesheet matching, inspection, and preview matrices.
- Do the SDK flag first, then the look variant, then the Theme Studio migration. That keeps compile errors localized and makes regressions easier to read.
- Add tests that cover adorner suppression for text buttons and labeled choice controls. Today the visible case is the focus adorner; test names can mention that concrete behavior while keeping the API named around adorners. Radio should cover both labeled and indicator-only roles because it already special-cases indicator-only layout.

## Non-Goals

- Do not create a separate custom control named `content_only_button`.
- Do not replace SDK controls with raw styled `div`s.
- Do not add separate builder flags for focus, hover, active, selected, border, and elevation.
- Do not make ghost buttons content-only; this is a distinct variant.
- Do not strip the indicator track/box/circle colors of checkboxes, radios, or switches.
