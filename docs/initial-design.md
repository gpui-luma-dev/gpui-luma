# Design Document: GPUI-Luma SDK

<style>
body {
  font-size: 0.9em;
}

pre,
code {
  color: inherit !important;
}

pre code,
:not(pre) > code {
  font-size: 0.8em;
}

pre code span,
:not(pre) > code span {
  color: inherit !important;
}
</style>

## 1. Project Overview

The GPUI-Luma SDK is a high-performance Rust UI framework built on top of **GPUI**. It separates control behavior from control appearance, with templates providing the architectural seam between them. Templates may translate semantics into a concrete rendering strategy, but their deeper purpose is to preserve clean boundaries between behavior, structure, and styling. Styling and theming remain important, but they are downstream of the control and template contracts rather than foundational assumptions of the architecture.

The project is intended to provide a disciplined foundation for reusable controls, themeable visuals, and strongly typed composition in pure Rust. The goal is to preserve the useful separation between behavior and appearance while keeping the model explicit, static, and easy to reason about.

GPUI-Luma is aimed at controls that need:

- stable behavior independent of a visual skin,
- reusable and swappable presentation layers,
- replaceable appearance systems without stringly typed markup,
- predictable performance and compile-time structure.

### 1.1 Core Principles

- **Separation of Concerns:** Component logic, interaction state, and behavior are distinct from rendering and cosmetic decisions.
- **Performance First:** The system should favor static structure, direct data flow, and minimal runtime indirection.
- **Lookless Architecture:** Controls are defined without a fixed appearance; visuals are supplied by templates.
- **Composable Structure:** Complex controls should be built from explicit parts and typed slots rather than hidden internal trees.
- **Replaceable Appearance:** Templates may target different appearance systems; changing the theme system should not require changing control logic.
- **Predictable Resolution:** State, templates, and appearance values should resolve through a small number of explicit steps.

### 1.2 Scope

GPUI-Luma covers the infrastructure needed to define controls, bind them to visual templates, and connect those templates to an appearance system at runtime.

GPUI-Luma does not attempt to define:

- a markup language,
- a runtime parser,
- reflection-driven property binding,
- a dynamic object model for templates.

---

## 2. Technical Architecture

### 2.1 The Core Model

1. **The Control:** A Rust `struct` that owns state, behavior, input handling, and control-specific logic.
2. **The Template:** A `trait` implementation that defines the rendering boundary for a control and maps control state and semantics into a visual tree.
3. **The Theme System:** A styling system consumed by templates.

These parts have strict responsibilities:

- Controls own behavior.
- Templates own presentation structure and separation of concerns.
- Theme systems own styling data and styling rules.

The model is intentionally asymmetric. Controls must be reusable without any particular theme system. Templates may assume a control contract, but may not add new behavior. Theme systems may provide values and styling conventions, but may not embed control logic.

### 2.2 Control Responsibilities

A control is the authoritative source for:

- user-facing state,
- internal interaction state,
- event handling,
- validation and value coercion,
- public API surface,
- required visual slots,
- semantic parts, variants, and states that templates must understand.

A control must not:

- hardcode colors, spacing, or visual styling,
- depend on a specific template implementation,
- encode layout details that belong to the visual tree,
- depend on a particular theme implementation or theming API.

### 2.3 Template Responsibilities

A template is responsible for:

- rendering the control into GPUI elements,
- choosing structure for the visual tree,
- styling individual parts,
- binding named slots to visual output,
- preserving the boundary between control semantics and appearance implementation.

A template must not:

- own persistent control behavior,
- become the source of truth for interaction state,
- mutate control data as part of presentation,
- reach across control boundaries to coordinate unrelated components.

Templates are the primary design boundary of the system. They keep controls free of presentation code while preventing theme code from leaking into control logic. If a template swap requires a control rewrite, the control boundary is wrong. If a different theme system requires a control rewrite, the template boundary is wrong.

### 2.4 Theme Responsibilities

A theme system is responsible for:

- supplying styling values and styling rules used by templates,
- providing a coherent visual language across controls,
- exposing reusable scales for color, spacing, typography, radius, and motion when those concepts exist,
- supporting the presentation needs of the template layer.

The theme system should remain narrow in scope. It should answer appearance questions for templates, not become a general-purpose service locator.

### 2.5 The Component Handshake

GPUI-Luma uses **trait injection**:

- The control defines a `Slots` struct for the visual pieces it needs.
- The template implementation fulfills the `Slots` and defines the visual structure.
- The template consults a theme system to obtain styling decisions.

The handshake is:

1. The control exposes its state and declares the visual parts it requires.
2. The template receives the control state and binds the declared slots.
3. The template resolves appearance through its chosen theme system.
4. The final visual tree is rendered through GPUI.

This keeps the contract explicit. The control declares what must exist. The template defines how those semantics become a coherent piece of UI. The theme system supplies styling data and rules, but remains downstream of the template contract.

### 2.6 Slots and Parts

Slots are the typed seam between control logic and template structure.

A slot represents a required visual role, not a concrete widget. Examples include:

- icon,
- label,
- track,
- thumb,
- selection fill,
- indicator.

Slots exist so the control can express what it needs without dictating how that need is rendered. Templates remain free to rearrange structure and styling as long as they satisfy the slot contract.

---

## 3. Core Scaffolding Components

### 3.1 ControlTemplate Trait

The fundamental interface for all visual skins.

```rust
pub trait ControlTemplate<C: 'static>: Send + Sync {
    fn render(&self, state: &C, cx: &mut ViewContext<C>) -> impl IntoElement;
}
```

This trait establishes the minimum contract:

- the template is parameterized by the control type,
- the control state is passed directly,
- rendering occurs in a typed GPUI context,
- the output is a GPUI element tree.

The trait should remain small. Any extra behavior added here should support all controls, not a single family of controls. If control-specific concerns start appearing on the base trait, that behavior belongs in a specialized layer.

### 3.2 Visual State Management

A unified system to handle interaction states across all templates.

- **States:** Default, Hover, Active, Focused, Disabled.
- **Logic:** A `StateMixer` helper resolves typed visual values, such as button background or foreground, based on the current interaction.

Visual state management exists to prevent each control or template from inventing its own state-resolution rules.

The state system should:

- normalize common interaction states,
- support composition of mutually relevant flags,
- expose a stable state shape to templates,
- resolve to a single effective visual state when needed.

The state system should not:

- duplicate control logic,
- hide behavior behind implicit rules,
- require every template to reimplement the same precedence order.

State precedence must be explicit. If `Disabled` wins over `Hover`, or `Active` wins over `Focused`, that ordering should be part of the system contract rather than a per-template convention.

### 3.3 Theme Systems

Templates must not hardcode styling literals into the control contract. They may, however, target different theme systems.

Valid theme styles include:

- typed theme keys such as `theme.color(ColorKey::BrandPrimary)`,
- control-family style APIs such as `button_style.background(kind, state)`,
- fixed look packs with no generic theme API,
- other strongly typed appearance models.

The requirement is not "use themes." The requirement is that the theme system be explicit, typed, and isolated behind templates.

When a theme system is used, it should prefer typed keys or control-specific interfaces over string lookup.

Theme systems often expose:

- color,
- spacing,
- typography,
- radius,

- border,
- motion.

Two rules are fundamental:

- controls do not depend on the theme system,
- changing the theme system should only require template changes.

If a theme swap forces changes in control logic, appearance concerns have leaked upward. If a theme system cannot supply what templates need without special cases in controls, the template boundary is too weak.

Theme systems should support:

- shared styling scales where those abstractions are useful,
- control-specific styling decisions where generic scales are too indirect,
- state-aware value resolution,
- fallback behavior or defaults when values are omitted.

The core rule is simple: string-based token lookup is not part of the design. If an appearance value cannot be accessed through a typed Rust API, the theme boundary is too loose.

### 3.4 Verification Architecture

The first verification slice should be a button. The purpose is not to finish a production button. The purpose is to prove the control/template/theme boundary under real interactive behavior and verify that the template is carrying its intended design responsibility.

The button slice should verify:

- the control owns behavior and semantic state,
- the template owns presentation structure and separation of concerns,
- the theme owns resolved appearance policy,
- the theme-driven implementation keeps appearance policy out of control code.

The verification target is successful only if the button logic remains clean while a single well-structured theme-driven implementation drives presentation.

#### Control Contract

The control should expose orthogonal semantic state rather than collapsing state too early.

```rust
use gpui::SharedString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonKind {
    Default,
    Primary,
    Destructive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ButtonState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct ButtonModel {
    pub label: SharedString,
    pub kind: ButtonKind,
    pub size: ButtonSize,
    pub icon: Option<SharedString>,
}

pub struct Button {
    model: ButtonModel,
    state: ButtonState,
}

impl Button {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            model: ButtonModel {
                label: label.into(),
                kind: ButtonKind::Default,
                size: ButtonSize::Md,
                icon: None,
            },
            state: ButtonState::default(),
        }
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.state.hovered = hovered;
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.state.pressed = pressed;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    pub fn set_focus_visible(&mut self, focus_visible: bool) {
        self.state.focus_visible = focus_visible;
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.state.disabled = disabled;
    }

    pub fn render_model(&self) -> ButtonRenderModel<'_> {
        ButtonRenderModel {
            label: &self.model.label,
            icon: self.model.icon.as_deref(),
            kind: self.model.kind,
            size: self.model.size,
            state: self.state,
        }
    }

    pub fn activate(&mut self) -> bool {
        if self.state.disabled {
            return false;
        }

        true
    }
}
```

The important property here is that the control does not calculate colors, spacing, borders, radii, or other appearance values. It only owns semantics.

#### Render Projection

Templates should consume a narrow render projection rather than the full control. This keeps the template boundary stable and makes presentation concerns explicit.

```rust
pub struct ButtonRenderModel<'a> {
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub kind: ButtonKind,
    pub size: ButtonSize,
    pub state: ButtonState,
}
```

This projection is intentionally semantic. It carries:

- parts such as `label` and `icon`,
- variants such as `kind` and `size`,
- interaction flags such as `hovered`, `pressed`, and `focus_visible`.

It does not carry resolved colors or theme-specific handles.

#### Resolved Appearance

The theme system should return a resolved appearance object. That keeps appearance policy out of the template and avoids scattering state matrices across multiple renderers.

```rust
use gpui::Hsla;

pub struct ButtonAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
}

pub trait ButtonTheme: Send + Sync {
    fn resolve(
        &self,
        kind: ButtonKind,
        size: ButtonSize,
        state: ButtonState,
    ) -> ButtonAppearance;
}
```

This is the key refinement: the theme owns the `kind x size x state` policy. The template consumes a fully resolved appearance description and focuses on structure, composition, and boundary discipline.

#### Template Boundary

The template turns the semantic projection plus resolved appearance into GPUI elements while keeping control behavior and appearance policy separate.

```rust
use gpui::{div, px, AnyElement, ViewContext};
use std::sync::Arc;

pub trait ButtonTemplate: Send + Sync {
    fn render(
        &self,
        model: &ButtonRenderModel<'_>,
        cx: &mut ViewContext<Button>,
    ) -> AnyElement;
}

pub struct ThemedButtonTemplate {
    pub theme: Arc<dyn ButtonTheme>,
}

impl ButtonTemplate for ThemedButtonTemplate {
    fn render(
        &self,
        model: &ButtonRenderModel<'_>,
        _cx: &mut ViewContext<Button>,
    ) -> AnyElement {
        let appearance = self
            .theme
            .resolve(model.kind, model.size, model.state);

        let mut root = div()
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .gap(px(appearance.gap))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius));

        if let Some(icon) = model.icon {
            root = root.child(div().child(icon.to_string()));
        }

        root.child(div().child(model.label.to_string())).into_any()
    }
}
```

The template still decides structure:

- where the icon goes,
- how parts are grouped,
- which GPUI elements are emitted.

The template does not decide the appearance matrix for `Default`, `Hover`, `Pressed`, `Disabled`, `Primary`, or `Destructive`. That belongs to the theme.

#### Theme-Backed Proof

The initial implementation only needs one good theme system.

```rust
pub struct ButtonThemePack {
    pub colors: ThemeColors,
    pub metrics: ThemeMetrics,
}

impl ButtonTheme for ButtonThemePack {
    fn resolve(
        &self,
        kind: ButtonKind,
        size: ButtonSize,
        state: ButtonState,
    ) -> ButtonAppearance {
        let background = match (kind, state.disabled, state.pressed, state.hovered) {
            (_, true, _, _) => self.colors.surface_disabled,
            (ButtonKind::Primary, false, true, _) => self.colors.primary_pressed,
            (ButtonKind::Primary, false, false, true) => self.colors.primary_hover,
            (ButtonKind::Primary, false, false, false) => self.colors.primary,
            (_, false, true, _) => self.colors.surface_pressed,
            (_, false, false, true) => self.colors.surface_hover,
            _ => self.colors.surface,
        };

        ButtonAppearance {
            background,
            foreground: if state.disabled {
                self.colors.text_disabled
            } else {
                self.colors.text
            },
            border: self.colors.border,
            focus_ring: state.focus_visible.then_some(self.colors.focus_ring),
            radius: self.metrics.radius(size),
            padding_x: self.metrics.padding_x(size),
            padding_y: self.metrics.padding_y(size),
            gap: self.metrics.gap(size),
        }
    }
}
```

This is enough for the initial proof. The important point is that the button control remains semantic while the theme owns the `kind x size x state` styling policy.

For the button verification slice, that is the bar:

- the control compiles and behaves without styling policy embedded inside it,
- the template can render the button through a single coherent theme,
- the theme can own resolved appearance without infecting control logic,
- the render model remains semantic rather than visual.

### 3.5 Example Usage

The following example is illustrative. It shows the full path of a click event:

1. the rendered control receives a click,
2. the control emits `ButtonEvent::Click`,
3. the app root entity subscribes to that button entity,
4. the app updates its own state in response.

Inside the control, the event originates from the control's click handler:

```rust
use gpui::{ClickEvent, Context, EventEmitter, IntoElement, Window, div};

#[derive(Clone, Debug)]
pub enum ButtonEvent {
    Click,
}

impl EventEmitter<ButtonEvent> for Button {}

impl Button {
    fn handle_click(
        &mut self,
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.activate() {
            cx.emit(ButtonEvent::Click);
        }
    }
}

impl Render for Button {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().on_click(cx.listener(Self::handle_click))
    }
}
```

At the application boundary, `main()` creates the root app entity, and that root entity subscribes to the button's emitted events:

```rust
use gpui::*;
use gpui_luma::{
    controls::button::{Button, ButtonEvent},
    sdk,
};

fn main() {
    let app = gpui_platform::application();

    app.run(|cx| {
        sdk::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(800.0), px(600.0)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            // `main()` does not handle button events directly.
            // It creates the root app entity that will own the subscription.
            let window = cx.open_window(window_options, |_window, cx| {
                cx.new(|cx| ExampleApp::new(cx))
            })?;

            window.update(cx, |_, window, _| {
                window.activate_window();
                window.set_window_title("GPUI-Luma Button Example");
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}

struct ExampleApp {
    button: Entity<Button>,
    _subscriptions: Vec<Subscription>,
}

impl ExampleApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let button = cx.new(|cx| Button::new("example-button").label("Click me").spawn(cx));

        // Subscribe at the app boundary to the semantic event emitted by the control.
        let subscriptions = vec![cx.subscribe(&button, |this, _, event: &ButtonEvent, cx| {
            this.handle_button_event(event, cx);
        })];

        Self {
            button,
            _subscriptions: subscriptions,
        }
    }

    fn handle_button_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                self.button.update(cx, |button, cx| {
                    button.set_label("Clicked", cx);
                });
            }
        }
    }
}

impl Render for ExampleApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .child(self.button.clone())
    }
}
```

The important point is that the event does not bubble to `main()` as a global callback. `main()` creates the root entity, the control emits `ButtonEvent::Click`, and the root entity handles that emitted event through its subscription.

### 3.6 Canonical API

The first implementation should use one canonical API shape. The document currently discusses control semantics, templates, and a simple application example; implementation should treat the builder-style usage shown above as the intended external API.

In that shape, `ButtonEvent` is the outward semantic event surface of the live control. Application code subscribes to those events and responds to `ButtonEvent::Click`, while low-level pointer, keyboard, hover, and pressed transitions remain internal to the control.

For the first slice:

- `Button::new(id)` creates a button model builder.
- Builder methods such as `.label(...)`, `.kind(...)`, and `.enabled(...)` configure public button properties.
- `.spawn(cx)` creates the live control entity.
- The live control may expose a small mutation surface for externally meaningful updates such as `set_label(...)` or `set_enabled(...)`.
- The live control emits semantic events such as `ButtonEvent::Click` for application code to handle.
- Internal interaction state such as hover, press, and focus transitions is not configured by external callers.

This implies a three-layer shape:

- public button properties,
- internal interaction state,
- semantic render projection for templates.

The implementation should not expose public setters for transient interaction state such as `set_pressed(...)` or `set_hovered(...)`. Those belong to event handling inside the control.

### 3.7 First Slice Scope

The first slice should stay deliberately small. Its purpose is to prove the architecture on one useful control, not to finish a general-purpose button system.

In scope for the first slice:

- text label content,
- one click event,
- enabled and disabled behavior,
- focus and focus-visible support,
- one default theme implementation,
- one centered example application,
- label update on click.

Optional but acceptable if they fall out naturally:

- `Default` and `Primary` button kinds,
- a small size set such as `Md` only or `Sm` and `Md`,
- icon support if it does not complicate the contract.

Explicitly deferred for the first slice:

- multiple theme systems,
- loading states,
- icon-only buttons,
- split buttons,
- menus attached to buttons,
- compound content beyond simple label or label-plus-icon,
- advanced animation or motion policy,
- broad control family generalization beyond what the button needs.

If a capability does not directly help prove the control/template/theme boundary on the button, it should be deferred.

### 3.8 Implementation Layout

The first implementation should also have a clear crate and module layout so there is little ambiguity about where each concern belongs.

Suggested layout:

- `crates/sdk`
  Owns the reusable lookless framework and the button verification slice.
- `apps/gallery`
  Acts as the primary example application and the first end-to-end integration surface.
- `apps/examples` or additional small example crates
  Optional home for tiny focused demos if they become useful beyond the gallery.

Suggested SDK layout:

- `crates/sdk/src/init.rs`
  Initializes the SDK, default theme, and any global registrations required for the first slice.
- `crates/sdk/src/controls/button/model.rs`
  Defines public button properties, builder methods, and semantic content.
- `crates/sdk/src/controls/button/control.rs`
  Defines the live button control, event handling, internal interaction state, and emitted events.
- `crates/sdk/src/controls/button/template.rs`
  Defines `ButtonTemplate`, the render projection, and the template-side boundary.
- `crates/sdk/src/theme/button.rs`
  Defines `ButtonAppearance`, `ButtonTheme`, and the first concrete theme implementation.

Suggested example layout:

- `apps/gallery/src/main.rs`
  Boots the application and initializes the SDK.
- `apps/gallery/src/app_shell.rs`
  Holds application-level initialization and window setup helpers.
- `apps/gallery/src/gallery/...`
  Hosts the primary button example and any follow-on exploratory pages.

Responsibility split:

- `crates/sdk` owns the reusable API and architecture.
- `apps/gallery` proves the initial vertical slice end-to-end.
- `model.rs` owns public configuration.
- `control.rs` owns behavior and interaction.
- `template.rs` owns presentation structure.
- `theme/button.rs` owns resolved appearance policy.

Done means:

- the app launches,
- the button renders centered,
- clicking updates the label,
- disabling the button prevents click activation,
- control code does not import theme-specific implementation details.

---

## 4. Implementation Roadmap

The roadmap for the first implementation should stay narrowly focused on a single verification target: the button.

### Button Verification

- [ ] Define the button control contract for parts, variants, and orthogonal state.
- [ ] Establish the `ButtonTemplate` boundary and narrow `ButtonRenderModel`.
- [ ] Implement one theme-driven button template.
- [ ] Implement one button theme with resolved appearance output.
- [ ] Verify that button behavior remains independent of theme policy.
- [ ] Verify that no theme-specific API leaks into control code.

Success criteria:

- the button control owns behavior and semantic state only,
- the button template owns structure and presentation concerns only,
- the theme owns resolved appearance policy,
- one strong theme implementation is sufficient to prove the separation,
- theme policy does not require control changes,
- the verification slice is small enough to expose abstraction problems early.
