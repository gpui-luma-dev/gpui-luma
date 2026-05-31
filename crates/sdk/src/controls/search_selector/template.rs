use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, ScrollWheelEvent,
    SharedString, Stateful, Window, div, prelude::*, px,
};

use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::controls::icon::lucide_icon;
use crate::controls::selector_panel::{SelectorItemsPanelAppearance, SelectorPanelClickHandler, SelectorPanelHoverHandler};

use super::behavior::SelectionItem;
use super::item_template::{SearchSelectorItemRenderModel, SearchSelectorItemTemplate};
use crate::controls::textfield::{TextFieldState, TextFieldTheme, TextFieldVariant};
use crate::theme::{ControlSize, StandardBoxScale};

pub type SearchSelectorKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorScrollWheelHandler = Box<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerMouseUpHandler = Box<dyn Fn(&gpui::MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SearchSelectorTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;

pub struct SearchSelectorTemplateHandlers {
    pub key_down: SearchSelectorKeyDownHandler,
    pub scroll_wheel: SearchSelectorScrollWheelHandler,
    pub trigger_click: SearchSelectorTriggerClickHandler,
    pub trigger_hover: SearchSelectorTriggerHoverHandler,
    pub trigger_mouse_down: SearchSelectorTriggerMouseDownHandler,
    pub trigger_mouse_up: SearchSelectorTriggerMouseUpHandler,
    pub trigger_mouse_up_out: SearchSelectorTriggerMouseUpHandler,
    pub trigger_bounds: SearchSelectorTriggerBoundsHandler,
}

fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_scroll_wheel(_: &ScrollWheelEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &gpui::MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

impl Default for SearchSelectorTemplateHandlers {
    fn default() -> Self {
        Self {
            key_down: Box::new(noop_key_down),
            scroll_wheel: Box::new(noop_scroll_wheel),
            trigger_click: Box::new(noop_click),
            trigger_hover: Box::new(noop_hover),
            trigger_mouse_down: Box::new(noop_mouse_down),
            trigger_mouse_up: Box::new(noop_mouse_up),
            trigger_mouse_up_out: Box::new(noop_mouse_up),
            trigger_bounds: Box::new(noop_bounds),
        }
    }
}

pub struct SearchSelectorRenderModel {
    pub id: SharedString,
    pub trigger_label: SharedString,
    pub trigger_label_is_placeholder: bool,
    pub trigger_state: TextFieldState,
    pub trigger_theme: Arc<dyn TextFieldTheme>,
    pub full_width: bool,
    pub minimum_trigger_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_content: Option<AnyElement>,
}

pub trait SearchSelectorTemplate: Send + Sync {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultSearchSelectorTemplate;

pub fn default_search_selector_template() -> Arc<dyn SearchSelectorTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SearchSelectorTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultSearchSelectorTemplate)).clone()
}

impl SearchSelectorTemplate for DefaultSearchSelectorTemplate {
    fn render(
        &self,
        model: SearchSelectorRenderModel,
        handlers: SearchSelectorTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let SearchSelectorTemplateHandlers {
            key_down,
            scroll_wheel,
            trigger_click,
            trigger_hover,
            trigger_mouse_down,
            trigger_mouse_up,
            trigger_mouse_up_out,
            trigger_bounds,
        } = handlers;

        let trigger_appearance = model.trigger_theme.resolve_appearance(
            TextFieldVariant::Standard,
            model.trigger_state,
            true,
            &StandardBoxScale::compute(ControlSize::Md, model.trigger_theme.metrics(), window.scale_factor()),
        );

        div()
            .id(format!("{}-root", model.id))
            .when(model.full_width, |root| root.w_full())
            .flex()
            .flex_col()
            .on_key_down(key_down)
            .on_scroll_wheel(scroll_wheel)
            .child(
                div()
                    .on_children_prepainted(move |bounds, window, cx| {
                        if let Some(bounds) = bounds.first() {
                            trigger_bounds(bounds, window, cx);
                        }
                    })
                    .min_w(model.minimum_trigger_width)
                    .when(model.full_width, |row| row.w_full())
                    .relative()
                    .child(
                        render_button_family_focus_ring(
                            format!("{}-trigger-ring", model.id).into(),
                            div()
                                .id(format!("{}-trigger", model.id))
                                .relative()
                                .w_full()
                                .h(px(trigger_appearance.min_height))
                                .px(px(trigger_appearance.padding_x))
                                .py(px(trigger_appearance.padding_y))
                                .border(px(trigger_appearance.border_width))
                                .border_color(trigger_appearance.border)
                                .rounded(px(trigger_appearance.radius))
                                .bg(trigger_appearance.background)
                                .text_size(px(trigger_appearance.typography.size))
                                .line_height(px(trigger_appearance.typography.line_height))
                                .font_weight(trigger_appearance.typography.weight)
                                .cursor_pointer()
                                .on_hover(trigger_hover)
                                .on_mouse_down(MouseButton::Left, trigger_mouse_down)
                                .on_mouse_up(MouseButton::Left, trigger_mouse_up)
                                .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
                                .on_click(trigger_click)
                                .child(
                                    div()
                                        .h_full()
                                        .w_full()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(trigger_appearance.gap))
                                        .child(
                                            div()
                                                .min_w(px(0.0))
                                                .truncate()
                                                .text_color(if model.trigger_label_is_placeholder {
                                                    trigger_appearance.placeholder
                                                } else {
                                                    trigger_appearance.foreground
                                                })
                                                .child(model.trigger_label),
                                        )
                                        .child(
                                            div()
                                                .id(format!("{}-icon", model.id))
                                                .w(px(18.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_color(trigger_appearance.icon)
                                                .child(lucide_icon(
                                                    lucide_icons::Icon::Search,
                                                    trigger_appearance.icon,
                                                    12.0,
                                                )),
                                        ),
                                ),
                            trigger_appearance.focus_ring,
                            trigger_appearance.radius,
                        )
                        .w_full(),
                    ),
            )
            .when_some(model.popup_content, |root, popup_content| root.child(popup_content))
    }
}

pub struct SearchSelectorItemsRenderModel<'a> {
    pub menu_id: &'a SharedString,
    pub search_selector_id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a SearchSelectorItemTemplate<SelectionItem>>,
    pub appearance: SelectorItemsPanelAppearance,
}

pub struct SearchSelectorItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub trait SearchSelectorItemsTemplate: Send + Sync {
    fn render(
        &self,
        model: &SearchSelectorItemsRenderModel<'_>,
        handlers: SearchSelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultSearchSelectorItemsTemplate;

pub fn default_search_selector_items_template() -> Arc<dyn SearchSelectorItemsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SearchSelectorItemsTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultSearchSelectorItemsTemplate)).clone()
}

impl SearchSelectorItemsTemplate for DefaultSearchSelectorItemsTemplate {
    fn render(
        &self,
        model: &SearchSelectorItemsRenderModel<'_>,
        handlers: SearchSelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let appearance = model.appearance.clone();

        let mut root = div()
            .id(format!("{}-rows", model.menu_id))
            .relative()
            .flex()
            .flex_col()
            .min_w(px(appearance.min_width))
            .p(px(appearance.padding));
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let selected = model.selected_source_index == Some(source_index);
            let active = model.active_visible_index == Some(visible_index);
            let content = if let Some(item_template) = model.item_template {
                item_template(
                    &SearchSelectorItemRenderModel {
                        search_selector_id: model.search_selector_id,
                        item,
                        source_index,
                        visible_index,
                        selected,
                        active,
                        open: model.open,
                        enabled: model.enabled,
                    },
                    cx,
                )
            } else {
                div().flex_1().child(item.label.clone()).into_any_element()
            };

            let mut row = div()
                .id(format!("{}-row-{}", model.menu_id, visible_index))
                .flex()
                .items_center()
                .min_h(px(appearance.item_height))
                .px(px(appearance.item_padding_x))
                .rounded(px(appearance.item_radius))
                .text_color(appearance.foreground)
                .text_size(px(appearance.item_typography.size))
                .line_height(px(appearance.item_typography.line_height))
                .font_weight(appearance.item_typography.weight)
                .child(content);

            row = row.cursor_pointer().on_hover(hover).hover({
                let hover_background = appearance.item_hover_background;
                move |style| style.bg(hover_background)
            });

            if active {
                row = row.bg(appearance.item_hover_background);
            }

            if let Some(click) = clicks.next() {
                row = row.on_click(click);
            }

            root = root.child(row);
        }

        root
    }
}

#[deprecated(note = "Use SearchSelectorItemsTemplate::render with SearchSelectorItemsRenderModel")]
#[allow(clippy::too_many_arguments)]
pub fn render_popup_rows(
    menu_id: &SharedString,
    search_selector_id: &SharedString,
    items: &[SelectionItem],
    visible_indices: &[usize],
    selected_source_index: Option<usize>,
    active_visible_index: Option<usize>,
    open: bool,
    enabled: bool,
    item_template: Option<&SearchSelectorItemTemplate<SelectionItem>>,
    appearance: SelectorItemsPanelAppearance,
    item_hovers: Vec<SelectorPanelHoverHandler>,
    item_clicks: Vec<SelectorPanelClickHandler>,
    cx: &mut App,
) -> Stateful<gpui::Div> {
    DefaultSearchSelectorItemsTemplate.render(
        &SearchSelectorItemsRenderModel {
            menu_id,
            search_selector_id,
            items,
            visible_indices,
            selected_source_index,
            active_visible_index,
            open,
            enabled,
            item_template,
            appearance,
        },
        SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks },
        cx,
    )
}
