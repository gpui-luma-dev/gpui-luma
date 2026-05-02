use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Hsla, KeyDownEvent, Pixels, ScrollWheelEvent, SharedString, Window, anchored,
    deferred, div, point, prelude::*, px,
};
use gpui_luma::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler};
use gpui_luma::controls::menu_item::MenuItem;

use super::text_selection;

pub(super) type AutocompleteTextBoxKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub(super) type AutocompleteTextBoxScrollWheelHandler = Box<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static>;
pub(super) type AutocompleteTextBoxClearClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub(super) type AutocompleteTextBoxTriggerBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;

pub(super) struct AutocompleteTextBoxTemplateHandlers {
    pub key_down: AutocompleteTextBoxKeyDownHandler,
    pub scroll_wheel: AutocompleteTextBoxScrollWheelHandler,
    pub clear_click: AutocompleteTextBoxClearClickHandler,
    pub trigger_bounds: AutocompleteTextBoxTriggerBoundsHandler,
}

pub(super) struct AutocompleteTextBoxRenderModel {
    pub textfield: text_selection::TextSelection,
    pub query_is_empty: bool,
    pub status_label: SharedString,
    pub status_detail: SharedString,
    pub status_color: Hsla,
    pub muted_text_color: Hsla,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_appearance: gpui_luma::theme::FloatingMenuAppearance,
    pub popup_content: Option<AnyElement>,
}

pub(super) trait AutocompleteTextBoxTemplate: Send + Sync {
    fn render(
        &self,
        model: AutocompleteTextBoxRenderModel,
        handlers: AutocompleteTextBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement;
}

pub(super) struct DefaultAutocompleteTextBoxTemplate;

pub(super) fn default_autocomplete_textbox_template() -> Arc<dyn AutocompleteTextBoxTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AutocompleteTextBoxTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultAutocompleteTextBoxTemplate)).clone()
}

impl AutocompleteTextBoxTemplate for DefaultAutocompleteTextBoxTemplate {
    fn render(
        &self,
        model: AutocompleteTextBoxRenderModel,
        handlers: AutocompleteTextBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let AutocompleteTextBoxTemplateHandlers { key_down, scroll_wheel, clear_click, trigger_bounds } = handlers;

        let status_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(model.status_color)
                    .child(model.status_label),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(model.muted_text_color)
                    .child(model.status_detail),
            );

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(status_row)
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
                                .child(
                                    div()
                                        .font_family("lucide")
                                        .text_size(px(12.0))
                                        .line_height(px(12.0))
                                        .child(char::from(lucide_icons::Icon::X).to_string()),
                                ),
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
                                .child(
                                    div()
                                        .id("prototype-autocomplete-popup-shell")
                                        .w(bounds.size.width)
                                        .bg(model.popup_appearance.background)
                                        .border_1()
                                        .border_color(model.popup_appearance.border)
                                        .rounded(px(model.popup_appearance.radius))
                                        .overflow_hidden()
                                        .child(popup_content),
                                ),
                        )
                        .with_priority(1),
                    )
                })
            })
            .into_any_element()
    }
}

pub(super) fn render_popup_rows(
    id: &SharedString,
    items: &[MenuItem],
    appearance: gpui_luma::theme::FloatingMenuAppearance,
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
