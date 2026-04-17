# Organization Plan

This document describes the intended module organization for GPUI-Luma as the SDK grows.

The structure should be adopted incrementally. Do not create empty directories or move working code
just to satisfy this shape. Add each layer when real controls need it.

## Guiding Rule

Organize controls by taxonomy, move shared behavior into `foundation`, and keep theme policy
orthogonal to control behavior and template structure.

Controls own behavior. Templates own structure. Themes own appearance policy.

## Recommended SDK Shape

```text
crates/sdk/src/
  foundation/
    interaction.rs
    layout/
      anchoring.rs
      placement.rs
    state/
      control_state.rs
      validation_state.rs
    ids/
      control_id.rs
    events/
      control_events.rs
    a11y/
      roles.rs
      semantics.rs
      navigation.rs

  controls/
    command/
      button/
        control.rs
        model.rs
        template.rs
        mod.rs
      icon_button/
        control.rs
        icon.rs
        model.rs
        template.rs
        mod.rs
      toggle_button/
        control.rs
        model.rs
        template.rs
        mod.rs

    choice/
      checkbox/
        control.rs
        model.rs
        template.rs
        mod.rs

    input/
      slider/
        control.rs
        model.rs
        template.rs
        mod.rs

    menu/
      dropdown_menu/
        control.rs
        model.rs
        template.rs
        mod.rs
      context_menu/
        control.rs
        model.rs
        template.rs
        mod.rs

    feedback/
      progress/
        control.rs
        model.rs
        template.rs
        mod.rs

    layout/
      scrollbar/
        control.rs
        model.rs
        template.rs
        mod.rs

  theme/
    mod.rs
    tokens/
      mod.rs
      color.rs
      typography.rs
      spacing.rs
      radius.rs
      motion.rs
      elevation.rs
      density.rs

    state/
      mod.rs
      interaction.rs
      validation.rs
      focus.rs

    command/
      mod.rs
      button_family.rs
      button.rs
      icon_button.rs
      toggle_button.rs

    choice/
      mod.rs
      checkbox.rs

    input/
      mod.rs
      range.rs
      slider.rs

    menu/
      mod.rs
      menu_surface.rs
      menu_item.rs
      dropdown_menu.rs
      context_menu.rs

    feedback/
      mod.rs
      progress.rs

    layout/
      mod.rs
      scrollbar.rs
```

## Gallery Shape

The gallery should remain outside the SDK crate so it continues to act as a real consumer.

```text
apps/gallery/src/gallery/
  mod.rs
  registry.rs
  navigation.rs
  pages/
    command.rs
    choice.rs
    input.rs
    menu.rs
    feedback.rs
    layout.rs
```

The gallery should use the same public APIs as any application. Custom templates and custom themes
used by the gallery should live in the gallery app until they prove broadly reusable.

## Near-Term Migration

Use this order unless a concrete implementation need suggests otherwise:

1. Add `Progress` using the current module style.
2. Add `Slider` using the current module style.
3. Extract shared range, track, or interaction behavior only after `Progress` and `Slider` reveal real duplication.
4. Add `Scrollbar`.
5. Move shared interaction and placement helpers into `foundation`.
6. Reorganize controls by taxonomy once multiple categories exist.
7. Add gallery navigation and a registry before the gallery becomes hard to scan.
8. Add a gallery-local production theme.
9. Extract a production theme crate only after the theme API feels stable.

## Control Categories

Use these categories for new controls:

- `command`: buttons and command-like invocations.
- `choice`: checkboxes, radio buttons, switches, selects, and other choice controls.
- `input`: text and value-entry controls, including sliders.
- `menu`: dropdown menus, context menus, menu bars, and menu items.
- `feedback`: progress, spinners, alerts, badges, skeletons, and other status indicators.
- `navigation`: tabs, sidebars, breadcrumbs, pagination, steppers, accordions, and collapsibles.
- `data`: lists, tables, trees, and virtualized data surfaces.
- `layout`: scrollbars, resizable regions, dividers, docks, title bars, and focus traps.
- `media`: icons, images, avatars, clipboard affordances, and keyboard key displays.

Only add a category directory when it has real controls.

## Foundation

`foundation` is for behavior or semantics shared across control families.

Good candidates:

- hover, pressed, disabled, and focus projection
- pointer handling
- keyboard handling
- control IDs
- anchoring and placement
- validation state
- accessibility semantics

Do not move code into `foundation` just because it might become shared. Wait until at least two
controls need the same behavior.

## Themes

The SDK default theme should stay generic, complete, and predictable. It should validate the control
architecture, not carry the final product aesthetic.

The theme tree should scale by taxonomy, just like controls. Avoid one flat `theme/` directory once
there are more than a handful of controls.

Recommended theme organization:

- `tokens/`: primitive design tokens such as color, spacing, radius, typography, motion, elevation,
  and density.
- `state/`: cross-family visual state projections such as interaction, validation, and focus.
- category directories: control-family appearance traits, default resolvers, and shared appearance
  structs for that category.

Theme modules should mirror useful visual families, not mechanically duplicate every control file.
For example, `button_family.rs` can cover `Button`, `IconButton`, and `ToggleButton` when they
share state and token policy. Likewise, a future `range.rs` can cover shared slider/progress/scrollbar
track policy if the duplication is real.

Each control-family theme module should own:

- the appearance struct consumed by templates
- the theme trait
- the default theme implementation
- a `default_*_theme()` constructor when useful

Theme modules should not own:

- control behavior
- persistent control state
- emitted events
- GPUI element structure

Production visual styles should start as gallery-owned themes. Once a production theme stabilizes,
move it into a separate crate rather than blending it into the SDK default theme.

Potential future crate:

```text
crates/theme-modern/
  src/
    lib.rs
    tokens/
      mod.rs
      color.rs
      typography.rs
      spacing.rs
      radius.rs
      motion.rs
      elevation.rs
      density.rs
    state/
      mod.rs
      interaction.rs
      validation.rs
      focus.rs
    command/
      mod.rs
      button_family.rs
    choice/
      mod.rs
      checkbox.rs
    input/
      mod.rs
      range.rs
      slider.rs
    menu/
      mod.rs
      menu_surface.rs
      menu_item.rs
    feedback/
      mod.rs
      progress.rs
    layout/
      mod.rs
      scrollbar.rs
```

## Deferred: Look Layer

A separate `look` layer may become useful later, but it should not be introduced yet.

The risk is that `look` can become a second template system or a second theme system. If it appears,
it should act as an adapter layer for reusable visual recipes across templates, while preserving:

- controls own behavior
- templates own structure
- themes own appearance policy

Introduce this layer only after there are multiple real visual systems competing with the generic
templates and theme APIs.

## Deferred: Visual State Manager

A Visual State Manager-style layer is likely useful later for focus and composite control states,
but it should wait until the need is concrete.

Likely trigger points:

- focus-visible vs focused vs keyboard modality
- active descendant focus
- composite controls with internal focus zones
- animated transitions
- validation states layered with interaction states
- selected, current, expanded, hovered, pressed, disabled, and focused states overlapping

Until then, `InteractionState` and shared interaction helpers should remain the simple visual-state
projection layer.
