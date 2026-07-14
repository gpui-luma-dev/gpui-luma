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

## Visual Contract

### Button, Icon Button, and Toggle
For button-like controls, **Content Only** should:
- render only the control content (label/icon)
- have no background fill, borders, or elevation in any state
- have no hover or active/pressed fills
- have no focus adorner and reserve no focus-adorner space
- keep disabled opacity behavior (e.g., `0.56` opacity)
- keep cursor, focus state, activation, and event behavior
<!--  -->
### Checkbox, Radio, and Switch
Since these are state indicators, they must remain visible and functional. **Content Only** for these controls should:
- **Keep Indicator Visible**: Resolve checkmark boxes, radio circles, and switch tracks using standard visible colors (falling back to `Primary` colors) so their selected vs. unselected states remain distinguishable.
- **Suppress Focus & Chrome**: Remove the focus ring adorner, reserve no focus-ring space, and strip any container borders or background fills around the control.

## API Shape

Expose look-layer factories through `ShadcnLookControlExt`, for example:

```rust
look.content_only_icon_button("theme-studio-sidebar-toggle", LucideIcon::PanelLeft)
look.content_only_button("plain-action")
look.content_only_checkbox("plain-checkbox")
```

These should still return normal SDK controls/builders, not raw `div` chrome.

## Implementation Sketch

1. Add a `ContentOnly` button style/variant in [ShadcnButtonStyle](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/button.rs#L54-L60).
2. Add factory methods for content-only controls in [ShadcnLookControlExt](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/ext.rs#L50-L151).
3. Update [button_family_focus_adorner](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/button_family/theme.rs#L139-L163) to return `None` if `focus_ring.a <= 0.0`. This suppresses focus space reservation automatically via the layout engine.
4. For `Checkbox`, `Radio`, and `Switch` color look-resolvers:
   - If the style is `ContentOnly`, resolve the indicator colors using `ShadcnButtonStyle::Primary` instead, but override their `adorner` to `None`.
5. Keep SDK behavior intact: subscriptions, focus handling, activation, disabled state, presenter updates, and typed data all remain normal.
6. Move Theme Studio titlebar icon buttons to `content_only_icon_button`.
7. Update the Theme Studio style guide previews to include **Content Only**.

## Non-Goals

- Do not create a separate custom control named `content_only_button`.
- Do not replace SDK controls with raw styled `div`s.
- Do not add separate builder flags for focus, hover, active, selected, border, and elevation.
- Do not make ghost buttons content-only; this is a distinct variant.
- Do not strip the indicator track/box/circle colors of checkboxes, radios, or switches.

