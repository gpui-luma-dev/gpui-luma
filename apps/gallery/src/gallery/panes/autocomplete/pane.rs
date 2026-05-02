use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, IntoElement, Render, ScrollWheelEvent, SharedString, Subscription,
    Window, div, prelude::*, px,
};
use gpui_luma::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::scrollbar::ScrollbarEvent;
use gpui_luma::theme::{DefaultFloatingMenuTheme, FloatingMenuTheme};

use gpui_luma::controls::autocomplete::{self, AutocompleteTextBox, AutocompleteTextBoxEvent, SelectionItem};
use super::popup_scroll_surface::PopupScrollSurface;
use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_usage_description, notify_entity};
use crate::gallery::theme::GalleryThemePack;

#[derive(Clone)]
pub(in crate::gallery) struct AutocompleteTextFieldPane {
    autocomplete_textbox: AutocompleteTextBox,
    popup_surface_demo: Entity<PopupScrollSurfaceDemo>,
}

impl AutocompleteTextFieldPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let theme_clone = theme.clone();
        let demo_items = autocomplete_demo_items();
        let autocomplete_textbox = autocomplete::new("prototype-autocomplete", demo_items)
            .placeholder("Start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .textfield_template(theme_clone.textfield_template())
            .scrollbar_template(theme_clone.scrollbar_template())
            .spawn(cx);

        let theme_clone = theme.clone();
        let popup_surface_demo = cx.new(|cx| PopupScrollSurfaceDemo::new(theme_clone, cx));

        Self { autocomplete_textbox, popup_surface_demo }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(
            &self.autocomplete_textbox,
            |app, _, _event: &AutocompleteTextBoxEvent, cx| {
                app.panes.autocomplete_textfield.notify_controls(cx);
            },
        ));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage_description(
            "Autocomplete TextField",
            Some("Autocomplete text field"),
            "Floating Menu",
            div()
                .w(px(200.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .child(self.autocomplete_textbox.clone())
                .child(self.popup_surface_demo.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.autocomplete_textbox, cx);
        notify_entity(&self.popup_surface_demo, cx);
    }
}

struct PopupScrollSurfaceDemo {
    theme: GalleryThemePack,
    surface: PopupScrollSurface,
    items: Vec<MenuItem>,
    wheel_count: usize,
    initial_visibility_synced: bool,
    _subscriptions: Vec<Subscription>,
}

impl PopupScrollSurfaceDemo {
    fn new(theme: GalleryThemePack, cx: &mut Context<Self>) -> Self {
        let surface = PopupScrollSurface::new("prototype-popup-surface", theme.scrollbar_template(), cx);

        let items = (0..80)
            .map(|index| MenuItem::new(format!("demo-item-{index}")).label(format!("Option {:02}", index + 1)))
            .collect::<Vec<_>>();

        let subscriptions =
            vec![cx.subscribe(&surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| match event {
                ScrollbarEvent::Change { value } => {
                    this.surface.set_vertical_offset(*value, cx);
                    cx.notify();
                }
            })];

        Self {
            theme,
            surface,
            items,
            wheel_count: 0,
            initial_visibility_synced: false,
            _subscriptions: subscriptions,
        }
    }
}

impl PopupScrollSurfaceDemo {
    fn handle_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let delta_y = event.delta.pixel_delta(px(20.0)).y.as_f32();
        let moved = self.surface.scroll_wheel(event, cx);

        if moved {
            self.wheel_count += 1;
            cx.notify();
        }

        if delta_y.is_finite() && delta_y.abs() > f32::EPSILON {
            cx.stop_propagation();
        }
    }
}

impl Render for PopupScrollSurfaceDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let appearance = DefaultFloatingMenuTheme::new(self.theme.tokens()).resolve();
        let row_height = px(appearance.item_height);
        let viewport_height = px(220.0);
        let content_top_padding = px(appearance.padding);

        self.surface.configure(self.items.len(), row_height, content_top_padding, viewport_height);
        if !self.initial_visibility_synced {
            self.surface.ensure_item_visible(24, cx);
            self.initial_visibility_synced = true;
        }
        self.surface.sync(cx);

        let item_hovers =
            (0..self.items.len()).map(|_| Box::new(noop_hover) as FloatingMenuHoverHandler).collect::<Vec<_>>();
        let item_clicks =
            (0..self.items.len()).map(|_| Box::new(noop_click) as FloatingMenuClickHandler).collect::<Vec<_>>();

        let content = div()
            .id("prototype-popup-surface-wheel-capture")
            .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel))
            .child(render_popup_rows(
                &SharedString::from("prototype-popup-surface-menu"),
                &self.items,
                appearance.clone(),
                None,
                item_hovers,
                item_clicks,
            ));

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(self.theme.chrome().muted_text)
                    .child("Standalone popup scroll surface (for combobox reuse)"),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .line_height(px(15.0))
                    .text_color(self.theme.chrome().muted_text)
                    .child(format!("wheel events captured: {}", self.wheel_count)),
            )
            .child(
                div()
                    .w(px(300.0))
                    .bg(appearance.background)
                    .border_1()
                    .border_color(appearance.border)
                    .rounded(px(appearance.radius))
                    .overflow_hidden()
                    .child(self.surface.render(content.into_any_element())),
            )
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn render_popup_rows(
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

fn autocomplete_demo_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem::new("alabama", "Alabama"),
        SelectionItem::new("alaska", "Alaska"),
        SelectionItem::new("arizona", "Arizona"),
        SelectionItem::new("arkansas", "Arkansas"),
        SelectionItem::new("california", "California"),
        SelectionItem::new("colorado", "Colorado"),
        SelectionItem::new("connecticut", "Connecticut"),
        SelectionItem::new("delaware", "Delaware"),
        SelectionItem::new("florida", "Florida"),
        SelectionItem::new("georgia", "Georgia"),
        SelectionItem::new("hawaii", "Hawaii"),
        SelectionItem::new("idaho", "Idaho"),
        SelectionItem::new("illinois", "Illinois"),
        SelectionItem::new("indiana", "Indiana"),
        SelectionItem::new("iowa", "Iowa"),
        SelectionItem::new("kansas", "Kansas"),
        SelectionItem::new("kentucky", "Kentucky"),
    ]
}
