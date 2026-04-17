# GPUI-Luma Control Design

## 1. Purpose

This document describes the real control architecture used by GPUI-Luma after the first SDK phase. It replaces the initial scaffold-oriented design notes as the working control design reference.

GPUI-Luma is a Rust control SDK built on top of GPUI. Its control model separates behavior, presentation structure, and appearance policy so that controls remain reusable while templates and themes can evolve independently.

The current SDK includes:

- `Button`
- `IconButton`
- `ToggleButton`
- `Checkbox`
- `Slider`
- `Progress`
- `DropdownMenu`
- `ContextMenu`
- shared button-family theme resolution
- checkbox-specific theme resolution
- slider-specific theme resolution
- progress-specific theme resolution
- dropdown-menu-specific theme resolution
- context-menu-specific theme resolution
- shared interaction-state resolution
- configurable theme tokens
- a gallery app that acts as the first real consumer

This document records the design shape that should be preserved as more controls are added.

## 2. Design Principles

### 2.1 Separation Of Concerns

Controls own behavior. Templates own presentation structure. Themes own appearance policy.

A control should be understandable without reading a theme implementation. A theme should be replaceable without rewriting control behavior. A template should be swappable without changing the public control API.

### 2.2 Lookless Controls

Controls are lookless at the behavior boundary. They do not hardcode colors, spacing, border radii, font weights, or state color matrices.

Controls expose semantic state to templates through render models. Templates convert that semantic state into GPUI elements. Themes resolve visual decisions for templates.

### 2.3 Typed Boundaries

The SDK should prefer typed Rust APIs over stringly typed configuration.

Acceptable examples:

```rust
ButtonKind::Primary
ControlSize::Md
LucideIcon::Plus
ButtonFamilyRole::Icon
```

Avoid designing APIs around arbitrary token names or framework-owned string normalization. If a consumer wants to normalize app-specific strings into icons, variants, or semantic values, that conversion belongs in the consumer.

### 2.4 Shared Behavior Before More Controls

When two controls need the same interaction behavior, that behavior should become shared infrastructure before the third copy appears.

The SDK already has this requirement for:

- hover state
- pressed state
- focus state
- disabled tab-stop behavior
- common interaction-state projection

New controls should use shared interaction helpers rather than copy pointer and focus handling by hand.

### 2.5 Templates Are Structural, Not Behavioral

Templates may choose visual structure:

- where the label appears,
- whether an icon is before or after text,
- how a toggle communicates selection,
- which GPUI nodes are emitted.

Templates must not own persistent control behavior or mutate control state. A template can render state, but it should not become the source of that state.

Controls with template-owned child interaction surfaces can pass control-owned event handlers into the template separately from the render model. `DropdownMenu` uses this shape for trigger and item clicks: the template owns where trigger and item nodes appear, while the control owns what those clicks mean and which semantic event is emitted.

### 2.6 Themes Resolve Appearance Policy

Themes answer visual questions:

- What background does a primary pressed button use?
- What foreground does a disabled icon button use?
- What radius does a medium icon button use?
- What focus-ring color is used when focused?

Themes should not know how to emit events, focus controls, or update application state.

## 3. Architecture

### 3.1 Control Layers

Each control family follows the same broad module split:

- `model.rs`
  Defines public configuration, builder state, render model data, and spawn helpers.
- `control.rs`
  Defines the live GPUI entity, internal behavior, emitted events, and external mutation methods.
- `template.rs`
  Defines the render template trait and default themed template.

Current SDK layout:

```text
crates/sdk/src/controls/
  button/
    control.rs
    model.rs
    template.rs
  icon_button/
    control.rs
    icon.rs
    model.rs
    template.rs
  toggle_button/
    control.rs
    model.rs
    template.rs
  checkbox/
    control.rs
    model.rs
    template.rs
  slider/
    control.rs
    model.rs
    template.rs
  progress/
    control.rs
    model.rs
    template.rs
  dropdown_menu/
    control.rs
    model.rs
    template.rs
  context_menu/
    control.rs
    model.rs
    template.rs
  button_family.rs
  interaction.rs
  value.rs
```

### 3.2 Control Responsibility

A control owns:

- its public model,
- internal interaction state,
- input handling,
- focus behavior,
- validation or value coercion,
- emitted semantic events,
- external mutation methods.

A control must not own:

- color decisions,
- spacing decisions,
- visual state matrices,
- template-specific tree structure,
- application state outside the control.

### 3.3 Template Responsibility

A template owns:

- GPUI element structure,
- slot placement,
- role-specific layout,
- connection to a theme resolver,
- conversion from render model to element tree.

A template receives a semantic render model and returns GPUI elements.

Example shape:

```rust
pub trait ButtonTemplate: Send + Sync {
    fn render(
        &self,
        model: &ButtonRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

The template receives `Window` and `App` because GPUI rendering requires them, but the template should still treat them as rendering context, not as a place to mutate control behavior.

### 3.4 Theme Responsibility

The theme system owns resolved appearance policy.

Current button-family theme shape:

```rust
pub trait ButtonFamilyTheme: Send + Sync {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance;
}
```

The theme is intentionally role-aware. A text button, icon button, and toggle button are part of the same interaction family, but they do not have identical layout or appearance needs.

Current roles:

```rust
pub enum ButtonFamilyRole {
    Text,
    Icon,
    Toggle {
        selected: bool,
    },
}
```

The role lets the same theme resolve family-level concepts without forcing every control into the same shape.

## 4. Public Control APIs

### 4.1 SDK Initialization

SDK initialization is explicit and fallible.

```rust
fn main() {
    gpui_platform::application().run(|cx| {
        if let Err(error) = gpui_luma::init(cx).and_then(|_| app_shell::open(cx)) {
            eprintln!("failed to open GPUI-Luma gallery: {error:?}");
        }
    });
}
```

The SDK currently registers the bundled Lucide icon font because SDK templates can render Lucide-backed icons. Initialization failure must not be swallowed because missing icon fonts cause incorrect rendering.

### 4.2 Button

`Button` uses a builder-style API. `Button::new(id)` creates a builder. `.spawn(cx)` creates the live GPUI entity.

```rust
use gpui_luma::controls::button::{Button, ButtonKind};

let button = Button::new("save-button")
    .label("Save")
    .kind(ButtonKind::Primary)
    .enabled(true)
    .spawn(cx);
```

The public `id` is not the visible label. Visible text is configured with `.label(...)`.

The live control exposes mutation methods for externally meaningful properties:

```rust
button.update(cx, |button, cx| {
    button.set_label("Saved", cx);
    button.set_enabled(false, cx);
});
```

Transient interaction state is internal. Callers do not set hover, pressed, or focused state directly.

### 4.3 IconButton

`IconButton` requires the caller to provide the icon. The SDK does not choose a default icon.

```rust
use gpui_luma::controls::icon_button::{IconButton, IconButtonKind};
use lucide_icons::Icon as LucideIcon;

let icon_button = IconButton::new("add-button", LucideIcon::Plus)
    .kind(IconButtonKind::Primary)
    .enabled(true)
    .spawn(cx);
```

This boundary is intentional:

- SDK consumers own app-level icon choices.
- The SDK owns rendering support for supported icon representations.
- The SDK does not normalize strings such as `"plus"` into Lucide icons.

If a consumer wants string-to-icon conversion, it should implement that conversion outside the SDK:

```rust
use lucide_icons::Icon as LucideIcon;

fn app_icon(name: &str) -> Option<LucideIcon> {
    match name {
        "add" => Some(LucideIcon::Plus),
        "done" => Some(LucideIcon::Check),
        _ => None,
    }
}
```

`IconButton` supports:

- `lucide_icons::Icon`, for bundled Lucide glyph rendering,
- string-like values, for explicit SVG paths owned by the caller.

These are different contracts. A string means a path, not a Lucide name.

### 4.4 ToggleButton

`ToggleButton` owns its selected state and emits semantic change events.

```rust
use gpui_luma::controls::toggle_button::ToggleButton;

let toggle = ToggleButton::new("filter-toggle")
    .label("Filter")
    .selected(false)
    .enabled(true)
    .spawn(cx);
```

The live control exposes selected-state mutation:

```rust
toggle.update(cx, |toggle, cx| {
    toggle.set_selected(true, cx);
});
```

Toggle activation is still semantic. Application code observes `ToggleButtonEvent::Change`.

### 4.5 Checkbox

`Checkbox` owns its checked state and emits semantic change events.

```rust
use gpui_luma::controls::checkbox::Checkbox;

let checkbox = Checkbox::new("remember-choice")
    .label("Remember choice")
    .checked(false)
    .enabled(true)
    .spawn(cx);
```

The live control exposes checked-state mutation:

```rust
checkbox.update(cx, |checkbox, cx| {
    checkbox.set_checked(true, cx);
});
```

Application code observes `CheckboxEvent::Change { checked }`. The checkbox checkmark is an SDK-owned affordance, so callers do not pass an icon for the internal checkmark.

### 4.6 Slider

`Slider` owns its numeric value, coerces it through a configured range and step, and emits semantic change events.

```rust
use gpui_luma::controls::slider::Slider;

let slider = Slider::new("threshold-slider")
    .range(1..100)
    .step(10)
    .value(41)
    .spawn(cx);
```

The live control exposes value, range, step, and enabled-state mutation:

```rust
slider.update(cx, |slider, cx| {
    slider.set_value(51, cx);
    slider.set_range(1..100, cx);
    slider.set_step(10, cx);
});
```

Application code observes `SliderEvent::Change { value }`. Drag and pointer math stay in the control, while the template owns the bar structure and passes measured bounds back through control-owned handlers.

### 4.7 Progress

`Progress` owns a numeric value and range, defaults to `0..100`, and renders as a circular progress indicator.

```rust
use gpui_luma::controls::progress::Progress;

let progress = Progress::new("upload-progress")
    .value(72)
    .spawn(cx);
```

The live control exposes value and range mutation:

```rust
progress.update(cx, |progress, cx| {
    progress.set_value(84, cx);
    progress.set_range(0..100, cx);
});
```

`Progress` is non-interactive status display. It has no hover, pressed, or focus behavior and emits no user events.

### 4.8 DropdownMenu

`DropdownMenu` owns its open state internally and emits semantic select events for enabled menu items.

```rust
use gpui_luma::controls::dropdown_menu::{
    DropdownMenu, DropdownMenuItem,
};
use lucide_icons::Icon as LucideIcon;

let menu = DropdownMenu::new("actions-menu")
    .label("Actions")
    .items([
        DropdownMenuItem::new("new")
            .label("New file")
            .icon(LucideIcon::FilePlus),
        DropdownMenuItem::new("rename")
            .label("Rename")
            .icon(LucideIcon::Pencil),
        DropdownMenuItem::new("archive").label("Archive"),
        DropdownMenuItem::new("share")
            .label("Share")
            .icon(LucideIcon::Share2)
            .submenu([
                DropdownMenuItem::new("copy-link")
                    .label("Copy link")
                    .icon(LucideIcon::Link),
                DropdownMenuItem::new("email")
                    .label("Email")
                    .icon(LucideIcon::Mail),
            ]),
        DropdownMenuItem::new("disabled")
            .label("Unavailable")
            .enabled(false),
    ])
    .enabled(true)
    .spawn(cx);
```

The live control exposes mutation methods for externally meaningful properties:

```rust
menu.update(cx, |menu, cx| {
    menu.set_label("More actions", cx);
    menu.set_enabled(true, cx);
});
```

Application code observes `DropdownMenuEvent::Select { item_id, label }`. The open state and active submenu state are internal interaction state; callers do not set hover, pressed, focused, open, or submenu state directly. The default template renders the menu pane as a deferred overlay anchored to the trigger, so opening the menu does not change the trigger's layout footprint and the pane paints above later page content.

Dropdown item icons are app-owned content. The SDK supports typed `lucide_icons::Icon` values and explicit SVG paths for item icons, but it does not normalize strings into Lucide icon names.

### 4.9 ContextMenu

`ContextMenu` owns its open position and active submenu state internally. It opens from a secondary click on its target and emits semantic select events for enabled menu items.

```rust
use gpui_luma::controls::context_menu::ContextMenu;
use gpui_luma::controls::dropdown_menu::DropdownMenuItem;
use lucide_icons::Icon as LucideIcon;

let menu = ContextMenu::new("file-context-menu")
    .label("Right-click target")
    .items([
        DropdownMenuItem::new("open")
            .label("Open")
            .icon(LucideIcon::FolderOpen),
        DropdownMenuItem::new("copy")
            .label("Copy")
            .icon(LucideIcon::Copy),
        DropdownMenuItem::new("inspect").label("Inspect"),
        DropdownMenuItem::new("more")
            .label("More")
            .icon(LucideIcon::Ellipsis)
            .submenu([
                DropdownMenuItem::new("download")
                    .label("Download")
                    .icon(LucideIcon::Download),
            ]),
    ])
    .enabled(true)
    .spawn(cx);
```

Application code observes `ContextMenuEvent::Select { item_id, label }`. The open position and active submenu state are internal interaction state; callers do not set hover, pressed, focused, open position, or submenu state directly. The default template renders the menu pane as a deferred overlay anchored to the pointer position, so opening the menu does not affect surrounding layout.

## 5. Events And Application Ownership

Controls emit semantic events. Application entities subscribe to those events and own application state changes.

Example:

```rust
use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};

struct ExampleApp {
    button: Entity<Button>,
    clicks: usize,
    _subscriptions: Vec<Subscription>,
}

impl ExampleApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let button = Button::new("button-example")
            .label("Click me")
            .kind(ButtonKind::Primary)
            .spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&button, |this, _, event: &ButtonEvent, cx| {
                this.handle_button_event(event, cx);
            }),
        ];

        Self {
            button,
            clicks: 0,
            _subscriptions: subscriptions,
        }
    }

    fn handle_button_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                self.clicks += 1;
                let label = format!("Clicked {}", self.clicks);

                self.button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }
}
```

Important boundary:

- controls emit semantic events,
- app entities subscribe to those events,
- `main()` only boots the app and opens the root entity,
- application state does not live inside SDK controls.

## 6. Interaction State

Interaction state is shared across controls through `ControlInteraction`.

The public semantic state shape used by themes is:

```rust
pub struct InteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
}
```

The effective visual layer is resolved centrally:

```rust
pub enum InteractionLayer {
    Default,
    Hovered,
    Pressed,
    Disabled,
}
```

Precedence is explicit:

1. Disabled
2. Pressed
3. Hovered
4. Default

Focused state is orthogonal. It is used for focus-ring policy, not as a replacement for hover or pressed state.

### 6.1 Disabled Semantics

Disabled controls must not:

- emit activation events,
- remain pressed or hovered,
- stay in the tab order,
- show focus as active user interaction.

The shared interaction helper is responsible for disabled tab-stop behavior. Controls should not recreate this logic locally.

### 6.2 Keyboard Activation

Controls attach `.on_click(...)` to the focus-tracked GPUI element. GPUI converts Enter and Space on a focused clickable element into keyboard click events.

The SDK should keep keyboard and pointer activation on the same semantic event path:

```rust
fn handle_click(
    &mut self,
    _event: &ClickEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
) {
    if self.model.enabled {
        cx.emit(ButtonEvent::Click);
    }
}
```

Do not create a separate keyboard-only event path unless GPUI behavior requires it.

## 7. Render Models

Templates consume narrow render models instead of full control objects.

Example:

```rust
pub struct ButtonRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub kind: ButtonKind,
    pub size: ButtonSize,
    pub state: ButtonInteractionState,
}
```

Render models should carry:

- control identity needed for element IDs,
- semantic content,
- semantic variants,
- public state,
- projected interaction state.

Render models should not carry:

- resolved colors,
- resolved padding,
- template-owned child trees,
- app callbacks,
- mutable control references.

## 8. Theme Tokens

Theme tokens are data-backed and configurable.

Current token shape:

```rust
pub struct ThemeTokens {
    pub colors: ColorTokens,
    pub metrics: MetricTokens,
}

pub struct MetricTokens {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

pub struct ControlMetricTokens {
    pub radius: f32,
    pub control_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
}
```

`MetricTokens` also exposes convenience accessors such as `radius(size)` and `control_height(size)` so resolvers can stay readable.

The design rule is that shared scales should be configurable without replacing the whole theme resolver.

## 9. Button-Family Theme Resolution

The current button family shares one resolver across text, icon, and toggle controls.

Resolved appearance:

```rust
pub struct ButtonFamilyAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
}
```

The resolver maps:

- variant,
- role,
- size,
- interaction state,

into a concrete appearance.

Text buttons use token radius and horizontal padding. Icon buttons use square sizing with circular radius. Toggle buttons use selected state through `ButtonFamilyRole::Toggle { selected }`.

Example role-specific policy:

```rust
let height = metrics.control_height(size);

let radius = match role {
    ButtonFamilyRole::Icon => height / 2.0,
    _ => metrics.radius(size),
};
```

This keeps shape policy in the family theme instead of hardcoding it in `IconButton`.

### 9.1 Non-Button Theme Families

Controls should only join `ButtonFamilyTheme` when they actually behave like members of the button interaction family. Controls with different shape or value affordance policy should define their own resolver.

`Checkbox` uses:

```rust
pub trait CheckboxTheme: Send + Sync {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance;
}
```

The resolver still uses shared tokens and `InteractionState::layer()` for state precedence, but it owns control-specific policy such as checkbox container background and border, indicator size, checked background, dropdown trigger styling, context target styling, menu surface styling, item hover styling, label color, and affordance color. This keeps non-button controls from expanding `ButtonFamilyRole` into a catch-all enum.

## 10. Icon Policy

The SDK supports icon rendering, but app-level icon choice belongs to the app.

Current icon representation:

```rust
pub enum IconButtonIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}
```

Conversion rules:

- `LucideIcon` becomes `IconButtonIcon::Lucide`.
- `&str`, `String`, and `SharedString` become `IconButtonIcon::SvgPath`.
- The SDK does not convert strings into Lucide icons.

This prevents partial name support from becoming an accidental public contract.

The SDK may use Lucide internally for SDK-owned affordances later, such as dropdown arrows. That is separate from application-owned icon choices.

SDK-owned affordance icons are part of a control's built-in structure. For example, a checked checkbox may render `lucide_icons::Icon::Check` internally because the checkmark communicates checkbox state, and dropdown or context menus may render `lucide_icons::Icon::ChevronDown` and `lucide_icons::Icon::ChevronRight` internally because those chevrons communicate expandable affordances. App-owned icons, such as an add or save icon in an `IconButton` or an item icon in a menu, must still be passed explicitly by the consumer.

## 11. Gallery Boundary

The gallery app is the first real consumer of the SDK. It should demonstrate consumer-owned usage patterns.

Current gallery usage should:

- import Lucide directly when the gallery chooses Lucide icons,
- pass typed Lucide icons to `IconButton`,
- subscribe to semantic SDK events,
- avoid relying on SDK string normalization.

Example:

```rust
use gpui_luma::controls::icon_button::{IconButton, IconButtonKind};
use lucide_icons::Icon as LucideIcon;

let icon_button = IconButton::new("icon-button-example", LucideIcon::Plus)
    .kind(IconButtonKind::Primary)
    .spawn(cx);
```

The gallery is allowed to have app-specific conveniences. Those conveniences should not leak into the SDK as general policy.

## 12. Adding New Controls

When adding a new control:

1. Define the public model and builder.
2. Define the semantic event surface.
3. Use shared interaction behavior where possible.
4. Define a narrow render model.
5. Define a template trait for the control.
6. Decide whether the control fits an existing theme family.
7. If it does not fit, create a new family resolver rather than forcing it into the wrong one.
8. Add gallery coverage as the first consumer.

New controls should not copy an existing control and edit until it works. If interaction, sizing, or theme policy repeats, extract the shared layer first.

## 13. Current Non-Goals

The current SDK design does not include:

- a markup language,
- runtime template parsing,
- reflection-driven property binding,
- CSS-like string token lookup,
- global app-owned state management,
- automatic icon-name normalization,
- multiple theme engines.

These may be revisited later, but they are not part of the current control design.

## 14. Design Checklist

Before considering a control design complete, verify:

- The control can be understood without reading the theme.
- The theme can be changed without rewriting control behavior.
- The template can be changed without changing the public model.
- Transient interaction state is internal.
- Disabled state prevents activation and removes tab-stop behavior.
- Keyboard and pointer activation share the same semantic event path.
- Render models are semantic, not visual.
- App-level choices stay in the app.
- SDK-level defaults do not silently choose application content.
- Shared behavior is extracted before it becomes duplicated.
