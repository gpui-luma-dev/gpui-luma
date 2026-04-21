# GPUI-Luma Control Guidelines

This document is the implementation-facing companion to `docs/control-design.md`. It describes the
control features that exist today and the implementation patterns to follow when adding a new
control to the SDK.

The short version: controls own behavior, templates own GPUI structure, and themes own appearance
policy. Keep those boundaries intact even when a control is small.

## Current Control Set

The SDK currently ships these controls:

- `Button`: command button with text label, kind, size, enabled state, and click events.
- `IconButton`: command button with a required app-owned icon and click events.
- `ToggleButton`: button-family control with owned selected state and change events.
- `Checkbox`: binary checked control with owned checked state and change events.
- `Switch`: binary on/off control with optional label and change events.
- `RadioGroup`: composite choice control with mutually exclusive item selection.
- `Slider`: numeric input with range, step, pointer dragging, and keyboard value changes.
- `Scrollbar`: horizontal or vertical range control with line/page movement and draggable thumb.
- `Progress`: non-interactive circular status indicator.
- `PopupMenu`: trigger-owned menu with one level of submenu support.
- `ContextMenu`: secondary-click or keyboard-opened menu with one level of submenu support.

Each control is under `crates/sdk/src/controls/<control>/` and normally uses:

```text
control.rs   live GPUI entity, behavior, input handling, events, mutation methods
model.rs     public builder, stored model, render model, item models if any
template.rs  template trait, default themed template, template handler bundle if needed
mod.rs       public re-exports
```

Shared helpers currently live in:

- `controls/interaction.rs`: hover, pressed, disabled, focus handle, and tab-stop handling.
- `controls/state.rs`: focus and composite item state projection.
- `controls/value.rs`: numeric range, finite-value coercion, and step normalization.
- `controls/menu_navigation.rs`: enabled-item navigation for popup and context menus.
- `controls/button_family.rs`: shared button kind, size, and interaction aliases.
- `theme/*`: appearance resolvers and data-backed theme tokens.
- `keyhandling.rs`: action names, key contexts, and default key bindings.

## Core Architecture

### Model And Builder

The builder owns an initial model and exposes typed builder methods. `new(id)` should be cheap,
typed, and deterministic. `.spawn(cx)` creates the GPUI entity with `cx.new`.

Existing builders follow this shape:

```rust
pub struct ExampleBuilder {
    pub(crate) model: ExampleModel,
}

impl ExampleBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ExampleModel {
                id: id.into(),
                enabled: true,
                template: default_example_template(),
            },
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Example> {
        cx.new(|cx| Example::from_builder(self, cx))
    }
}
```

Guidelines:

- Use `SharedString` for public IDs and labels.
- Default labels to the ID only when a visible label is part of the control contract.
- Store the template as `Arc<dyn <Control>Template>`.
- Prefer typed enums and typed values over strings.
- Normalize builder inputs immediately when invalid values can leak into rendering.
- For numeric controls, use `ControlRange`, `value_from_input`, and `normalized_step`.

### Control Entity

The live control owns persistent behavior state and emits semantic events. It should expose mutation
methods for externally meaningful state only.

Common responsibilities:

- store the model,
- initialize focus and interaction state,
- implement `EventEmitter` when user events exist,
- implement `Focusable` for interactive controls,
- implement `Render`,
- build a narrow render model,
- wire GPUI pointer and keyboard handlers,
- call `cx.notify()` after local state changes,
- call `cx.emit(...)` for semantic events.

Existing command-style controls use this activation shape:

```rust
fn activate(&mut self, cx: &mut Context<Self>) -> bool {
    if !self.model.enabled {
        return false;
    }

    cx.emit(ButtonEvent::Click);
    true
}
```

Use a shared `activate` method when pointer and keyboard paths should produce the same behavior.
Do not create separate mouse-only and keyboard-only semantic paths unless the control behavior
requires it.

### Render Models

Templates receive render models, not mutable control references. Render models carry semantic state
and data needed to create element IDs.

Good render-model fields:

- `id`,
- visible content such as `label` or `icon`,
- semantic variant such as `kind`,
- semantic value such as `checked`, `on`, `selected`, `value`, or `percentage`,
- `enabled`,
- projected interaction or focus state.

Avoid render-model fields for resolved colors, padding, mutable callbacks, or application state.
Handler bundles are separate from render models for controls where the template owns child surfaces.

### Templates

A template trait turns a render model into GPUI elements:

```rust
pub trait ExampleTemplate: Send + Sync {
    fn render(
        &self,
        model: &ExampleRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

Default templates are themed templates behind `OnceLock<Arc<dyn ...>>`:

```rust
pub fn default_example_template() -> Arc<dyn ExampleTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ExampleTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedExampleTemplate::new(default_example_theme())))
        .clone()
}
```

Template guidelines:

- The template owns structure and layout, not persistent behavior.
- The template may attach handlers supplied by the control.
- The template may use SDK-owned affordance icons when they communicate built-in state.
- The template must not emit SDK events directly.
- Use named child IDs for repeated item children when the root ID is shared.

The default templates currently draw focused state with `focus_debug_border()`. Theme appearance
objects also expose `focus_ring`, but most default templates do not consume it yet. When improving
focus rendering, update templates and themes together rather than changing behavior in controls.

### Themes

Themes convert semantic state into appearance:

```rust
pub trait ExampleTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ExampleAppearance;
}
```

The default themes are data-backed by `ThemeTokens`:

```rust
pub struct ThemeTokens {
    pub colors: ColorTokens,
    pub metrics: MetricTokens,
}
```

Use `InteractionState::layer()` for visual precedence:

1. disabled
2. pressed
3. hovered
4. default

Focused state is orthogonal. It should affect focus affordances, not replace hover or pressed state.

Use `ButtonFamilyTheme` only for controls that behave like the button family. The current roles are:

```rust
pub enum ButtonFamilyRole {
    Text,
    Icon,
    Toggle { selected: bool },
}
```

Create a specific theme resolver when the control has distinct policy, as `CheckboxTheme`,
`SwitchTheme`, `RadioGroupTheme`, `SliderTheme`, `ScrollbarTheme`, `ProgressTheme`,
`PopupMenuTheme`, and `ContextMenuTheme` do.

## Interaction And Focus

Use `ControlInteraction` for single-surface interactive controls. It owns:

- `InteractionState`,
- a GPUI `FocusHandle`,
- tab-stop enablement,
- hover and pressed state,
- focus on mouse down,
- disabled-state cleanup.

Controls using `ControlInteraction` should call:

- `ControlInteraction::new(enabled, cx)` in `from_builder`,
- `set_enabled(enabled)` from the control's `set_enabled`,
- `render_state(enabled, window)` when building render models,
- `handle_hover`,
- `handle_mouse_down`,
- `handle_mouse_up`.

Disabled controls should not emit events, keep hover or pressed state, or remain in the tab order.
`ControlInteraction::set_enabled(false)` clears hover/pressed state and updates the tab stop.

Composite controls may need explicit item state. `RadioGroup` uses one group focus handle and
per-item `CompositeItemState`. Menus use `MenuPath` to separate active descendants from hovered
items and open submenus.

Navigation controls such as `NavigationSidebar` and future tab controls should also use a composite
active-descendant model. The navigation control owns focus for its own item surface: pointer clicks
and keyboard activation should focus the navigation control, set the active item, and render a
visible affordance on that active item. Do not automatically move focus into the activated content
pane from inside the navigation control.

Activated content owns the focus handoff decision. The navigation control should emit a semantic
activation event, and the application or destination pane should decide whether to:

- keep focus on the navigation control,
- focus the content pane root,
- focus a primary child control inside the pane,
- defer focus until after the activated content is rendered.

This keeps route navigation, tab previews, passive documentation panes, forms, editors, and canvas
surfaces from sharing one hard-coded focus policy. Gallery or app code should make the focus
handoff explicit when a pane wants focus on activation.

## Keyboard Handling

Install default control key bindings with `keyhandling::bind_default_control_keys(cx)` and default
focus traversal keys with `focus::bind_default_focus_keys(cx)`. `gpui_luma::init(cx)` only registers
the Lucide icon font; app startup must bind focus and control keys where appropriate, as the gallery
does in `apps/gallery/src/main.rs`. See `focus-handling.md` for the focus-scope model behind
`Tab`, `Shift-Tab`, and `Escape`.

Use the existing key profiles:

- `ControlKeyProfile::Command`: enter/space activation for button-family command controls.
- `ControlKeyProfile::Choice`: space activation for checkbox and switch.
- `ControlKeyProfile::RadioGroup`: arrows, home, and end.
- `ControlKeyProfile::RangeValue`: arrows, page up/down, home, and end.
- `ControlKeyProfile::ScrollOffset`: scroll offset arrows, page up/down, home, and end.
- `ControlKeyProfile::Menu`: popup menu navigation and activation.
- `ControlKeyProfile::ContextMenu`: context-menu open, navigation, and activation.
- `ControlKeyProfile::Navigation`: navigation collection arrows, home/end, submenu open/close, and
  activation.

Attach action handlers to the focus-tracked element. For example, button-family controls track focus,
set the key context, and handle `ActivateControl`.

## Control-Specific Notes

### Button

`Button::new(id)` creates a text command button. Builder methods include `label`, `kind`, `size`,
`enabled`, `template`, and `spawn`.

Runtime mutation:

- `set_label`,
- `set_enabled`.

Event:

- `ButtonEvent::Click`.

Implementation notes:

- Uses `ControlInteraction`.
- Uses `ButtonFamilyTheme` with `ButtonFamilyRole::Text`.
- `ButtonKind` maps to `ButtonVariant`.

### IconButton

`IconButton::new(id, icon)` requires an app-owned icon.

Runtime mutation:

- `set_icon`,
- `set_enabled`.

Event:

- `IconButtonEvent::Click`.

Implementation notes:

- Uses `ControlInteraction`.
- Uses `ButtonFamilyTheme` with `ButtonFamilyRole::Icon`.
- `ButtonFamilyTheme` gives icon buttons square sizing and circular radius.
- `IconButtonIcon::Lucide` renders through the registered Lucide font.
- `IconButtonIcon::SvgPath` renders through `svg().external_path(...)`.
- Strings are explicit SVG paths, not Lucide names.

### ToggleButton

`ToggleButton` owns selected state.

Runtime mutation:

- `selected`,
- `set_selected`,
- `set_enabled`.

Event:

- `ToggleButtonEvent::Change { selected }`.

Implementation notes:

- Uses `ControlInteraction`.
- Activation toggles internal state, emits `Change`, and notifies.
- Uses `ButtonFamilyTheme` with `ButtonFamilyRole::Toggle { selected }`.

### Checkbox

`Checkbox` owns checked state and visible label.

Runtime mutation:

- `checked`,
- `set_checked`,
- `set_enabled`.

Event:

- `CheckboxEvent::Change { checked }`.

Implementation notes:

- Uses `ControlInteraction`.
- Uses `ControlKeyProfile::Choice`.
- Uses `CheckboxTheme`.
- The default template renders the checkmark as an SDK-owned Lucide `Check` affordance.

### Switch

`Switch` owns on/off state and supports an optional label.

Runtime mutation:

- `on`,
- `set_on`,
- `set_enabled`.

Event:

- `SwitchEvent::Change { on }`.

Implementation notes:

- Uses `ControlInteraction`.
- Uses `ControlKeyProfile::Choice`.
- Uses `SwitchTheme`.
- The template derives thumb position from `on`; the control does not store visual positions.

### RadioGroup

`RadioGroup` owns a list of `RadioGroupItem`s and one selected item.

Builder methods:

- `item`,
- `items`,
- `selected`,
- `enabled`,
- `template`,
- `spawn`.

Runtime mutation:

- `selected_id`,
- `set_selected`,
- `set_items`,
- `set_enabled`.

Event:

- `RadioGroupEvent::Change { selected_id, label }`.

Implementation notes:

- Uses its own group `FocusHandle` instead of `ControlInteraction`.
- Normalizes invalid or missing selection to the first enabled item.
- Selection changes only to enabled items.
- Arrow navigation wraps and skips disabled items.
- Template handlers are generated per item and passed separately from the render model.
- Item appearance is resolved through `RadioGroupTheme::resolve_item`.

### Slider

`Slider` owns a numeric value, range, step, enabled state, and measured track bounds.

Builder methods:

- `range`,
- `step`,
- `value`,
- `enabled`,
- `template`,
- `spawn`.

Runtime mutation:

- `value`,
- `range`,
- `set_value`,
- `set_range`,
- `set_step`,
- `set_enabled`.

Event:

- `SliderEvent::Change { value }`.

Implementation notes:

- Uses `ControlInteraction`.
- Uses `ControlRange` to clamp and snap values.
- Invalid numeric inputs fall back to finite defaults through `value_from_input`.
- Invalid steps fall back to `1.0`.
- The template reports track bounds through a canvas measurement callback.
- Pointer down and drag compute value from pointer position.
- Keyboard actions adjust by `step`, `step * 10.0`, or jump to range boundaries.
- Programmatic `set_value` notifies but does not emit `SliderEvent`; user interaction emits.

### Scrollbar

`Scrollbar` owns a numeric scroll offset, range, step, page step, orientation, thumb fraction,
enabled state, and measured track/thumb geometry.

Builder methods:

- `orientation`,
- `horizontal`,
- `vertical`,
- `range`,
- `step`,
- `page_step`,
- `value`,
- `thumb_fraction`,
- `enabled`,
- `template`,
- `spawn`.

Runtime mutation:

- `value`,
- `range`,
- `orientation`,
- `step`,
- `page_step`,
- `thumb_fraction`,
- `set_value`,
- `set_range`,
- `set_orientation`,
- `set_step`,
- `set_page_step`,
- `set_thumb_fraction`,
- `set_enabled`.

Event:

- `ScrollbarEvent::Change { value }`.

Implementation notes:

- Uses `ControlInteraction`.
- Uses `ControlRange` to clamp and snap values.
- Invalid numeric inputs fall back to finite defaults through `value_from_input`.
- Invalid line/page steps fall back to `1.0`.
- Thumb fractions clamp to a visible range.
- The template reports track and thumb bounds through canvas measurement callbacks.
- Pointer down on the track pages toward the pointer; thumb dragging maps pointer movement to value.
- Keyboard actions adjust by `step`, `page_step`, or jump to range boundaries.
- Focused scrollbars handle trackpad and wheel input along their orientation axis.
- Programmatic `set_value` notifies but does not emit `ScrollbarEvent`; user interaction emits.

### Progress

`Progress` is a non-interactive circular status indicator.

Builder methods:

- `range`,
- `value`,
- `template`,
- `spawn`.

Runtime mutation:

- `value`,
- `range`,
- `set_value`,
- `set_range`.

Events:

- none.

Implementation notes:

- Does not implement `EventEmitter` or `Focusable`.
- Uses `ControlRange` to clamp values.
- Render model includes `percentage`.
- Default template paints the circular track and progress arc on a GPUI canvas.

### PopupMenu

`PopupMenu` owns trigger state, open state, one active root item, and at most one open submenu.

Builder methods:

- `label`,
- `item`,
- `items`,
- `enabled`,
- `placement`,
- `template`,
- `spawn`.

Runtime mutation:

- `set_label`,
- `set_items`,
- `set_enabled`.
- `set_placement`.

Event:

- `PopupMenuEvent::Select { item_id, label }`.

Implementation notes:

- Uses `ControlInteraction` for the trigger.
- Menu open state, `open_submenu`, and `active_path` are internal.
- Disabled menus close immediately when disabled.
- Root and submenu navigation use `MenuNavigator`.
- Only enabled leaf items emit select events.
- Current `MenuPath` supports root items and one submenu level.
- The default template renders the menu as a deferred overlay anchored from the trigger according to
  `PopupMenuPlacement`.
- Menu item icons use the same explicit Lucide-or-SVG-path contract as icon buttons.
- Chevron affordances are SDK-owned.

### ContextMenu

`ContextMenu` reuses `PopupMenuItem` as its item model and owns a pointer or keyboard menu
position.

Builder methods:

- `label`,
- `item`,
- `items`,
- `enabled`,
- `template`,
- `spawn`.

Runtime mutation:

- `set_label`,
- `set_items`,
- `set_enabled`.

Event:

- `ContextMenuEvent::Select { item_id, label }`.

Implementation notes:

- Uses `ControlInteraction` for the target.
- Opens from right click through `on_aux_click`.
- Opens from keyboard through `OpenContextMenu`, currently bound to `shift-f10` and `menu`.
- Records target bounds so keyboard-opened menus can appear at the target's lower-left corner.
- Uses `anchored().snap_to_window_with_margin(...)` and `deferred(...)` for overlay placement.
- Navigation, submenu support, and selection rules match `PopupMenu`.
- Custom templates can radically change presentation while preserving the event and item model; the
  gallery's radial context menu is the current example.

## Icons

App-owned icons must be passed explicitly. The SDK supports:

- typed `lucide_icons::Icon` values,
- explicit SVG paths as `&str`, `String`, or `SharedString`.

A string is treated as an SVG path, not as a Lucide icon name. Do not add string-to-Lucide
normalization to the SDK. If an app wants that mapping, it belongs in the app.

SDK-owned affordance icons are allowed when they communicate built-in control state or structure:

- checkbox checkmark,
- popup chevron,
- submenu chevron,
- other future structural affordances.

Because Lucide glyphs render through the bundled font, applications must call `gpui_luma::init(cx)`
before rendering SDK controls that use Lucide icons.

## Adding A New Control

Use this sequence when adding a control:

1. Decide whether it is command, choice, value input, menu, feedback, or another category.
2. Add a `<control>/` module with `model.rs`, `control.rs`, `template.rs`, and `mod.rs`.
3. Define the public builder and stored model.
4. Define semantic events before writing template code.
5. Decide whether the control is interactive. Use `ControlInteraction` for a single interactive
   surface.
6. Decide whether it is composite. If the template owns child surfaces, define a handler bundle.
7. Define a narrow render model.
8. Define a template trait and a default themed template.
9. Decide whether an existing theme family fits. If not, add a specific theme resolver.
10. Add public re-exports in the control module and `controls/mod.rs`.
11. Add key actions or key contexts only when existing contexts are not sufficient.
12. Add gallery coverage as the first real consumer.
13. Add focused tests for normalization, navigation, value coercion, or event behavior when the
    control owns non-trivial logic.

## Reuse Decision Rules

Reuse `ControlInteraction` when:

- the control has one focusable surface,
- hover and pressed state map directly to the root visual,
- disabled state should remove the tab stop,
- mouse down should focus the control.

Use explicit composite state when:

- the control has multiple semantic child items,
- selection and active descendant are different concepts,
- hover/pressed state is item-specific,
- the template needs a handler per rendered item.
- activation may change app content, but the destination content should decide whether and where to
  take focus.

Join `ButtonFamilyTheme` when:

- the control behaves like a button,
- it has button-like variants and sizes,
- it differs mainly by role, such as text, icon, or selected toggle.

Add a new theme resolver when:

- the control has its own indicator, track, pane, surface, menu item, or value affordance,
- adding another `ButtonFamilyRole` would make the enum vague,
- appearance policy depends on control-specific semantic state.

Use `ControlRange` when:

- the control has numeric min/max values,
- values need finite fallback,
- values need clamp, percentage, or value-at-percentage behavior.

Use `MenuNavigator` when:

- the control navigates an enabled item list,
- disabled items must be skipped,
- navigation should wrap.

## Implementation Checklist

Before considering a new control complete, verify:

- The public API is typed and does not rely on string normalization.
- The control owns behavior and transient interaction state.
- The template owns structure only.
- The theme owns colors, metrics, and state appearance.
- Disabled state prevents activation and cleans up hover/pressed state.
- Pointer and keyboard activation share the same semantic path where applicable.
- Render models are semantic and narrow.
- Programmatic setters notify without emitting user events unless there is a deliberate exception.
- User interactions emit semantic events with enough data for the app to update its state.
- Composite controls skip disabled items during navigation and selection.
- Navigation-style composites keep focus on their own active item unless application or pane code
  explicitly transfers focus to activated content.
- App-owned content, including icons, stays app-owned.
- The gallery demonstrates the control through public SDK APIs.
