# Gallery Enhancement: Theme Usage Documentation

The gallery needs to explain how the active Luma theme is actually used by SDK controls. The Palette
page shows which semantic tokens exist and what values they resolve to, but it does not show which
controls consume each token or which tokens share the same concrete value.

Avoid documenting templates by hand in prose first. That will drift as resolvers and templates
change. The gallery should be driven by structured metadata that lives close to the resolver code and
can be rendered as documentation/debug UI.

## Goal

Build a theme observability layer for the gallery.

The gallery should answer:

- which semantic tokens exist in the active theme,
- which component recipes consume each token,
- which resolved appearance fields are populated by each token,
- which controls are affected by a token change,
- which tokens currently share the same concrete value,
- which tokens are active in the current SDK and which are reserved for future components.

The first version should focus on palette/color usage. The same pattern should later extend to
metrics, radii, typography, and elevation.

## Problem

Today, theme decisions are encoded in Rust resolver code. For example, a resolver may decide that a
checked checkbox indicator uses `action.primary.background`, while the template only receives and
applies `CheckboxAppearance.indicator_background`.

Navigation sidebar styling is now part of the SDK theme layer through `NavigationSidebarTheme`. That
means `navigation.*` tokens are real SDK-consumed theme roles, not gallery-only chrome values. The
usage documentation should treat the navigation sidebar like any other themed SDK component.

That means there are two useful layers to document:

```text
resolver recipe:
  checked checkbox indicator background -> action.primary.background

template application:
  CheckboxAppearance.indicator_background -> rendered indicator bg()
```

The gallery should document the resolver recipe first, because that is where semantic token choices
are made. Template documentation can come later as an appearance-field inspector.

## Proposed Model

Add structured theme usage metadata alongside each component theme resolver.

Example shape:

```rust
ThemeUsage {
    component: "Button",
    parts: vec![
        ThemePartUsage {
            part: "default background",
            token: "action.secondary.background",
            states: vec!["default"],
            appearance_field: "ButtonFamilyAppearance.background",
        },
        ThemePartUsage {
            part: "primary background",
            token: "action.primary.background",
            states: vec!["default"],
            appearance_field: "ButtonFamilyAppearance.background",
        },
    ],
}
```

This metadata should not drive rendering in the first implementation. It should be a documentation
and debugging surface. If the model proves stable, it can later become part of a data-driven recipe
system.

## Gallery Pages

Add a `Theme Usage` gallery page with at least two views.

### By Token

Show each token and where it is consumed.

Example:

```text
action.primary.background
  Button primary background
  Checkbox checked indicator background
  Radio selected indicator background
  Switch checked track background
  Slider fill background
  Progress fill color
```

Example for navigation:

```text
navigation.selected_background
  Navigation Sidebar selected item background
```

### By Component

Show each component and the semantic tokens used by its resolver.

Example:

```text
Checkbox
  unchecked indicator background -> form.input.background
  unchecked indicator border -> form.input.border
  checked indicator background -> action.primary.background
  checked checkmark -> action.primary.foreground
  focus ring -> focus.ring
```

## Shared-Value Discovery

The gallery should also expose tokens that currently share the same concrete value.

Example:

```text
hsla(221.2 83.2% 53.3% / 1)
  action.primary.background
  state.selected.background
  data.accent_1 *
```

This helps identify accidental coupling and intentional aliases.

## Reserved Tokens

The Palette page already marks reserved/future-facing tokens with `*`. The usage page should make the
same distinction explicitly:

- `Used by SDK`
- `Used by Gallery Chrome`
- `Reserved`

Reserved tokens should remain visible because they are part of the Luma theme contract, but the UI
should not imply they currently affect SDK controls.

## Scope For First Pass

Implement only palette/color usage metadata.

Include the current SDK controls:

- Button
- Navigation Sidebar
- Icon Button
- Toggle Button
- Toggle Group
- Checkbox
- Radio Group
- Switch
- Slider
- Scrollbar
- Popup Menu
- Context Menu
- Tabs Navigation
- Progress

The first pass does not need runtime introspection of every state combination. Static metadata is
enough, as long as it lives near the resolver code and is easy to keep in sync.

Navigation sidebar usage metadata starts in `crates/sdk/src/theme/navigation_sidebar.rs`, alongside
the `NavigationSidebarTheme` resolver, and is rendered by the gallery's `Theme Usage` page.

## Later Scope

After palette usage is useful, extend the same model to non-color theme roles:

- radius roles such as `radius.md`, `radius.sm`, and `radius.pill`,
- spacing and control size roles,
- typography roles such as `text.label`, `text.body`, and `text.caption`,
- elevation roles such as `shadow.thumb` and `shadow.menu`,
- border width and focus affordance roles.

Add dedicated gallery pages or tabs for:

- Palette
- Metrics
- Typography
- Elevation
- Usage

## Resolved Appearance Inspector

A later gallery enhancement should show the final resolved appearance structs for each component and
state.

Example:

```text
Button / Primary / Hovered
  background: action.primary.hover_background -> hsla(...)
  foreground: action.primary.foreground -> hsla(...)
  border: action.primary.background -> hsla(...)
  radius: radius.md -> 6
  typography: text.label -> 13 / 18 / 500
```

Example for navigation:

```text
Navigation Sidebar / Item / Selected
  background: navigation.selected_background -> hsla(...)
  foreground: navigation.selected_foreground -> hsla(...)
  focus: navigation.focus_ring -> hsla(...)
  radius: radius.md -> 6
  typography: text.label -> 13 / 18 / 500
```

This should complement the usage metadata. The usage metadata explains intended recipe choices; the
resolved inspector confirms the final values currently produced by the SDK.

## Implementation Notes

- Keep semantic decisions in resolvers.
- Keep templates focused on applying resolved appearance fields.
- Treat navigation sidebar as a first-class SDK themed component.
- Keep usage metadata close to resolver modules, not in gallery-only prose.
- Avoid duplicating long hand-written explanations in docs and UI.
- Use the gallery to render the metadata and make drift visible during development.
- Start with color tokens before expanding to radius, shadows, and typography.
