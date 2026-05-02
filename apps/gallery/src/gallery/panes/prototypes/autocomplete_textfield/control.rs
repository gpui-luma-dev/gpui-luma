use gpui::{
    Bounds, ClickEvent, Context, Corner, Entity, EventEmitter, IntoElement, KeyDownEvent, Pixels, Render,
    ScrollWheelEvent, SharedString, Subscription, Window, anchored, deferred, div, point, prelude::*, px,
};
use lucide_icons::Icon as LucideIcon;
use gpui_luma::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::scrollbar::ScrollbarEvent;
use gpui_luma::theme::{DefaultFloatingMenuTheme, FloatingMenuTheme};

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionItem, SelectionStatus, SubmitResult};
use super::popup_scroll_surface::PopupScrollSurface;
use super::text_selection::{self, TextSelectionEvent};
use crate::gallery::theme::GalleryThemePack;

#[derive(Clone, Debug)]
pub(super) enum AutocompleteTextBoxEvent {
    Change,
    Select,
    Complete,
    Clear,
}

pub(super) type AutocompleteTextBox = Entity<AutocompleteTextBoxControl>;

pub(super) struct AutocompleteTextBoxControl {
    theme: GalleryThemePack,
    textfield: text_selection::TextSelection,
    popup_surface: PopupScrollSurface,
    items: Vec<SelectionItem>,
    behavior: SelectionBehavior,
    trigger_bounds: Option<Bounds<Pixels>>,
    last_event: SharedString,
    last_keyboard_event: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<AutocompleteTextBoxEvent> for AutocompleteTextBoxControl {}

impl AutocompleteTextBoxControl {
    pub(super) fn new(theme: GalleryThemePack, items: Vec<SelectionItem>, cx: &mut Context<Self>) -> Self {
        let textfield = text_selection::new("prototype-autocomplete-textfield")
            .placeholder("Prompt: start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .template(theme.textfield_template())
            .spawn(cx);

        let popup_surface =
            PopupScrollSurface::new("prototype-autocomplete-popup-surface", theme.scrollbar_template(), cx);

        let subscriptions = vec![
            cx.subscribe(&textfield, |this, _, event: &gpui_luma::controls::textfield::TextFieldEvent, cx| {
                this.handle_text_selection_event(text_selection::map_event(event), cx);
            }),
            cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| match event {
                ScrollbarEvent::Change { value } => {
                    this.popup_surface.set_vertical_offset(*value, cx);
                    cx.notify();
                }
            }),
        ];

        Self {
            theme,
            textfield,
            popup_surface,
            items,
            behavior: SelectionBehavior::new(),
            trigger_bounds: None,
            last_event: SharedString::from("none"),
            last_keyboard_event: SharedString::from("none"),
            _subscriptions: subscriptions,
        }
    }

    fn handle_text_selection_event(&mut self, event: TextSelectionEvent, cx: &mut Context<Self>) {
        match event {
            TextSelectionEvent::Change { value } => {
                self.behavior.set_query(value.clone(), &self.items);
                self.last_event = SharedString::from("change");
                cx.emit(AutocompleteTextBoxEvent::Change);
                cx.notify();
            }
            TextSelectionEvent::Submit { value } => {
                self.last_event = SharedString::from(format!("submit:{value}"));
                let result = self.behavior.apply(SelectionEvent::Submit, &self.items);
                self.handle_submit_result(result, cx);
            }
            TextSelectionEvent::FocusEnter => {
                self.behavior.apply(SelectionEvent::Focus, &self.items);
                cx.notify();
            }
            TextSelectionEvent::FocusLeave => {
                if self.behavior.state.status == SelectionStatus::Searching
                    && let Some(index) = self
                        .behavior
                        .state
                        .highlighted_filtered
                        .and_then(|highlighted| self.behavior.state.filtered.get(highlighted).copied())
                {
                    self.select_item(index, false, cx);
                }

                self.behavior.apply(SelectionEvent::Blur, &self.items);
                cx.notify();
            }
        }
    }

    fn handle_submit_result(&mut self, result: SubmitResult, cx: &mut Context<Self>) {
        match result {
            SubmitResult::None => {
                self.last_event = SharedString::from("submit:no-match");
                cx.notify();
            }
            SubmitResult::Select { index, exact_complete } => {
                self.select_item(index, exact_complete, cx);
            }
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.behavior.apply(SelectionEvent::Clear, &self.items);
        self.last_event = SharedString::from("clear");

        self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));

        cx.emit(AutocompleteTextBoxEvent::Clear);
        cx.notify();
    }

    fn select_item(&mut self, index: usize, exact_complete: bool, cx: &mut Context<Self>) {
        let item = &self.items[index];
        let label = item.label.clone();

        self.behavior.set_query(label.clone(), &self.items);
        self.behavior.select_index(index);
        self.last_event = SharedString::from(format!("select:{}", item.label));

        self.textfield.update(cx, move |textfield, cx| textfield.set_value(label.as_ref(), cx));

        if exact_complete {
            cx.emit(AutocompleteTextBoxEvent::Complete);
        } else {
            cx.emit(AutocompleteTextBoxEvent::Select);
        }
        cx.notify();
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if hovered && self.behavior.state.open {
            self.behavior.state.highlighted_filtered = Some(index);
            self.sync_popup_highlight_visibility(cx);
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() || !self.behavior.state.open {
            return;
        }

        if let Some(item_index) = self.behavior.state.filtered.get(index).copied() {
            self.select_item(item_index, false, cx);
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.last_keyboard_event = SharedString::from(format!("key:{}", event.keystroke.key));

        if event.keystroke.modifiers.control || event.keystroke.modifiers.secondary() {
            return;
        }

        match event.keystroke.key.as_str() {
            "escape" => {
                self.behavior.apply(SelectionEvent::Escape, &self.items);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "down" | "arrowdown" => {
                if self.behavior.state.open && !self.behavior.state.filtered.is_empty() {
                    self.behavior.apply(SelectionEvent::MoveNext, &self.items);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "up" | "arrowup" => {
                if self.behavior.state.open && !self.behavior.state.filtered.is_empty() {
                    self.behavior.apply(SelectionEvent::MovePrevious, &self.items);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn handle_clear_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.clear(cx);
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.trigger_bounds = Some(bounds.clone());
    }

    fn handle_popup_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.behavior.state.open || self.behavior.state.filtered.is_empty() {
            return;
        }

        let moved = self.popup_surface.scroll_wheel(event, cx);
        if moved {
            cx.notify();
        }

        let delta_y = event.delta.pixel_delta(px(20.0)).y.as_f32();
        if delta_y.is_finite() && delta_y.abs() > f32::EPSILON {
            cx.stop_propagation();
        }
    }

    fn sync_popup_highlight_visibility(&self, cx: &mut Context<Self>) {
        if !self.behavior.state.open {
            return;
        }

        if let Some(visible_index) = self.behavior.state.highlighted_filtered {
            self.popup_surface.ensure_item_visible(visible_index, cx);
        }
    }
}

impl Render for AutocompleteTextBoxControl {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let appearance = DefaultFloatingMenuTheme::new(self.theme.tokens()).resolve();
        let selected_label = self
            .behavior
            .state
            .selected_item
            .map(|index| self.items[index].label.to_string())
            .unwrap_or_else(|| "none".to_string());

        let menu_items = self
            .behavior
            .state
            .filtered
            .iter()
            .enumerate()
            .map(|(visible_index, item_index)| {
                let item = &self.items[*item_index];
                MenuItem::new(format!("prototype-item-{}-{visible_index}", item.id)).label(item.label.clone())
            })
            .collect::<Vec<_>>();

        let item_hovers = (0..menu_items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(index, *hovered, cx);
                })) as FloatingMenuHoverHandler
            })
            .collect::<Vec<_>>();

        let item_clicks = (0..menu_items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_click(index, event, window, cx);
                })) as FloatingMenuClickHandler
            })
            .collect::<Vec<_>>();

        let tokens = self.theme.tokens();
        let status_color = match self.behavior.state.status {
            SelectionStatus::ErrorNoMatch => tokens.palette.form.input.invalid_border,
            SelectionStatus::Complete => tokens.palette.state.selected.background,
            _ => self.theme.chrome().muted_text,
        };

        let trigger_bounds = cx.listener(Self::handle_trigger_bounds);

        let status_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(status_color)
                    .child(format!("status: {}", self.behavior.state.status.label())),
            )
            .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(self.theme.chrome().muted_text).child(
                format!(
                    "selected: {selected_label} | matches: {} | last: {} | keyboard: {}",
                    self.behavior.state.filtered.len(),
                    self.last_event,
                    self.last_keyboard_event
                ),
            ));

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(status_row)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_scroll_wheel(cx.listener(Self::handle_popup_scroll_wheel))
            .child(
                div()
                    .on_children_prepainted(move |bounds, window, cx| {
                        if let Some(bounds) = bounds.first() {
                            trigger_bounds(bounds, window, cx);
                        }
                    })
                    .w_full()
                    .relative()
                    .child(self.textfield.clone())
                    .when(!self.behavior.state.query.is_empty(), |row| {
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
                                .text_color(self.theme.chrome().muted_text)
                                .hover(|style| style.text_color(self.theme.chrome().body_text))
                                .on_click(cx.listener(Self::handle_clear_click))
                                .child(
                                    div()
                                        .font_family("lucide")
                                        .text_size(px(12.0))
                                        .line_height(px(12.0))
                                        .child(char::from(LucideIcon::X).to_string()),
                                ),
                        )
                    }),
            )
            .when(self.behavior.state.open && !menu_items.is_empty(), |root| {
                root.when_some(self.trigger_bounds.clone(), |root, bounds| {
                    let menu_id = SharedString::from("prototype-autocomplete-menu");
                    let row_height = px(appearance.item_height);
                    let content_top_padding = px(appearance.padding);
                    let max_visible_rows = 7.0;
                    let viewport_height = px((appearance.padding * 2.0) + (appearance.item_height * max_visible_rows));

                    self.popup_surface.configure(menu_items.len(), row_height, content_top_padding, viewport_height);
                    self.popup_surface.sync(cx);

                    let menu_content = render_popup_rows(
                        &menu_id,
                        &menu_items,
                        appearance.clone(),
                        self.behavior.state.highlighted_filtered,
                        item_hovers,
                        item_clicks,
                    );

                    let overlay = deferred(
                        anchored()
                            .snap_to_window_with_margin(px(8.0))
                            .anchor(Corner::TopLeft)
                            .position(point(bounds.left(), bounds.bottom()))
                            .offset(point(px(0.0), px(4.0)))
                            .child(
                                div()
                                    .id("prototype-autocomplete-popup-shell")
                                    .w(bounds.size.width)
                                    .bg(appearance.background)
                                    .border_1()
                                    .border_color(appearance.border)
                                    .rounded(px(appearance.radius))
                                    .overflow_hidden()
                                    .child(self.popup_surface.render(menu_content.into_any_element())),
                            ),
                    )
                    .with_priority(1);

                    root.child(overlay)
                })
            })
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
