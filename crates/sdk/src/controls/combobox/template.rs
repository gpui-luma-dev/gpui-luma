use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, ScrollWheelEvent,
    SharedString, Window, anchored, deferred, div, point, prelude::*, px,
};

use crate::controls::icon::lucide_icon;
use crate::controls::selector_panel::SelectorItemsPanelAppearance;

pub type ComboBoxKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxScrollWheelHandler = Box<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxClearClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxTriggerClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxTriggerMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ComboBoxTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;

pub struct ComboBoxTemplateHandlers {
    pub key_down: ComboBoxKeyDownHandler,
    pub scroll_wheel: ComboBoxScrollWheelHandler,
    pub clear_click: ComboBoxClearClickHandler,
    pub trigger_click: ComboBoxTriggerClickHandler,
    pub trigger_mouse_down: ComboBoxTriggerMouseDownHandler,
    pub trigger_bounds: ComboBoxTriggerBoundsHandler,
}

pub struct ComboBoxRenderModel {
    pub textfield: AnyElement,
    pub query_is_empty: bool,
    pub show_down_arrow: bool,
    pub show_clear_button: bool,
    pub full_width: bool,
    pub minimum_trigger_width: Pixels,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_appearance: SelectorItemsPanelAppearance,
    pub popup_content: Option<AnyElement>,
}

pub trait ComboBoxTemplate: Send + Sync {
    fn render(
        &self,
        model: ComboBoxRenderModel,
        handlers: ComboBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement;
}

pub struct DefaultComboBoxTemplate;

pub fn default_combobox_template() -> Arc<dyn ComboBoxTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ComboBoxTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultComboBoxTemplate)).clone()
}

impl ComboBoxTemplate for DefaultComboBoxTemplate {
    fn render(
        &self,
        model: ComboBoxRenderModel,
        handlers: ComboBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let ComboBoxTemplateHandlers {
            key_down,
            scroll_wheel,
            clear_click,
            trigger_click,
            trigger_mouse_down,
            trigger_bounds,
        } = handlers;

        div()
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
                    .on_mouse_down(MouseButton::Left, trigger_mouse_down)
                    .min_w(model.minimum_trigger_width)
                    .when(model.full_width, |row| row.w_full())
                    .relative()
                    .child(model.textfield)
                    .when(model.show_down_arrow, |row| {
                        row.child(
                            div()
                                .id("combobox-arrow")
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
                                .on_click(trigger_click)
                                .child(lucide_icon(lucide_icons::Icon::ChevronDown, model.muted_text_color, 12.0)),
                        )
                    })
                    .when(model.show_clear_button && !model.query_is_empty, |row| {
                        row.child(
                            div()
                                .id("combobox-clear")
                                .absolute()
                                .top(px(0.0))
                                .right(if model.show_down_arrow { px(30.0) } else { px(10.0) })
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
                                .anchor(gpui::Corner::TopLeft)
                                .position(point(bounds.left(), bounds.bottom()))
                                .offset(point(px(0.0), px(4.0)))
                                .child(div().id("combobox-popup-shell").w(bounds.size.width).child(popup_content)),
                        )
                        .with_priority(1),
                    )
                })
            })
            .into_any_element()
    }
}

pub use super::items_template::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers, default_combobox_items_template,
};
