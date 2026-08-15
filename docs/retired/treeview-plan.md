# TreeView Control Specification

This document details the interface, design, and virtualization model for a new performance-oriented **TreeView** control in `gpui-luma`. 

A TreeView represents hierarchical data, such as a file explorer tree. To support deep nesting and thousands of nodes efficiently without UI lag, the control flattens the hierarchy and feeds rows to GPUI's virtualized scrolling state (`gpui::ListState`).

---

## Architecture & Virtualization Design

To handle virtualization, the TreeView control relies on a **Flattening Engine** that converts the nested node tree into a single-dimensional vector of visible nodes based on expansion state:

```
Nested Tree Structure:
├── Folder A (Expanded)
│   ├── File 1
│   └── File 2
└── Folder B (Collapsed)
    └── File 3 (Hidden)

Flattened Vector (Visible Rows):
[
  0: Folder A (depth: 0, expanded: true, has_children: true)
  1: File 1   (depth: 1, expanded: false, has_children: false)
  2: File 2   (depth: 1, expanded: false, has_children: false)
  3: Folder B (depth: 0, expanded: false, has_children: true)
]
```

This flattened index is used by GPUI's built-in `list(...)` component, ensuring only visible items in the viewport are rendered.

---

## File Structure

```
crates/sdk/src/controls/tree_view/
├── mod.rs        # Public API exports and types
├── model.rs      # TreeNode structures, builders, and render models
├── control.rs    # Flattening engine, ListState view entity, and arrow navigation
├── template.rs   # Layout rendering, indentation padding, trigger hooks
└── theme.rs      # Styling palettes and pixel-snapped depth spacing scales
```

---

## 1. Public API Interface (`mod.rs`)

```rust
mod control;
mod model;
mod template;
mod theme;

pub use control::{TreeViewControl, TreeViewEvent};
pub use model::{
    TreeViewBuilder, TreeNode, FlatTreeNode, TreeViewModel,
    TreeViewRenderModel, TreeViewSelectionMode,
};
pub use template::{
    TreeViewTemplate, TreeViewTemplateHandlers, ThemedTreeViewTemplate,
    default_tree_view_template,
)
pub use theme::{
    TreeViewTheme, TreeViewPalette, TreeViewScale, DefaultTreeViewTheme,
    default_tree_view_theme,
};

pub use crate::controls::state::{CompositeItemState as TreeViewItemState, ControlFocusState};
use gpui::{Entity, SharedString};

pub type TreeView<T> = Entity<TreeViewControl<T>>;

/// Creates a new builder for a TreeView.
pub fn new<T>(id: impl Into<SharedString>) -> TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    TreeViewBuilder::new(id)
}
```

---

## 2. Models and Builder (`model.rs`)

Declares the nested `TreeNode` dynamic schema, flattening snapshots, and fluent builders.

```rust
use std::sync::Arc;
use gpui::{AnyElement, AppContext, Entity, IntoElement, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{TreeViewControl, TreeViewTemplate, default_tree_view_template};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeViewSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}

/// Represents a node in the tree structure.
#[derive(Clone)]
pub struct TreeNode<T> {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<LucideIcon>,
    pub data: T,
    pub children: Vec<TreeNode<T>>,
    pub is_branch: bool, // Force folder chevron even if children are loaded lazily
    pub expanded: bool,
    pub enabled: bool,
}

impl<T> TreeNode<T> {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, data: T) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            data,
            children: Vec::new(),
            is_branch: false,
            expanded: false,
            enabled: true,
        }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn branch(mut self, is_branch: bool) -> Self {
        self.is_branch = is_branch;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.children = children.into_iter().collect();
        self.is_branch = !self.children.is_empty() || self.is_branch;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A read-only snapshot representing a single item in the virtual list.
pub struct FlatTreeNode<'a, T> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub icon: Option<LucideIcon>,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub enabled: bool,
    pub state: crate::controls::state::CompositeItemState,
    pub data: &'a T,
}

#[derive(Clone)]
pub struct TreeViewModel<T> {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<TreeNode<T>>,
    pub(crate) selection_mode: TreeViewSelectionMode,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn TreeViewTemplate>,
}

pub struct TreeViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub selection_mode: TreeViewSelectionMode,
    pub enabled: bool,
    pub focus: crate::controls::state::ControlFocusState,
}

pub struct TreeViewBuilder<T> {
    pub(crate) model: TreeViewModel<T>,
}

impl<T> TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TreeViewModel {
                id: id.into(),
                items: Vec::new(),
                selection_mode: TreeViewSelectionMode::Single,
                enabled: true,
                template: default_tree_view_template(),
            },
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.model.selection_mode = mode;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn TreeViewTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TreeViewControl<T>> {
        cx.new(|cx| TreeViewControl::from_builder(self, cx))
    }
}
```

---

## 3. Virtualization & Keyboard Handling (`control.rs`)

Responsible for generating the flat layout cache and handling hierarchical keyboard actions.

```rust
use std::collections::HashSet;
use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, FocusHandle,
    IntoElement, ListState, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Window, div, prelude::*,
};

use super::{
    TreeViewBuilder, TreeViewEvent, FlatTreeNode, TreeViewModel,
    TreeViewRenderModel, TreeViewTemplateHandlers,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};

// Custom keyboard actions for collapsing/expanding folders
gpui::actions!(tree, [ExpandNode, CollapseNode]);

pub enum TreeViewEvent<T> {
    NodeExpanded { node_id: SharedString, data: T },
    NodeCollapsed { node_id: SharedString, data: T },
    SelectionChanged { selected_ids: HashSet<SharedString> },
}

pub struct TreeViewControl<T> {
    model: TreeViewModel<T>,
    focus_handle: FocusHandle,
    list_state: ListState,
    expanded_ids: HashSet<SharedString>,
    selected_ids: HashSet<SharedString>,
    active_index: Option<usize>, // Selected index in the flat list
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
    flat_cache: Vec<FlatNodeWrapper<T>>,
}

// Flat representation stored in control state
struct FlatNodeWrapper<T> {
    id: SharedString,
    label: SharedString,
    icon: Option<lucide_icons::Icon>,
    depth: usize,
    has_children: bool,
    enabled: bool,
    data: T,
}

impl<T> EventEmitter<TreeViewEvent<T>> for TreeViewControl<T> where T: 'static {}

impl<T> TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) fn from_builder(builder: TreeViewBuilder<T>, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        
        let mut control = Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            list_state: ListState::new(0),
            expanded_ids: HashSet::new(),
            selected_ids: HashSet::new(),
            active_index: None,
            hovered_index: None,
            pressed_index: None,
            flat_cache: Vec::new(),
        };

        // Populate initial expanded states from item model
        control.populate_initial_expands();
        control.rebuild_flat_cache(cx);

        control
    }

    fn populate_initial_expands(&mut self) {
        fn traverse<T>(node: &TreeNode<T>, expanded: &mut HashSet<SharedString>) {
            if node.expanded {
                expanded.insert(node.id.clone());
            }
            for child in &node.children {
                traverse(child, expanded);
            }
        }
        for item in &self.model.items {
            traverse(item, &mut self.expanded_ids);
        }
    }

    fn rebuild_flat_cache(&mut self, cx: &mut Context<Self>) {
        let mut flat = Vec::new();
        
        fn flatten<T: Clone>(
            node: &TreeNode<T>,
            depth: usize,
            expanded: &HashSet<SharedString>,
            flat: &mut Vec<FlatNodeWrapper<T>>,
        ) {
            let node_expanded = expanded.contains(&node.id);
            let has_children = !node.children.is_empty() || node.is_branch;

            flat.push(FlatNodeWrapper {
                id: node.id.clone(),
                label: node.label.clone(),
                icon: node.icon,
                depth,
                has_children,
                enabled: node.enabled,
                data: node.data.clone(),
            });

            if node_expanded {
                for child in &node.children {
                    flatten(child, depth + 1, expanded, flat);
                }
            }
        }

        for item in &self.model.items {
            flatten(item, 0, &self.expanded_ids, &mut flat);
        }

        self.flat_cache = flat;
        self.list_state.reset(self.flat_cache.len());
        cx.notify();
    }

    pub fn toggle_node_expand(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.flat_cache.len() {
            return;
        }
        let wrapper = &self.flat_cache[index];
        if !wrapper.has_children {
            return;
        }

        let id = wrapper.id.clone();
        let data = wrapper.data.clone();
        if self.expanded_ids.contains(&id) {
            self.expanded_ids.remove(&id);
            cx.emit(TreeViewEvent::NodeCollapsed { node_id: id, data });
        } else {
            self.expanded_ids.insert(id.clone());
            cx.emit(TreeViewEvent::NodeExpanded { node_id: id, data });
        }

        self.rebuild_flat_cache(cx);
    }

    fn render_row(&mut self, local_index: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(node) = self.flat_cache.get(local_index) else {
            return div().into_any_element();
        };

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let selected = self.selected_ids.contains(&node.id);
        let active = self.active_index == Some(local_index);
        let hovered = self.model.enabled && self.hovered_index == Some(local_index);
        let pressed = self.model.enabled && self.pressed_index == Some(local_index);
        
        let state = CompositeItemState {
            hovered,
            pressed,
            disabled: !node.enabled,
            selected,
            active: focus.focused && active,
            focus_visible: focus.focus_visible && active,
        };

        let render_node = FlatTreeNode {
            id: &node.id,
            label: &node.label,
            icon: node.icon,
            depth: node.depth,
            has_children: node.has_children,
            expanded: self.expanded_ids.contains(&node.id),
            enabled: node.enabled,
            state,
            data: &node.data,
        };

        // Delegate row render to template helper
        self.model.template.render_node(
            &render_node,
            self.template_row_handlers(local_index, cx),
            window,
            cx,
        )
    }

    fn template_row_handlers(&self, idx: usize, cx: &mut Context<Self>) -> TreeViewTemplateHandlers {
        TreeViewTemplateHandlers {
            hover: Box::new(cx.listener(move |this, hovered, _, cx| {
                if this.model.enabled {
                    this.hovered_index = hovered.then_some(idx);
                    cx.notify();
                }
            })),
            mouse_down: Box::new(cx.listener(move |this, _, window, cx| {
                if this.model.enabled {
                    this.pressed_index = Some(idx);
                    this.active_index = Some(idx);
                    this.focus_handle.focus(window, cx);
                    cx.notify();
                }
            })),
            mouse_up: Box::new(cx.listener(move |this, _, _, cx| {
                this.pressed_index = None;
                cx.notify();
            })),
            click: Box::new(cx.listener(move |this, _, _, cx| {
                this.handle_node_select(idx, cx);
            })),
            chevron_click: Box::new(cx.listener(move |this, _, _, cx| {
                this.toggle_node_expand(idx, cx);
            })),
        }
    }

    fn handle_node_select(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx >= self.flat_cache.len() { return; }
        let node = &self.flat_cache[idx];
        if !node.enabled { return; }

        let id = node.id.clone();
        match self.model.selection_mode {
            TreeViewSelectionMode::None => {},
            TreeViewSelectionMode::Single => {
                self.selected_ids.clear();
                self.selected_ids.insert(id);
            }
            TreeViewSelectionMode::Multiple => {
                if self.selected_ids.contains(&id) {
                    self.selected_ids.remove(&id);
                } else {
                    self.selected_ids.insert(id);
                }
            }
        }
        cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        cx.notify();
    }

    // Keyboard handlers
    fn handle_previous(&mut self, _: &SelectPreviousItem, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = self.active_index {
            if active > 0 {
                self.active_index = Some(active - 1);
                self.list_state.scroll_to_reveal_item(active - 1);
                cx.notify();
            }
        } else if !self.flat_cache.is_empty() {
            self.active_index = Some(0);
            cx.notify();
        }
    }

    fn handle_next(&mut self, _: &SelectNextItem, _: &mut Window, cx: &mut Context<Self>) {
        let max_idx = self.flat_cache.len().saturating_sub(1);
        if let Some(active) = self.active_index {
            if active < max_idx {
                self.active_index = Some(active + 1);
                self.list_state.scroll_to_reveal_item(active + 1);
                cx.notify();
            }
        } else if !self.flat_cache.is_empty() {
            self.active_index = Some(0);
            cx.notify();
        }
    }

    fn handle_expand_focused_node(&mut self, _: &ExpandNode, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_index {
            let node = &self.flat_cache[idx];
            if node.has_children && !self.expanded_ids.contains(&node.id) {
                self.toggle_node_expand(idx, cx);
            } else if node.has_children {
                // Focus moves to first child
                if idx + 1 < self.flat_cache.len() {
                    self.active_index = Some(idx + 1);
                    self.list_state.scroll_to_reveal_item(idx + 1);
                    cx.notify();
                }
            }
        }
    }

    fn handle_collapse_focused_node(&mut self, _: &CollapseNode, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_index {
            let node = &self.flat_cache[idx];
            if node.has_children && self.expanded_ids.contains(&node.id) {
                self.toggle_node_expand(idx, cx);
            } else if node.depth > 0 {
                // Move focus to parent node
                let current_depth = node.depth;
                let mut search_idx = idx;
                while search_idx > 0 {
                    search_idx -= 1;
                    if self.flat_cache[search_idx].depth < current_depth {
                        self.active_index = Some(search_idx);
                        self.list_state.scroll_to_reveal_item(search_idx);
                        cx.notify();
                        break;
                    }
                }
            }
        }
    }

    fn handle_activate(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_index {
            self.handle_node_select(idx, cx);
        }
    }
}

impl<T> Focusable for TreeViewControl<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_model = TreeViewRenderModel {
            id: &self.model.id,
            selection_mode: self.model.selection_mode,
            enabled: self.model.enabled,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window),
        };

        let body = list(self.list_state.clone(), cx.processor(Self::render_row)).size_full().into_any_element();

        self.model
            .template
            .render(&render_model, body, window, cx)
            .track_focus(&self.focus_handle)
            .key_context(ControlKeyProfile::Selector.context())
            .on_action(cx.listener(Self::handle_previous))
            .on_action(cx.listener(Self::handle_next))
            .on_action(cx.listener(Self::handle_expand_focused_node))
            .on_action(cx.listener(Self::handle_collapse_focused_node))
            .on_action(cx.listener(Self::handle_activate))
    }
}
```

---

## 4. Layout Rendering Template (`template.rs`)

Handles the visual nesting offsets, item trigger rendering, and directory icons.

```rust
use std::sync::{Arc, OnceLock};
use gpui::{
    AnyElement, App, ClickEvent, Div, MouseDownEvent, MouseUpEvent,
    Stateful, Window, div, px, prelude::*,
};
use lucide_icons::Icon as LucideIcon;

use super::{TreeViewRenderModel, FlatTreeNode, TreeViewTheme, default_tree_view_theme};
use crate::theme::{ControlSize, LayoutCacheKey, LumaLayoutCacheExt};
use crate::define_control_template;

pub type TreeViewHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TreeViewMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TreeViewMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type TreeViewClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

pub struct TreeViewTemplateHandlers {
    pub hover: TreeViewHoverHandler,
    pub mouse_down: TreeViewMouseDownHandler,
    pub mouse_up: TreeViewMouseUpHandler,
    pub click: TreeViewClickHandler,
    pub chevron_click: TreeViewClickHandler,
}

pub trait TreeViewTemplate: Send + Sync {
    /// Renders the virtualized shell container.
    fn render(
        &self,
        model: &TreeViewRenderModel,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    /// Renders an individual node item.
    fn render_node(
        &self,
        node: &FlatTreeNode<'_>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;
}

define_control_template!(
    ThemedTreeViewTemplate,
    dyn TreeViewTheme,
    TreeViewRenderModel<'static>,
    TreeViewTemplate,
    default_tree_view_theme()
);

impl TreeViewTemplate for ThemedTreeViewTemplate {
    fn render(
        &self,
        model: &TreeViewRenderModel,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        div()
            .id(model.id.clone())
            .size_full()
            .overflow_hidden()
            .child(body)
    }

    fn render_node(
        &self,
        node: &FlatTreeNode<'_>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| super::theme::TreeViewScale::compute(ControlSize::Md, metrics, scale_factor),
        );

        let palette = self.theme.resolve_row(node.state, node.state.selected);
        let left_padding = scale.base_padding_x + (node.depth as f32 * scale.indentation_width);

        let mut row = div()
            .id(format!("{}-row", node.id))
            .flex()
            .items_center()
            .w_full()
            .h(px(scale.row_height))
            .pl(px(left_padding))
            .pr(px(scale.base_padding_x))
            .rounded(px(scale.radius))
            .bg(palette.background)
            .text_color(palette.foreground)
            .text_size(px(palette.typography.size))
            .line_height(px(palette.typography.line_height))
            .font_family(palette.font_family.clone())
            .font_weight(palette.typography.weight)
            .cursor_pointer();

        if node.enabled {
            row = row
                .on_hover(handlers.hover)
                .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
                .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
                .on_click(handlers.click);
        } else {
            row = row.opacity(0.40);
        }

        // 1. Disclosure chevron trigger (only if it has child items)
        if node.has_children {
            let chevron_icon = if node.expanded { LucideIcon::ChevronDown } else { LucideIcon::ChevronRight };
            let chevron = div()
                .size(px(scale.chevron_size))
                .flex()
                .items_center()
                .justify_center()
                .child(render_icon(chevron_icon, palette.chevron_color, scale.chevron_size))
                .on_click(handlers.chevron_click);
            
            row = row.child(chevron);
        } else {
            // Spacer to keep file and folder icons aligned
            row = row.child(div().size(px(scale.chevron_size)));
        }

        // Inner layout gap
        row = row.child(div().w(px(scale.inner_gap)));

        // 2. Node file/folder icon
        let display_icon = node.icon.unwrap_or_else(|| {
            if node.has_children {
                if node.expanded { LucideIcon::FolderOpen } else { LucideIcon::Folder }
            } else {
                LucideIcon::File
            }
        });
        row = row.child(render_icon(display_icon, palette.icon_color, scale.icon_size));
        
        row = row.child(div().w(px(scale.inner_gap)));

        // 3. Label Text
        row = row.child(div().flex_1().truncate().child(node.label.clone()));

        row.into_any_element()
    }
}

fn render_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(lucide_glyph(icon))
        .into_any_element()
}
```

---

## 5. Theme and Styling Resolvers (`theme.rs`)

Splits style values into visual colors (`TreeViewPalette`) and indentation scales (`TreeViewScale`), matching the standard Phase-5 density design pattern.

```rust
use std::sync::{Arc, OnceLock};
use gpui::{Hsla, SharedString};

use crate::theme::adorner::AdornerSpec;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};
use crate::theme::layout::snap_to_pixel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeViewScale {
    pub row_height: f32,
    pub base_padding_x: f32,
    pub indentation_width: f32,
    pub inner_gap: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
}

impl TreeViewScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let base_height = metrics.control_height(size);
        
        Self {
            row_height: snap_to_pixel(base_height * 0.85, scale_factor), // TreeView items are usually more compact
            base_padding_x: snap_to_pixel(metrics.padding_x(size), scale_factor),
            indentation_width: snap_to_pixel(16.0, scale_factor), // Shift left 16px per depth level
            inner_gap: snap_to_pixel(metrics.gap(size), scale_factor),
            radius: metrics.radius(size),
            icon_size: 16.0,
            chevron_size: 14.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeViewPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub chevron_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

pub trait TreeViewTheme: Send + Sync {
    fn resolve_row(&self, state: InteractionState, selected: bool) -> TreeViewPalette;
    fn metrics(&self) -> &MetricTokens;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTreeViewTheme {
    tokens: ThemeTokens,
}

pub fn default_tree_view_theme() -> Arc<dyn TreeViewTheme> {
    static THEME: OnceLock<Arc<dyn TreeViewTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(DefaultTreeViewTheme::default())).clone()
}

impl DefaultTreeViewTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TreeViewTheme for DefaultTreeViewTheme {
    fn resolve_row(&self, state: InteractionState, selected: bool) -> TreeViewPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let background = match (selected, layer) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(palette.action.prominent.pressed_background),
            (true, InteractionLayer::Hovered) => Some(palette.action.prominent.hover_background),
            (true, InteractionLayer::Default) => Some(palette.navigation.selected_background),
            (false, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (false, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (false, InteractionLayer::Default) => None,
        };

        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else if selected {
            palette.navigation.selected_foreground
        } else {
            palette.app.foreground
        };

        TreeViewPalette {
            background,
            foreground,
            icon_color: if selected { palette.navigation.selected_foreground } else { palette.navigation.muted_foreground },
            chevron_color: palette.navigation.muted_foreground,
            adorner: None,
            typography: typography.text.label,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }
}
```

---

## Shorthand Macro DSL Syntax

To integrate with declarative nodes (as proposed in `docs/ai/nav_tree.md`), we can extend tree-building with a simple shorthand `tree_node!` macro pattern:

```rust
macro_rules! tree_nodes {
    ( $( $node:tt ),* $(,)? ) => {
        vec![ $( tree_node!( $node ) ),* ]
    };
}

macro_rules! tree_node {
    // Branch node shorthand with children and custom metadata data payload
    ( ( $id:expr, $label:expr, $data:expr, expanded: $expanded:expr, [ $( $child:tt ),* $(,)? ] ) ) => {
        TreeNode::new($id, $label, $data)
            .expanded($expanded)
            .children(vec![ $( tree_node!( $child ) ),* ])
    };

    // Leaf node shorthand with custom data payload
    ( ( $id:expr, $label:expr, $data:expr ) ) => {
        TreeNode::new($id, $label, $data)
    };
}
```

---

## Gallery App Integration

To showcase and validate the new virtualized TreeView control under heavy load, a sample pane will be integrated into the `gpui-luma-gallery` application:

1. **Create the Gallery Pane (`apps/gallery/src/gallery/panes/tree_view.rs`)**:
   Implement a `struct TreeViewPane` that spawns a virtualized directory structure (mocking a file tree with 1,000+ nested items) to verify smooth rendering and layout caching.

2. **Register the Page (`apps/gallery/src/gallery/panes/registry.rs`)**:
   * Add `TreeView` to the `GalleryPageKind` enum.
   * Define the page constant:
     ```rust
     const TREE_VIEW_PAGE: GalleryPage = GalleryPage {
         id: "tree-view",
         label: "Tree View",
         icon: Some(LucideIcon::FolderTree),
         kind: GalleryPageKind::TreeView,
     };
     ```
   * Add the page to the selection page group (e.g., `SELECTION_PAGES`):
     ```rust
     const SELECTION_PAGES: &[GalleryPage] = &[
         // ... existing pages
         TREE_VIEW_PAGE,
     ];
     ```
   * Add the `tree_view` pane field to `GalleryPanes` struct, construct it in `GalleryPanes::new()`, and integrate it into `subscribe()`, `notify_controls()`, and `render_selected()`.
