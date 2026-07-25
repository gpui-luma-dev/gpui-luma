use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, Pixels, ScrollWheelEvent, SharedString, Stateful, Window,
    anchored, deferred, div, point, prelude::*, px,
};

use crate::controls::icon::lucide_icon;
use crate::controls::selector_panel::{
    SelectorItem, SelectorItemsPanelLook, SelectorPanelClickHandler, SelectorPanelHoverHandler,
};
use crate::theme::LumaTypography;

pub type AutocompleteTextBoxKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type AutocompleteTextBoxScrollWheelHandler = Box<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static>;
pub type AutocompleteTextBoxClearClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type AutocompleteTextBoxTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type AutocompleteTextBoxTemplateModifier =
    Box<dyn Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static>;

pub struct AutocompleteTextBoxTemplateHandlers {
    pub key_down: AutocompleteTextBoxKeyDownHandler,
    pub scroll_wheel: AutocompleteTextBoxScrollWheelHandler,
    pub clear_click: AutocompleteTextBoxClearClickHandler,
    pub trigger_bounds: AutocompleteTextBoxTriggerBoundsHandler,
}

fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_scroll_wheel(_: &ScrollWheelEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

impl Default for AutocompleteTextBoxTemplateHandlers {
    fn default() -> Self {
        Self {
            key_down: Box::new(noop_key_down),
            scroll_wheel: Box::new(noop_scroll_wheel),
            clear_click: Box::new(noop_click),
            trigger_bounds: Box::new(noop_bounds),
        }
    }
}

pub struct AutocompleteTextBoxRenderModel {
    pub id: SharedString,
    pub textfield: AnyElement,
    pub query_is_empty: bool,
    pub popup_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_look: SelectorItemsPanelLook,
    pub popup_content: Option<AnyElement>,
}

pub trait AutocompleteTextBoxTemplate: Send + Sync {
    fn render(
        &self,
        model: AutocompleteTextBoxRenderModel,
        handlers: AutocompleteTextBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultAutocompleteTextBoxTemplate {
    modifiers: Vec<AutocompleteTextBoxTemplateModifier>,
}

struct ModifiedAutocompleteTextBoxTemplate {
    base: Arc<dyn AutocompleteTextBoxTemplate>,
    modifiers: Vec<AutocompleteTextBoxTemplateModifier>,
}

impl DefaultAutocompleteTextBoxTemplate {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<gpui::Div>, cx: &mut App) -> Stateful<gpui::Div> {
        for modifier in &self.modifiers {
            root = modifier(root, cx);
        }
        root
    }
}

impl Default for DefaultAutocompleteTextBoxTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl ModifiedAutocompleteTextBoxTemplate {
    fn new(base: Arc<dyn AutocompleteTextBoxTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: AutocompleteTextBoxTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<gpui::Div>, cx: &mut App) -> Stateful<gpui::Div> {
        for modifier in &self.modifiers {
            root = modifier(root, cx);
        }
        root
    }
}

pub fn default_autocomplete_textbox_template() -> Arc<dyn AutocompleteTextBoxTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AutocompleteTextBoxTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultAutocompleteTextBoxTemplate::new())).clone()
}

pub(super) fn modified_autocomplete_textbox_template<F>(
    template: Arc<dyn AutocompleteTextBoxTemplate>,
    modifier: F,
) -> Arc<dyn AutocompleteTextBoxTemplate>
where
    F: Fn(Stateful<gpui::Div>, &mut App) -> Stateful<gpui::Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedAutocompleteTextBoxTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl AutocompleteTextBoxTemplate for ModifiedAutocompleteTextBoxTemplate {
    fn render(
        &self,
        model: AutocompleteTextBoxRenderModel,
        handlers: AutocompleteTextBoxTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, cx)
    }
}

impl AutocompleteTextBoxTemplate for DefaultAutocompleteTextBoxTemplate {
    fn render(
        &self,
        model: AutocompleteTextBoxRenderModel,
        handlers: AutocompleteTextBoxTemplateHandlers,
        _window: &mut Window,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let AutocompleteTextBoxTemplateHandlers { key_down, scroll_wheel, clear_click, trigger_bounds } = handlers;

        let has_status_text = !model.status_label.is_empty() || !model.status_detail.is_empty();
        let status_style = LumaTypography::default().text.scale.sm;

        let status_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(status_style.size))
                    .line_height(px(status_style.line_height))
                    .font_weight(status_style.weight)
                    .text_color(model.status_color)
                    .child(model.status_label),
            )
            .child(
                div()
                    .text_size(px(status_style.size))
                    .line_height(px(status_style.line_height))
                    .font_weight(status_style.weight)
                    .text_color(model.muted_text_color)
                    .child(model.status_detail),
            );

        let root = div()
            .id(format!("{}-root", model.id))
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .when(has_status_text, |root| root.child(status_row))
            .on_key_down(key_down)
            .on_scroll_wheel(scroll_wheel)
            .child(
                div()
                    .on_children_prepainted(move |bounds, window, cx| {
                        if let Some(bounds) = bounds.first() {
                            trigger_bounds(bounds, window, cx);
                        }
                    })
                    .w_full()
                    .relative()
                    .child(model.textfield)
                    .when(!model.query_is_empty, |row| {
                        row.child(
                            div()
                                .id("prototype-autocomplete-clear")
                                .absolute()
                                .top(px(0.0))
                                .right(px(10.0))
                                .h_full()
                                .w(px(18.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .text_color(model.muted_text_color)
                                .hover(|style| style.text_color(model.status_color))
                                .on_click(clear_click)
                                .child(lucide_icon(lucide_icons::Icon::X, model.muted_text_color, 12.0)),
                        )
                    }),
            )
            .when_some(model.popup_content, |root, popup_content| {
                root.when_some(model.popup_bounds, |root, bounds| {
                    root.child(
                        deferred(
                            anchored()
                                .snap_to_window_with_margin(px(8.0))
                                .anchor(gpui::Anchor::TopLeft)
                                .position(point(bounds.left(), bounds.bottom()))
                                .offset(point(px(0.0), px(4.0)))
                                .child(
                                    div()
                                        .id("prototype-autocomplete-popup-shell")
                                        .w(model.popup_width)
                                        .bg(model.popup_look.background)
                                        .border_1()
                                        .border_color(model.popup_look.border)
                                        .rounded(px(model.popup_look.radius))
                                        .shadow(model.popup_look.shadow.clone())
                                        .overflow_hidden()
                                        .occlude()
                                        .child(popup_content),
                                ),
                        )
                        .with_priority(1),
                    )
                })
            });

        self.apply_modifiers(root, cx)
    }
}

pub struct AutocompleteItemsRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [SelectorItem],
    pub look: SelectorItemsPanelLook,
    pub highlighted_index: Option<usize>,
}

pub struct AutocompleteItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub type AutocompleteItemsTemplateModifier = Box<
    dyn for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
        + Send
        + Sync
        + 'static,
>;

pub trait AutocompleteItemsTemplate: Send + Sync {
    fn render(
        &self,
        model: &AutocompleteItemsRenderModel<'_>,
        handlers: AutocompleteItemsTemplateHandlers,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultAutocompleteItemsTemplate {
    modifiers: Vec<AutocompleteItemsTemplateModifier>,
}

struct ModifiedAutocompleteItemsTemplate {
    base: Arc<dyn AutocompleteItemsTemplate>,
    modifiers: Vec<AutocompleteItemsTemplateModifier>,
}

impl DefaultAutocompleteItemsTemplate {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(
        &self,
        mut root: Stateful<gpui::Div>,
        model: &AutocompleteItemsRenderModel<'_>,
    ) -> Stateful<gpui::Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl Default for DefaultAutocompleteItemsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl ModifiedAutocompleteItemsTemplate {
    fn new(base: Arc<dyn AutocompleteItemsTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(
        &self,
        mut root: Stateful<gpui::Div>,
        model: &AutocompleteItemsRenderModel<'_>,
    ) -> Stateful<gpui::Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_autocomplete_items_template() -> Arc<dyn AutocompleteItemsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AutocompleteItemsTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultAutocompleteItemsTemplate::new())).clone()
}

impl AutocompleteItemsTemplate for DefaultAutocompleteItemsTemplate {
    fn render(
        &self,
        model: &AutocompleteItemsRenderModel<'_>,
        handlers: AutocompleteItemsTemplateHandlers,
    ) -> Stateful<gpui::Div> {
        let AutocompleteItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let look = model.look.clone();

        let mut root = div()
            .id(format!("{}-rows", model.id))
            .relative()
            .flex()
            .flex_col()
            .min_w(px(look.min_width))
            .p(px(look.padding));
        let mut clicks = item_clicks.into_iter();

        for (index, (item, hover)) in model.items.iter().zip(item_hovers).enumerate() {
            let enabled_item = item.is_enabled();
            let color = if enabled_item {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let mut row = div()
                .id(format!("{}-row-{}", model.id, index))
                .flex()
                .items_center()
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(div().flex_1().min_w(px(0.0)).truncate().child(item.label_text().clone()));

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = look.item_hover_background;
                    let hover_foreground = look.item_hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if model.highlighted_index.is_some_and(|active| active == index) {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if let Some(click) = clicks.next() {
                    row = row.on_click(click);
                }
            } else {
                row = row.opacity(0.56);
            }

            root = root.child(row);
        }

        self.apply_modifiers(root, model)
    }
}

impl AutocompleteItemsTemplate for ModifiedAutocompleteItemsTemplate {
    fn render(
        &self,
        model: &AutocompleteItemsRenderModel<'_>,
        handlers: AutocompleteItemsTemplateHandlers,
    ) -> Stateful<gpui::Div> {
        let root = self.base.render(model, handlers);
        self.apply_modifiers(root, model)
    }
}

pub(super) fn modified_autocomplete_items_template<F>(
    template: Arc<dyn AutocompleteItemsTemplate>,
    modifier: F,
) -> Arc<dyn AutocompleteItemsTemplate>
where
    F: for<'a> Fn(Stateful<gpui::Div>, &AutocompleteItemsRenderModel<'a>) -> Stateful<gpui::Div>
        + Send
        + Sync
        + 'static,
{
    Arc::new(ModifiedAutocompleteItemsTemplate::new(template).with_modifier(modifier))
}
