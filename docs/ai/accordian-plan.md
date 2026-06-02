# Accordion Control Specification

This document outlines the architecture, file structure, API design, and layout strategies for introducing a new Accordion control into the `gpui-luma` SDK.

---

## Architecture & File Structure

Following the standard GPUI-Luma control module pattern, the Accordion control is organized under `crates/sdk/src/controls/accordion/` with a consistent module split:

```
crates/sdk/src/controls/accordion/
├── mod.rs        # Public API exports and convenience constructors
├── model.rs      # Configuration models, item models, and the builder
├── control.rs    # View entity, focus, state tracking, and keyboard events
├── template.rs   # Layout rendering logic, trigger/content composition
└── theme.rs      # Theme trait, palette resolution, and density scale computation
```

---

## 1. Public API Interface (`mod.rs`)

Provides clean public types, event schemas, and constructor entry points.

```rust
mod control;
mod model;
mod template;
mod theme;

pub use control::{AccordionControl, AccordionEvent};
pub use model::{
    AccordionBuilder, AccordionItem, AccordionTrigger, AccordionContent,
    AccordionModel, AccordionRenderModel, AccordionItemRenderModel,
    AccordionSelectionMode,
};
pub use template::{
    AccordionTemplate, AccordionTemplateHandlers, ThemedAccordionTemplate,
    default_accordion_template,
};
pub use theme::{
    AccordionTheme, AccordionPalette, AccordionScale, DefaultAccordionTheme,
    default_accordion_theme,
};

pub use crate::controls::state::{CompositeItemState as AccordionItemState, ControlFocusState};
use gpui::{Entity, SharedString};

/// Entity handle for an accordion control. Use directly — do not wrap in [`Entity`] again.
pub type Accordion = Entity<AccordionControl>;

/// Creates a new builder for an Accordion.
pub fn new(id: impl Into<SharedString>) -> AccordionBuilder {
    AccordionBuilder::new(id)
}
```

---

## 2. Configuration & Builder (`model.rs`)

Encapsulates static properties, items hierarchy, and builder methods. Leverages dynamic rendering slots for the accordion trigger and content areas.

```rust
use std::sync::Arc;
use gpui::{AnyElement, AppContext, Entity, IntoElement, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{Accordion, AccordionControl, AccordionTemplate, default_accordion_template};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AccordionSelectionMode {
    /// Only one item can be expanded at a time. Expanding another collapses the active one.
    #[default]
    Single,
    /// Multiple items can be expanded simultaneously.
    Multiple,
}

/// Represents the header/trigger area of an accordion item.
#[derive(Clone)]
pub struct AccordionTrigger {
    pub(crate) label: Option<SharedString>,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) custom_element: Option<Arc<dyn Fn() -> AnyElement + Send + Sync>>,
}

impl AccordionTrigger {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: Some(label.into()),
            icon: None,
            custom_element: None,
        }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn custom(custom: impl Fn() -> AnyElement + Send + Sync + 'static) -> Self {
        Self {
            label: None,
            icon: None,
            custom_element: Some(Arc::new(custom)),
        }
    }
}

/// Represents the collapsible panel content.
#[derive(Clone)]
pub struct AccordionContent {
    pub(crate) element: Option<Arc<dyn Fn() -> AnyElement + Send + Sync>>,
}

impl AccordionContent {
    pub fn new(element: impl IntoElement + Clone + 'static) -> Self {
        Self {
            element: Some(Arc::new(move || element.clone().into_any_element())),
        }
    }

    pub fn custom(custom: impl Fn() -> AnyElement + Send + Sync + 'static) -> Self {
        Self {
            element: Some(Arc::new(custom)),
        }
    }
}

/// Represents an item configuration in the accordion.
#[derive(Clone)]
pub struct AccordionItem {
    pub(crate) id: SharedString,
    pub(crate) trigger: AccordionTrigger,
    pub(crate) content: AccordionContent,
    pub(crate) enabled: bool,
    pub(crate) initially_expanded: bool,
}

impl AccordionItem {
    pub fn new(id: impl Into<SharedString>, trigger: AccordionTrigger, content: AccordionContent) -> Self {
        Self {
            id: id.into(),
            trigger,
            content,
            enabled: true,
            initially_expanded: false,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }
}

#[derive(Clone)]
pub struct AccordionModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<AccordionItem>,
    pub(crate) selection_mode: AccordionSelectionMode,
    pub(crate) collapsible: bool,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn AccordionTemplate>,
}

pub struct AccordionItemRenderModel<'a> {
    pub id: &'a SharedString,
    pub trigger: &'a AccordionTrigger,
    pub content: &'a AccordionContent,
    pub expanded: bool,
    pub enabled: bool,
    pub state: crate::controls::state::CompositeItemState,
}

pub struct AccordionRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<AccordionItemRenderModel<'a>>,
    pub selection_mode: AccordionSelectionMode,
    pub collapsible: bool,
    pub enabled: bool,
    pub focus: crate::controls::state::ControlFocusState,
}

pub struct AccordionBuilder {
    pub(crate) model: AccordionModel,
}

impl AccordionBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: AccordionModel {
                id: id.into(),
                items: Vec::new(),
                selection_mode: AccordionSelectionMode::Single,
                collapsible: true,
                enabled: true,
                template: default_accordion_template(),
            },
        }
    }

    pub fn mode(mut self, mode: AccordionSelectionMode) -> Self {
        self.model.selection_mode = mode;
        self
    }

    pub fn single(self) -> Self {
        self.mode(AccordionSelectionMode::Single)
    }

    pub fn multiple(self) -> Self {
        self.mode(AccordionSelectionMode::Multiple)
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.model.collapsible = collapsible;
        self
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn AccordionTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Accordion {
        cx.new(|cx| AccordionControl::from_builder(self, cx))
    }
}
```

---

## 3. View Logic & Event Handling (`control.rs`)

Tracks interactive element states and maps accessibility keyboard navigation profile actions to trigger headers. Skip disabled items when processing arrow traversal events.

```rust
use std::collections::HashSet;
use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, FocusHandle,
    IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Window, div, prelude::*,
};

use super::{
    AccordionBuilder, AccordionEvent, AccordionItemRenderModel, AccordionModel,
    AccordionRenderModel, AccordionTemplateHandlers,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem,
    SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum AccordionEvent {
    ExpandedChanged { item_id: SharedString, expanded: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccordionDirection {
    Previous,
    Next,
}

pub struct AccordionControl {
    model: AccordionModel,
    focus_handle: FocusHandle,
    expanded_ids: HashSet<SharedString>,
    focused_item_index: Option<usize>,
    hovered_item_index: Option<usize>,
    pressed_item_index: Option<usize>,
}

impl EventEmitter<AccordionEvent> for AccordionControl {}

impl AccordionControl {
    pub(crate) fn from_builder(builder: AccordionBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let mut expanded_ids = HashSet::new();

        for item in &builder.model.items {
            if item.initially_expanded {
                expanded_ids.insert(item.id.clone());
                if builder.model.selection_mode == super::AccordionSelectionMode::Single {
                    break;
                }
            }
        }

        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            expanded_ids,
            focused_item_index: None,
            hovered_item_index: None,
            pressed_item_index: None,
        }
    }

    pub fn toggle_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.model.enabled || index >= self.model.items.len() {
            return;
        }

        let item = &self.model.items[index];
        if !item.enabled {
            return;
        }

        let id = item.id.clone();
        let was_expanded = self.expanded_ids.contains(&id);

        match self.model.selection_mode {
            super::AccordionSelectionMode::Single => {
                if was_expanded {
                    if self.model.collapsible {
                        self.expanded_ids.remove(&id);
                        cx.emit(AccordionEvent::ExpandedChanged { item_id: id, expanded: false });
                    }
                } else {
                    for prev_id in self.expanded_ids.drain() {
                        cx.emit(AccordionEvent::ExpandedChanged { item_id: prev_id, expanded: false });
                    }
                    self.expanded_ids.insert(id.clone());
                    cx.emit(AccordionEvent::ExpandedChanged { item_id: id, expanded: true });
                }
            }
            super::AccordionSelectionMode::Multiple => {
                if was_expanded {
                    self.expanded_ids.remove(&id);
                    cx.emit(AccordionEvent::ExpandedChanged { item_id: id, expanded: false });
                } else {
                    self.expanded_ids.insert(id.clone());
                    cx.emit(AccordionEvent::ExpandedChanged { item_id: id, expanded: true });
                }
            }
        }

        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> AccordionRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        
        let items = self.model.items.iter().enumerate().map(|(index, item)| {
            let item_enabled = self.model.enabled && item.enabled;
            let expanded = self.expanded_ids.contains(&item.id);
            let has_keyboard_focus = focus.focused && self.focused_item_index == Some(index);

            AccordionItemRenderModel {
                id: &item.id,
                trigger: &item.trigger,
                content: &item.content,
                expanded,
                enabled: item_enabled,
                state: CompositeItemState {
                    hovered: item_enabled && self.hovered_item_index == Some(index),
                    pressed: item_enabled && self.pressed_item_index == Some(index),
                    disabled: !item_enabled,
                    selected: expanded,
                    active: item_enabled && has_keyboard_focus,
                    focus_visible: item_enabled && focus.focus_visible && has_keyboard_focus,
                },
            }
        }).collect();

        AccordionRenderModel {
            id: &self.model.id,
            items,
            selection_mode: self.model.selection_mode,
            collapsible: self.model.collapsible,
            enabled: self.model.enabled,
            focus,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> AccordionTemplateHandlers {
        AccordionTemplateHandlers {
            trigger_hovers: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, hovered, _, cx| {
                        if this.model.enabled && this.model.items[idx].enabled {
                            this.hovered_item_index = hovered.then_some(idx);
                            cx.notify();
                        }
                    })) as _
                })
                .collect(),
            trigger_mouse_downs: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, window, cx| {
                        if this.model.enabled && this.model.items[idx].enabled {
                            this.pressed_item_index = Some(idx);
                            this.focused_item_index = Some(idx);
                            this.focus_handle.focus(window, cx);
                            cx.notify();
                        }
                    })) as _
                })
                .collect(),
            trigger_mouse_ups: (0..self.model.items.len())
                .map(|_| {
                    Box::new(cx.listener(move |this, _, _, cx| {
                        this.pressed_item_index = None;
                        cx.notify();
                    })) as _
                })
                .collect(),
            trigger_clicks: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, _, cx| {
                        this.toggle_item(idx, cx);
                    })) as _
                })
                .collect(),
        }
    }

    // Helper: finds the next enabled trigger index (skips disabled triggers)
    fn next_enabled_index(&self, current: Option<usize>, direction: AccordionDirection) -> Option<usize> {
        let len = self.model.items.len();
        if len == 0 { return None; }
        
        let step = match direction {
            AccordionDirection::Previous => len - 1,
            AccordionDirection::Next => 1,
        };
        
        let mut index = match current {
            Some(idx) => (idx + step) % len,
            None if direction == AccordionDirection::Previous => len - 1,
            None => 0,
        };

        for _ in 0..len {
            if self.model.items[index].enabled {
                return Some(index);
            }
            index = (index + step) % len;
        }
        None
    }

    fn handle_previous(&mut self, _: &SelectPreviousItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled { return; }
        if let Some(next) = self.next_enabled_index(self.focused_item_index, AccordionDirection::Previous) {
            self.focused_item_index = Some(next);
            cx.notify();
        }
    }

    fn handle_next(&mut self, _: &SelectNextItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled { return; }
        if let Some(next) = self.next_enabled_index(self.focused_item_index, AccordionDirection::Next) {
            self.focused_item_index = Some(next);
            cx.notify();
        }
    }

    fn handle_first(&mut self, _: &SelectFirstItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled { return; }
        if let Some(idx) = self.model.items.iter().position(|item| item.enabled) {
            self.focused_item_index = Some(idx);
            cx.notify();
        }
    }

    fn handle_last(&mut self, _: &SelectLastItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled { return; }
        if let Some(idx) = self.model.items.iter().rposition(|item| item.enabled) {
            self.focused_item_index = Some(idx);
            cx.notify();
        }
    }

    fn handle_activate(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.focused_item_index {
            self.toggle_item(idx, cx);
        }
    }
}

impl Focusable for AccordionControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AccordionControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(&self.focus_handle)
                    .key_context(ControlKeyProfile::TabList.context())
                    .on_action(cx.listener(Self::handle_previous))
                    .on_action(cx.listener(Self::handle_next))
                    .on_action(cx.listener(Self::handle_first))
                    .on_action(cx.listener(Self::handle_last))
                    .on_action(cx.listener(Self::handle_activate)),
            )
            .into_any_element()
    }
}
```

---

## 4. Templates & Layout Rendering (`template.rs`)

Declares how the Accordion components are rendered, using an extensible template trait. Resolves trigger style palettes via `.state.interaction_state()`.

**Do not use `define_control_template!`.** `AccordionRenderModel<'a>` borrows from the control view; use a manual `ThemedAccordionTemplate` with `AccordionRenderModel<'_>` and an optional modifier pipeline (see implemented `template.rs`).

```rust
use std::sync::{Arc, OnceLock};
use gpui::{
    AnyElement, App, ClickEvent, Div, MouseDownEvent, MouseUpEvent,
    Stateful, Window, div, px, prelude::*,
};
use lucide_icons::Icon as LucideIcon;

use super::{AccordionRenderModel, AccordionItemRenderModel, AccordionTheme, default_accordion_theme};
use crate::controls::template::TemplateWithModifiers;
use crate::theme::{ControlSize, InteractionState, LayoutCacheKey, LumaLayoutCacheExt};
use crate::define_control_template;

pub type AccordionHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type AccordionMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type AccordionMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type AccordionClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Default)]
pub struct AccordionTemplateHandlers {
    pub trigger_hovers: Vec<AccordionHoverHandler>,
    pub trigger_mouse_downs: Vec<AccordionMouseDownHandler>,
    pub trigger_mouse_ups: Vec<AccordionMouseUpHandler>,
    pub trigger_clicks: Vec<AccordionClickHandler>,
}

pub trait AccordionTemplate: Send + Sync {
    fn render(
        &self,
        model: &AccordionRenderModel,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

// REMOVED: define_control_template! — incompatible with AccordionRenderModel<'a> lifetimes.

impl AccordionTemplate for ThemedAccordionTemplate {
    fn render(
        &self,
        model: &AccordionRenderModel,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| super::theme::AccordionScale::compute(ControlSize::Md, metrics, scale_factor),
        );

        let AccordionTemplateHandlers {
            mut trigger_hovers,
            mut trigger_mouse_downs,
            mut trigger_mouse_ups,
            mut trigger_clicks,
        } = handlers;

        let mut trigger_hovers = trigger_hovers.drain(..);
        let mut trigger_mouse_downs = trigger_mouse_downs.drain(..);
        let mut trigger_mouse_ups = trigger_mouse_ups.drain(..);
        let mut trigger_clicks = trigger_clicks.drain(..);

        let mut root = div()
            .id(model.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .gap(px(scale.item_gap));

        for item in &model.items {
            let hover_handler = trigger_hovers.next();
            let down_handler = trigger_mouse_downs.next();
            let up_handler = trigger_mouse_ups.next();
            let click_handler = trigger_clicks.next();

            // Resolve themed appearance palettes for current item states
            let trigger_palette = self.theme.resolve_trigger(item.state.interaction_state());
            let content_palette = self.theme.resolve_content(item.expanded);

            // Render AccordionTrigger
            let mut trigger = div()
                .id(format!("{}-trigger", item.id))
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .min_h(px(scale.trigger_height))
                .px(px(scale.padding_x))
                .py(px(scale.padding_y))
                .rounded(px(scale.radius))
                .bg(trigger_palette.background)
                .text_color(trigger_palette.foreground)
                .text_size(px(trigger_palette.typography.size))
                .line_height(px(trigger_palette.typography.line_height))
                .font_family(trigger_palette.font_family.clone())
                .font_weight(trigger_palette.typography.weight)
                .cursor_pointer();

            // Bind triggers handlers
            if item.enabled {
                if let Some(h) = hover_handler { trigger = trigger.on_hover(h); }
                if let Some(d) = down_handler { trigger = trigger.on_mouse_down(gpui::MouseButton::Left, d); }
                if let Some(u) = up_handler { trigger = trigger.on_mouse_up(gpui::MouseButton::Left, u); }
                if let Some(c) = click_handler { trigger = trigger.on_click(c); }
            } else {
                trigger = trigger.opacity(0.56);
            }

            let trigger_content = match &item.trigger.custom_element {
                Some(renderer) => renderer(),
                None => {
                    let mut row = div().flex().items_center().gap(px(scale.inner_gap));
                    if let Some(icon) = item.trigger.icon {
                        row = row.child(render_icon(icon, trigger_palette.icon_color, scale.icon_size));
                    }
                    row = row.child(item.trigger.label.clone().unwrap_or_default());
                    row.into_any_element()
                }
            };

            // Disclosure Indicator Chevron
            let chevron_icon = if item.expanded { LucideIcon::ChevronDown } else { LucideIcon::ChevronRight };
            let chevron = render_icon(chevron_icon, trigger_palette.chevron_color, scale.chevron_size);

            let trigger_el = trigger.child(trigger_content).child(chevron);

            // Render AccordionContent (collapsible panel)
            let content_el = if item.expanded {
                let mut content = div()
                    .id(format!("{}-content", item.id))
                    .w_full()
                    .px(px(scale.padding_x))
                    .py(px(scale.content_padding_y))
                    .bg(content_palette.background)
                    .text_color(content_palette.foreground);

                if let Some(renderer) = &item.content.element {
                    content = content.child(renderer());
                }

                Some(content)
            } else {
                None
            };

            // Combined Item Wrapper
            let mut item_container = div()
                .id(format!("{}-item", item.id))
                .flex()
                .flex_col()
                .w_full()
                .border_b_1()
                .border_color(trigger_palette.border_color)
                .child(trigger_el);

            if let Some(cel) = content_el {
                item_container = item_container.child(cel);
            }

            root = root.child(item_container);
        }

        self.apply_modifiers(root, model)
    }
}

fn render_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
```

---

## 5. Theme and Styling Resolvers (`theme.rs`)

Follows the **Phase-5 density refactoring** pattern. Palette colors are resolved via the theme trait depending on the control's interaction states, while scaling metrics are computed dynamically in `AccordionScale` using `MetricTokens` and the window's scale factor.

```rust
use std::sync::{Arc, OnceLock};
use gpui::{Hsla, SharedString};

use crate::theme::adorner::AdornerSpec;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};
use crate::theme::layout::snap_to_pixel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccordionScale {
    pub trigger_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub content_padding_y: f32,
    pub radius: f32,
    pub item_gap: f32,
    pub inner_gap: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
}

impl AccordionScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let base_height = metrics.control_height(size);
        
        Self {
            trigger_height: snap_to_pixel(base_height, scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(size), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(size), scale_factor),
            content_padding_y: snap_to_pixel(metrics.padding_y(size) * 1.5, scale_factor),
            radius: metrics.radius(size),
            item_gap: 0.0, // Accordion items usually stack flush with border dividers
            inner_gap: snap_to_pixel(metrics.gap(size), scale_factor),
            icon_size: 16.0,
            chevron_size: 14.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccordionPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub border_color: Hsla,
    pub icon_color: Hsla,
    pub chevron_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct AccordionContentPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
}

pub trait AccordionTheme: Send + Sync {
    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette;
    fn resolve_content(&self, expanded: bool) -> AccordionContentPalette;
    fn metrics(&self) -> &MetricTokens;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultAccordionTheme {
    tokens: ThemeTokens,
}

pub fn default_accordion_theme() -> Arc<dyn AccordionTheme> {
    static THEME: OnceLock<Arc<dyn AccordionTheme>> = OnceLock::new();
    THEME.get_or_init(|| Arc::new(DefaultAccordionTheme::default())).clone()
}

impl DefaultAccordionTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl AccordionTheme for DefaultAccordionTheme {
    fn resolve_trigger(&self, state: InteractionState) -> AccordionPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let background = match layer {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        AccordionPalette {
            background,
            foreground,
            border_color: palette.navigation.border,
            icon_color: foreground,
            chevron_color: palette.navigation.muted_foreground,
            adorner: None,
            typography: typography.text.label,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn resolve_content(&self, _expanded: bool) -> AccordionContentPalette {
        let palette = &self.tokens.palette;
        
        AccordionContentPalette {
            background: None,
            foreground: palette.app.foreground,
        }
    }

    fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }
}
```

---

## Declarative Usage Example

Below is a declarative usage example demonstrating how a consumer registers and mounts the new Accordion control using the specified builder interface:

```rust
use gpui::{Context, Entity, Render, Window, div, prelude::*};
use gpui_luma::controls::accordion::{self, Accordion, AccordionItem, AccordionTrigger, AccordionContent};
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

pub struct SettingsPanel {
    accordion: Accordion,
}

impl SettingsPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let accordion = accordion::new("settings-accordion")
            .single() // Allow only one item open at a time
            .collapsible(true) // Allow closing the active item
            .item(
                AccordionItem::new(
                    "profile",
                    AccordionTrigger::new("User Profile").icon(LucideIcon::User),
                    AccordionContent::new(
                        div()
                            .p_4()
                            .child("Manage your profile pictures and bio details here.")
                    )
                ).expanded(true) // Initially expanded
            )
            .item(
                AccordionItem::new(
                    "notifications",
                    AccordionTrigger::new("Notifications").icon(LucideIcon::Bell),
                    AccordionContent::new(
                        div()
                            .p_4()
                            .child("Enable/Disable push notifications and alerts.")
                    )
                )
            )
            .spawn(cx);

        // Subscribe to changes if needed
        cx.subscribe(&accordion, |_, _, event, cx| {
            match event {
                accordion::AccordionEvent::ExpandedChanged { item_id, expanded } => {
                    println!("Item '{}' expanded status: {}", item_id, expanded);
                }
            }
        }).detach();

        Self { accordion }
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_96()
            .border_1()
            .border_color(gpui::red()) // Outline border
            .child(self.accordion.clone())
    }
}
```

---

## Comparison: Early vs. Phase-5 Control Architecture

| Design Aspect | Early Controls (e.g. `NavigationSidebar`) | New Control Implementation (Accordion) |
| :--- | :--- | :--- |
| **Sizing Calculation** | Heights, paddings, and offsets are defined directly inside visual theme struct fields (e.g. `NavigationSidebarItemAppearance`). | Separated into a distinct `AccordionScale` computed dynamically from `MetricTokens` and snapped/cached via `cx.use_cached_layout(...)`. |
| **Color Resolution** | Unified style resolution struct. | Resolves specific `AccordionPalette` and `AccordionContentPalette` structs for separate sub-elements (Trigger vs. Content). |
| **Hierarchy** | Uses a deep, nested, recursive tree structure (`NavNode` containing a list of nested child nodes) with manual alignment rules. | Uses a clean, flat list of semantic `AccordionItem` instances, simplifying layouts and traversal. |
| **Hosted Elements** | Presenter-based content hosting. | Element closures (`Arc<dyn Fn() -> AnyElement>`) allowing complete child design freedom. |

---

## 6. Gallery App Integration

To showcase and validate the new control, a sample pane will be integrated into the `gpui-luma-gallery` application:

1. **Create the Gallery Pane (`apps/gallery/src/gallery/panes/accordion.rs`)**:
   Implement a `struct AccordionPane` that instantiates both a Single-expansion Accordion and a Multiple-expansion Accordion to demonstrate all builder configuration types.

2. **Register the Page (`apps/gallery/src/gallery/panes/registry.rs`)**:
   * Add `Accordion` to the `GalleryPageKind` enum.
   * Define the page constant:
     ```rust
     const ACCORDION_PAGE: GalleryPage = GalleryPage {
         id: "accordion",
         label: "Accordion",
         icon: None,
         kind: GalleryPageKind::Accordion,
     };
     ```
   * Add the page to the choice page group (e.g., `CHOICE_PAGES`):
     ```rust
     const CHOICE_PAGES: &[GalleryPage] = &[
         // ... existing pages
         ACCORDION_PAGE,
     ];
     ```
   * Include the new pane field inside the `GalleryPanes` struct, instantiate it in `GalleryPanes::new()`, and hook it up in the `subscribe()`, `notify_controls()`, and `render_selected()` functions.

---

## 7. Implementation Notes (Quality Review)

| Concern | Resolution |
| :--- | :--- |
| `define_control_template!` + `'static` lifetime | Manual `ThemedAccordionTemplate` + `AccordionTemplateModifier` pipeline |
| `Entity<Accordion>` double-wrap | `Accordion` is already `Entity<AccordionControl>` — use `accordion: Accordion`, not `Entity<Accordion>` |
| Index safety in `toggle_item` | Bounds check before access; `apply_toggle` uses `drain()` then `insert()` |
| Lucide icon font weight | `render_icon` sets `FontWeight::NORMAL` |
| Radix theme integration | `RadixTheme::accordion(id)` + `theme/radix/accordion.rs`; gallery uses `radix_theme.accordion(...)` |
