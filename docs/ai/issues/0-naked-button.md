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

Content Only should:

- render only the control content
- have no focus adorner
- reserve no focus-adorner space
- have no hover fill
- have no active/pressed fill
- have no selected fill
- have no border
- have no elevation
- keep disabled opacity behavior
- keep cursor, focus state, activation, and event behavior

## API Shape

Expose look-layer factories through `ShadcnLookControlExt`, for example:

```rust
look.content_only_icon_button("theme-studio-sidebar-toggle", LucideIcon::PanelLeft)
look.content_only_button("plain-action")
look.content_only_checkbox("plain-checkbox")
```

These should still return normal SDK controls/builders, not raw `div` chrome.

## Implementation Sketch

1. Add a `ContentOnly` button style/variant in the Shadcn button-family path.
2. Add factory methods for content-only controls in `ShadcnLookControlExt`.
3. Keep SDK behavior intact: subscriptions, focus handling, activation, disabled state, presenter updates, and typed data all remain normal.
4. If adorner reservation cannot be suppressed through look/style alone, add the smallest SDK-level visual-policy hook needed by the template.
5. Move Theme Studio titlebar icon buttons to `content_only_icon_button`.
6. Update the Theme Studio style guide previews to include **Content Only**.

## Non-Goals

- Do not create a separate custom control named `content_only_button`.
- Do not replace SDK controls with raw styled `div`s.
- Do not add separate builder flags for focus, hover, active, selected, border, and elevation.
- Do not make ghost buttons content-only; this is a distinct variant.

## Open Decision

Decide whether focus-adorner suppression belongs entirely in the Shadcn variant/template path or requires a minimal SDK button render-model policy. Prefer keeping the public concept as a **variant**, even if the template needs a small internal hook.
