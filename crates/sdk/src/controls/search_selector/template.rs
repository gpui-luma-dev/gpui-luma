use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, ScrollWheelEvent,
    SharedString, Stateful, Window, anchored, deferred, div, point, prelude::*, px,
};

use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::controls::floating_menu::{FloatingMenuAppearance, FloatingMenuClickHandler, FloatingMenuHoverHandler};
use crate::controls::menu_item::MenuItem;
use crate::controls::textfield::{TextFieldState, default_textfield_theme};

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

pub struct SearchSelectorRenderModel {
    pub id: SharedString,
    pub trigger_label: SharedString,
    pub trigger_label_is_placeholder: bool,
    pub trigger_state: TextFieldState,
    pub full_width: bool,
    pub minimum_trigger_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_appearance: FloatingMenuAppearance,
    pub popup_content: Option<AnyElement>,
    pub popup_search_field: Option<AnyElement>,
    pub popup_list_panel: Option<AnyElement>,
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
        _window: &mut Window,
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

        let trigger_appearance = default_textfield_theme().resolve(model.trigger_state, true);
        let structured_popup = model.popup_search_field.is_some() || model.popup_list_panel.is_some();

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
                                                .child(
                                                    div()
                                                        .font_family("lucide")
                                                        .text_size(px(12.0))
                                                        .line_height(px(12.0))
                                                        .child(char::from(lucide_icons::Icon::Search).to_string()),
                                                ),
                                        ),
                                ),
                            trigger_appearance.focus_ring,
                            trigger_appearance.radius,
                        )
                        .w_full(),
                    ),
            )
            .when_some(model.popup_content, |root, popup_content| {
                root.when_some(model.popup_bounds, |root, bounds| {
                    root.child(
                        deferred(
                            anchored()
                                .snap_to_window_with_margin(px(8.0))
                                .anchor(gpui::Corner::TopLeft)
                                .position(point(bounds.left(), bounds.bottom()))
                                .offset(point(px(0.0), px(4.0)))
                                .child(
                                    div()
                                        .id(format!("{}-popup-shell", model.id))
                                        .w(bounds.size.width)
                                        .bg(model.popup_appearance.background)
                                        .border_1()
                                        .border_color(model.popup_appearance.border)
                                        .rounded(px(model.popup_appearance.radius))
                                        .shadow(model.popup_appearance.shadow.clone())
                                        .overflow_hidden()
                                        .when_some(model.popup_search_field, |shell, search| {
                                            shell
                                                .child(div().p(px(8.0)).child(search))
                                                .child(div().h(px(1.0)).bg(model.popup_appearance.border))
                                        })
                                        .when_some(model.popup_list_panel, |shell, list| shell.child(list))
                                        .when(!structured_popup, |shell| shell.child(popup_content)),
                                ),
                        )
                        .with_priority(1),
                    )
                })
            })
    }
}

pub fn render_popup_rows(
    id: &SharedString,
    items: &[MenuItem],
    appearance: FloatingMenuAppearance,
    highlighted_index: Option<usize>,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> gpui::Stateful<gpui::Div> {
    let mut root = div().id(format!("{}-rows", id)).flex().flex_col().p(px(appearance.padding));
    let mut clicks = item_clicks.into_iter();

    for (index, (item, hover)) in items.iter().zip(item_hovers).enumerate() {
        let mut row = div()
            .id(format!("{}-row-{}", id, index))
            .flex()
            .items_center()
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(appearance.foreground)
            .text_size(px(appearance.item_typography.size))
            .line_height(px(appearance.item_typography.line_height))
            .font_weight(appearance.item_typography.weight)
            .child(item.label_text().clone());

        row = row.cursor_pointer().on_hover(hover).hover({
            let hover_background = appearance.item_hover_background;
            move |style| style.bg(hover_background)
        });

        if highlighted_index.is_some_and(|active| active == index) {
            row = row.bg(appearance.item_hover_background);
        }

        if let Some(click) = clicks.next() {
            row = row.on_click(click);
        }

        root = root.child(row);
    }

    root
}
