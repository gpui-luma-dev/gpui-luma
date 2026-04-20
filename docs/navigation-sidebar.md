# NavigationSidebar Architecture

## Overview

`NavigationSidebar` is a structural routing and focus-coordination control. It replaces rigid
navigation-only row models such as `NavButton`, `NavNodeItem`, and `NavLabel` with a universal
`NavNode` container that renders caller-provided content through a `ContentPresenter`.

In this design, `ContentPresenter` is not raw `AnyElement` introspection. It is an explicit hosted
content contract. A presenter projects custom UI into a sidebar row and returns the GPUI focus target
that the sidebar may use for spatial keyboard traversal.

The sidebar owns:

- region layout: header, main, and footer,
- visible node ordering,
- indentation and depth context,
- branch expansion/collapse state,
- optional spatial Up/Down focus traversal between hosted focus targets,
- row layout bounds through the sidebar template.

Hosted content owns:

- GPUI focus rendering,
- click and keyboard activation,
- hover, pressed, disabled, selected/current-route, checked, open, and editing state,
- command emission and application routing,
- value changes and menus,
- control-specific templates and themes.

`NavigationSidebar` remains a templated control. Its master template, such as
`NavigationSidebarTemplate`, dictates how header, footer, and the scrollable middle node list stack
and flex. Individual `NavNode`s provide hosted content for each row.

## ContentPresenter Contract

The presenter returns a `NavHostedContent` envelope, not a raw element:

```rust
pub struct NavHostedContent {
    pub element: gpui::AnyElement,
    pub focus_handle: Option<gpui::FocusHandle>,
}

pub type NavContentPresenter =
    Arc<dyn Fn(&NavNodeState, &mut Window, &mut App) -> NavHostedContent + Send + Sync>;
```

`element` is the visual projection that the row presenter places inside the sidebar layout.

`focus_handle` is the explicit GPUI focus target used by sidebar-level spatial traversal. If it is
`None`, the node is not a traversal stop. The sidebar must not infer focusability from the returned
element tree.

The sidebar should move focus with the actual GPUI APIs used in this repository, such as
`focus_handle.focus(window, cx)` or `window.focus(&focus_handle, cx)`.

## Stable Hosted Controls

Stateful GPUI controls must be stable entities or stable models owned by the application, not newly
created inside the presenter on every render.

Preferred shape:

```rust
let dashboard_button = Button::new("dashboard").label("Dashboard").spawn(cx);
let dashboard_focus = focus_handle_for(&dashboard_button, cx);

NavNode::new("dashboard").content_presenter({
    let dashboard_button = dashboard_button.clone();
    move |_state, _window, _cx| {
        NavHostedContent {
            element: dashboard_button.clone().into_any_element(),
            focus_handle: Some(dashboard_focus.clone()),
        }
    }
});
```

The exact helper API can be refined during implementation, but the ownership rule is fixed:
presenters may project stable hosted controls, while the controls keep their own identity, state,
subscriptions, and focus behavior across renders.

Inline construction inside a presenter is only appropriate for stateless layout elements or value
objects that are intentionally rebuilt every render.

## API Pattern

```rust
let dashboard_button = Button::new("dashboard").label("Dashboard").spawn(cx);
let settings_disclosure = DisclosureButton::new("settings-toggle").label("Settings").spawn(cx);
let volume_slider = Slider::new("volume").value(50.0).spawn(cx);
let profile_header = ProfileHeader::new("profile-header").spawn(cx);
let user_menu = PopupMenuButton::new("user-menu").label("User").spawn(cx);

let dashboard_focus = focus_handle_for(&dashboard_button, cx);
let settings_focus = focus_handle_for(&settings_disclosure, cx);
let volume_focus = focus_handle_for(&volume_slider, cx);
let profile_focus = focus_handle_for(&profile_header, cx);
let user_menu_focus = focus_handle_for(&user_menu, cx);

NavigationSidebar::new("sidebar")
    .header_node(
        NavNode::new("profile")
            .content_presenter(hosted_entity_presenter(profile_header.clone(), profile_focus.clone())),
    )
    .items([
        NavNode::new("dashboard")
            .content_presenter({
                let dashboard_button = dashboard_button.clone();
                let dashboard_focus = dashboard_focus.clone();
                move |state, _window, _cx| {
                    NavHostedContent {
                        element: div()
                            .flex()
                            .w_full()
                            .items_center()
                            .gap(gpui::px(8.0))
                            .child(render_custom_icon(LucideIcon::Home, state))
                            .child(div().flex_1().child(dashboard_button.clone()))
                            .child(render_custom_icon(LucideIcon::ArrowUpRight, state))
                            .into_any_element(),
                        focus_handle: Some(dashboard_focus.clone()),
                    }
                }
            }),

        NavNode::new("settings-group")
            .content_presenter({
                let settings_disclosure = settings_disclosure.clone();
                let settings_focus = settings_focus.clone();
                move |state, _window, _cx| {
                    NavHostedContent {
                        element: render_disclosure_entity(settings_disclosure.clone(), state.expanded),
                        focus_handle: Some(settings_focus.clone()),
                    }
                }
            })
            .children([
                NavNode::new("volume")
                    .content_presenter(hosted_entity_presenter(volume_slider.clone(), volume_focus.clone())),
            ]),
    ])
    .footer_node(
        NavNode::new("user-menu")
            .content_presenter(hosted_entity_presenter(user_menu.clone(), user_menu_focus.clone())),
    );
```

This sketch is intentionally illustrative. The final SDK helper names may differ, but the API should
preserve these contracts:

- presenters return `NavHostedContent`,
- focus traversal uses `NavHostedContent.focus_handle`,
- hosted controls are stable entities or stable app-owned models,
- click and command behavior stays on hosted controls,
- header and footer use the same node/content contract as the main region.

The helper shape is intentionally simple:

```rust
fn hosted_entity_presenter<T>(
    entity: Entity<T>,
    focus_handle: FocusHandle,
) -> NavContentPresenter
where
    Entity<T>: IntoElement + Clone + 'static,
{
    Arc::new(move |_state, _window, _cx| NavHostedContent {
        element: entity.clone().into_any_element(),
        focus_handle: Some(focus_handle.clone()),
    })
}
```

`focus_handle_for` stands in for the actual SDK helper that reads the control's `Focusable`
implementation after spawning. The important point is that the focus handle is explicit and stable;
the sidebar never tries to discover it from an `AnyElement`.

`render_disclosure_entity` stands in for whatever adapter the SDK provides for projecting sidebar
branch state into a stable disclosure control without mutating that control during render.

## Core Models

### `NavNode`

`NavNode` is the structural skeleton for sidebar rows.

It stores:

- a stable string id,
- a `NavContentPresenter`,
- optional children,
- expansion state for branch nodes,
- enabled/visible structural state if needed by the sidebar,
- layout metadata supplied to `NavNodeState`.

`NavNode` does not own activation semantics by default. Avoid a generic node-level `.on_click(...)`
because that turns the structural row back into a custom button. Leaf activation, commands, value
changes, menu opening, and route changes should come from hosted controls and their existing event
channels.

A future convenience API may wrap a hosted `Button` or `ToggleButton`, but such helpers should create
or receive real hosted controls rather than making `NavNode` itself the interactive control.

### Children And Scope

`NavigationSidebar` should stay a sidebar control, not become a general tree view by accident.

Initial scope:

- top-level nodes,
- one branch level of child nodes,
- header and footer regions using the same node contract.

If arbitrary recursion becomes necessary, the control should explicitly define tree-view behavior:
traversal order, nested expansion semantics, accessibility expectations, virtualization needs, and
how deeply nested hosted controls participate in focus traversal.

### Header And Footer

Header and footer content should use `NavNode` or a compatible `NavHostedContent` region contract.
Raw closures that return only `AnyElement` are allowed only for non-focusable decoration.

If header or footer content can receive focus, it must expose a `FocusHandle` through
`NavHostedContent` so it can participate in the same traversal order as main nodes.

## Structural Traversal

`NavigationSidebar` handles optional spatial Up/Down traversal by iterating visible nodes in rendered
order and moving real GPUI focus to the next node whose latest hosted content includes
`focus_handle: Some(...)`.

Traversal rules:

- only visible nodes are considered,
- collapsed children are skipped,
- nodes with `focus_handle: None` are skipped,
- traversal moves real GPUI focus, not active-descendant row focus,
- traversal must not imply selected/current-route styling,
- when a hosted control enters its own mode, such as text editing or menu navigation, that control
  owns its keys until it yields focus back.

Branch expansion/collapse is sidebar-owned structural state. Branch header activation should still
come from a hosted control, commonly a `ToggleButton` or disclosure button, whose events request the
sidebar to update expansion state.

## Content Presenter Pipeline

`NavRowPresenter` is used by the sidebar template to provide row layout bounds:

- edge-to-edge row width,
- indentation,
- spacing,
- baseline row height where appropriate,
- clipping or scroll-container behavior where appropriate.

`NavRowPresenter` must remain layout-only. It must not synthesize or render:

- hover state,
- pressed state,
- selected/current-route state,
- disabled control state,
- focus-visible state,
- checked/open/editing state.

Those states belong to hosted controls or application routing.

During rendering, `NavRowPresenter` evaluates the node's `content_presenter`, places
`NavHostedContent.element` inside the row layout bounds, and records `NavHostedContent.focus_handle`
for traversal.

## Replacement Rules

- Replace `NavView` with `NavigationSidebar`.
- Do not preserve the old `NavButton`, `NavNodeItem`, or `NavViewEvent::Activate` model.
- Do not use raw `AnyElement` presenters as the focus contract.
- Do not recreate stateful GPUI controls inside presenters on each render.
- Keep route/current state outside the sidebar.
- Keep row wrappers layout-only.
- Use native GPUI focus for hosted controls.
- Use the same hosted-content contract for header, main, and footer regions.
