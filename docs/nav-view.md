# Nav View Design

This document defines the first Luma navigation collection control: `NavView`.

`NavView` is being replaced by `NavigationSidebar`; see `navigation-sidebar.md`. That document
reframes navigation as a hierarchical sidebar of hosted controls rather than a widget that owns
custom button-like rows.

`NavView` is intentionally not a full arbitrary `TreeView`. It is a focused navigation control for
application sidebars, beginning with the gallery app. It should support grouped navigation,
selection, simple disclosure, and event emission without trying to model every possible tree-like
data structure.

## Goal

Build a Luma-native navigation view for app sidebars.

The first consumer is the gallery sidebar inside `SplitView`.

The initial control should provide:

- top-level navigation buttons,
- read-only category labels,
- single-level collapsible nodes,
- selectable child items inside nodes,
- bottom-aligned navigation buttons,
- stable item ids,
- selected item state,
- click activation,
- basic keyboard navigation when practical,
- semantic selection/toggle events,
- control-level template customization,
- item-level template customization.

The control should help the gallery move from a placeholder sidebar to a real page navigator.

## Non-Goals

The first version is not:

- a generic arbitrary-depth tree view,
- a virtualized file explorer,
- a table/tree hybrid,
- a data grid,
- a drag-and-drop outline editor,
- a complete accessibility framework,
- a port of Opal's `NavigationMenuTree`.

It can share ideas with tree controls, but it should not promise arbitrary tree capability.

## Nav View vs Tree View

`NavView` and `TreeView` overlap, but they have different contracts.

`NavView` is for application navigation:

- predictable sidebar structure,
- usually shallow depth,
- top-level buttons, labels, and nodes,
- selected route/page,
- optional node disclosure,
- app command invocation,
- navigation-specific styling.

`TreeView` is for arbitrary hierarchical data:

- arbitrary nesting,
- generic node expansion,
- potentially large data sets,
- richer item types,
- data inspection,
- possible virtualization,
- more complex keyboard and accessibility behavior.

Luma should start with `NavView` because the gallery needs navigation now, and because a focused
control will reveal the right collection templating shape before a broader tree view exists.

## Initial Scope

The first `NavView` should support the app-sidebar shape shown in the current Codex-style sidebar:

```text
button
button
label
node
  item
  item
node
  item
label
button
---
bottom button
```

The first version supports exactly these item roles:

- **Button**: top-level interactive row. It can represent a route or command.
- **Label**: top-level read-only category text. It is not focusable and emits no events.
- **Node**: top-level collapsible row with one level of child items.
- **Item**: selectable child row inside a node.
- **Bottom Button**: a button rendered in a bottom-aligned action area, such as Settings.

The first version should limit depth to two levels:

- top-level buttons, labels, and nodes,
- child items inside nodes.

Nodes may not contain nodes in the first version. If the gallery later needs nested subsections, add
one deliberate layer before generalizing to an arbitrary tree.

Bottom-aligned buttons should not be modeled as nested tree content. They are part of the sidebar
layout contract: primary navigation scrolls or fills above, and persistent utility actions sit at
the bottom.

## Model Sketch

The model should describe navigation structure and item semantics.

```rust
pub struct NavViewModel {
    id: SharedString,
    items: Vec<NavItem>,
    bottom_items: Vec<NavButton>,
    selected_item_id: Option<SharedString>,
    enabled: bool,
    template: Arc<dyn NavViewTemplate>,
    item_template: Arc<dyn NavItemTemplate>,
}
```

Possible item shape:

```rust
pub enum NavItem {
    Button(NavButton),
    Label(NavLabel),
    Node(NavNode),
}

pub struct NavButton {
    id: SharedString,
    label: SharedString,
    enabled: bool,
}

pub struct NavLabel {
    label: SharedString,
}

pub struct NavNode {
    id: SharedString,
    label: SharedString,
    children: Vec<NavNodeItem>,
    expanded: bool,
    enabled: bool,
}

pub struct NavNodeItem {
    id: SharedString,
    label: SharedString,
    enabled: bool,
}
```

This structure is intentionally not recursive in the first version.

## Builder Sketch

The builder should be simple and deterministic, matching the rest of the SDK:

```rust
let nav_view = NavView::new("gallery-nav")
    .items([
        NavItem::button("all-controls").label("All Controls"),
        NavItem::button("search").label("Search"),
        NavItem::label("Controls"),
        NavItem::node("command")
            .label("Command")
            .expanded(true)
            .children([
                NavNodeItem::new("button").label("Button"),
                NavNodeItem::new("icon-button").label("Icon Button"),
            ]),
        NavItem::node("choice")
            .label("Choice")
            .expanded(true)
            .children([
                NavNodeItem::new("checkbox").label("Checkbox"),
                NavNodeItem::new("radio-group").label("Radio Group"),
            ]),
    ])
    .bottom_items([
        NavButton::new("settings").label("Settings"),
    ])
    .selected("button")
    .spawn(cx);
```

The exact constructors can be refined during implementation. The key point is that app code should
provide stable ids and labels without encoding route logic into the control.

## Events

Events should be semantic and route-oriented.

```rust
pub enum NavViewEvent {
    Activate { item_id: SharedString },
    ToggleNode { node_id: SharedString, expanded: bool },
}
```

Activation should only fire for enabled buttons and enabled node items.

Node toggles should only fire for enabled nodes.

The app owns page routing. `NavView` emits activated ids; the gallery registry maps ids to pages or
commands.

## State Rules

- The selected item id should be stable and externally meaningful.
- Top-level labels are read-only, non-focusable, and never selected.
- Disabled buttons and node items cannot be activated by pointer or keyboard.
- Disabled nodes cannot be toggled.
- Collapsing a node should not clear selected state by itself.
- If the selected item is inside a collapsed node, the model may keep the selection but the item
  is not visible.
- App code may decide to auto-expand the selected node; the control should not force that policy
  in the first version.
- Replacing items should preserve selected id only if that id still exists and is enabled.

## First Collection Control Considerations

`NavView` will be the first collection-style SDK control. It should establish patterns that can be
reused by later controls without over-generalizing too early.

Important collection concerns:

- item identity,
- item state projection,
- active/focused item tracking,
- event routing by item id,
- repeated item rendering,
- item-level template customization,
- control-level template customization,
- eventual keyboard navigation across visible items.

Avoid inventing a generic collection framework in this first pass. Extract shared helpers only after
`NavView` and at least one other collection-like control show real duplication.

## Template Design

`NavView` needs two template layers:

1. control template,
2. item template.

The control template owns the outer structure:

- root container,
- main navigation region,
- bottom action region,
- top-level row layout,
- spacing,
- scroll/fill behavior if needed,
- deciding when to render children.

The item template owns repeated row rendering:

- top-level button row,
- read-only label row,
- collapsible node row,
- child item row,
- selected state visuals,
- hover/pressed/focus visuals,
- disclosure affordance for collapsible nodes,
- optional icon/accessory slots later.

Possible traits:

```rust
pub trait NavViewTemplate: Send + Sync {
    fn render(
        &self,
        model: &NavViewRenderModel<'_>,
        handlers: NavViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub trait NavItemTemplate: Send + Sync {
    fn render_button(
        &self,
        button: &NavButtonRenderModel<'_>,
        handlers: NavButtonTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_label(
        &self,
        label: &NavLabelRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_node(
        &self,
        node: &NavNodeRenderModel<'_>,
        handlers: NavNodeTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_node_item(
        &self,
        item: &NavNodeItemRenderModel<'_>,
        handlers: NavNodeItemTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}
```

This split keeps navigation-specific behavior in the control while letting apps customize row
appearance without replacing the whole control.

## Render Model Sketch

Render models should be precomputed enough that templates do not need to understand internal
selection or expansion mechanics.

```rust
pub struct NavViewRenderModel<'a> {
    id: &'a SharedString,
    items: &'a [NavItem],
    bottom_items: &'a [NavButton],
    selected_item_id: Option<&'a SharedString>,
    active_item_id: Option<&'a SharedString>,
    enabled: bool,
}

pub struct NavButtonRenderModel<'a> {
    id: &'a SharedString,
    label: &'a SharedString,
    selected: bool,
    active: bool,
    enabled: bool,
    depth: usize,
}

pub struct NavLabelRenderModel<'a> {
    label: &'a SharedString,
    depth: usize,
}

pub struct NavNodeRenderModel<'a> {
    id: &'a SharedString,
    label: &'a SharedString,
    expanded: bool,
    active: bool,
    enabled: bool,
    depth: usize,
}

pub struct NavNodeItemRenderModel<'a> {
    id: &'a SharedString,
    label: &'a SharedString,
    selected: bool,
    active: bool,
    enabled: bool,
    depth: usize,
}
```

The first version may keep render models simpler if pointer-only behavior ships before keyboard
navigation.

## Keyboard Navigation

Keyboard support is important, but it can be phased.

First useful target:

- `Up` / `Down`: move active row among visible enabled rows,
- `Enter` / `Space`: activate active button/item or toggle active node,
- `Right`: expand active node,
- `Left`: collapse active node,
- `Home` / `End`: move to first/last visible enabled row.

Read-only labels should be skipped by focus and active-row movement. Bottom buttons should
participate in keyboard navigation after the main visible rows, even though the template renders
them in a separate bottom region.

The control should eventually use the existing keyhandling profile model rather than ad hoc key
strings. If that slows the first pass too much, ship pointer interaction first and add keyboard in a
follow-up before calling the control SDK-complete.

## Gallery Integration

The first gallery integration should be:

```text
Gallery registry
  page id
  title
  category
  render route

SplitView sidebar
  NavView from registry categories

SplitView content
  selected page
```

Recommended implementation sequence:

1. Add gallery `registry.rs` with page ids and categories.
2. Add gallery-local navigation data derived from the registry.
3. Add SDK `NavView` with top-level buttons, labels, and single-level nodes.
4. Wire `NavViewEvent::Select` to `GalleryApp.selected_page`.
5. Render the selected page in the split-view content pane.
6. Move current demo stack into one or more initial pages.

If this feels too large, start with gallery-local `navigation.rs` and promote the API into SDK after
one iteration. The control should still be designed as if it will become SDK-owned.

## Opal Navigation Prior Art

Opal's `NavigationMenuTree` demonstrates useful behaviors:

- minimal tree,
- full interaction prototype,
- long-scroll stress case,
- programmatic command selection,
- programmatic command invocation,
- programmatic toggle changes,
- programmatic branch expansion,
- command/accessory/toggle telemetry.

Useful ideas to reuse:

- stable node ids,
- event source distinctions later if needed,
- explicit selected command APIs,
- explicit expanded state APIs,
- stress coverage with long navigation lists.

Avoid carrying over:

- arbitrary recursive tree structure in the first version,
- command/accessory/toggle complexity before the gallery needs it,
- macro-heavy authoring,
- generic tree naming if the control is navigation-specific.

## Later Scope

Likely later additions:

- icons,
- badges,
- item accessories,
- label or node header actions,
- search filtering,
- disabled item reasons/tooltips,
- activation source in events,
- programmatic invoke separate from select,
- controlled expansion state,
- nested subsections,
- scroll-to-selected,
- typeahead,
- richer accessibility semantics.

These should be added only when a real gallery or app workflow needs them.

## Open Questions

- Should expansion state be owned internally, externally controlled, or both?
- Should selection state be owned internally or controlled by the gallery registry?
- Should item templates be one trait with methods for each row kind, or separate traits for buttons,
  labels, nodes, and node items?
- Should icons be part of the first model, or should the gallery wait until the visual language
  settles?
- Should activation source be included in first events, or deferred until programmatic invocation
  exists?

## Decision Bias

Build the smallest navigation control that makes the gallery shell real.

Do not build an arbitrary tree view yet. Use `NavView` to learn the collection control patterns Luma
needs, especially item identity, repeated row rendering, and item-level templates.
