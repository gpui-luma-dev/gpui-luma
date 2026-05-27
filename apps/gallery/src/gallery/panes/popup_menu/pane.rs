use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Pixels, Render,
    SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{
    ControlFocusState, PopupMenu, PopupMenuEvent, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplate,
    PopupMenuTemplateHandlers,
};
use gpui_luma::theme::{InteractionState, RadixTheme};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct PopupMenuPane {
    popup_smart: Entity<PopupMenu>,
    popup_below: Entity<PopupMenu>,
    popup_above: Entity<PopupMenu>,
    popup_centered: Entity<PopupMenu>,
    state_preview: Entity<PopupMenuStatePreview>,
    selection: String,
}

impl PopupMenuPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let popup_template = radix_theme.popup_menu_template();
        Self {
            popup_smart: PopupMenu::new("popup-menu-smart-example")
                .template(popup_template.clone())
                .label("Smart popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::Smart)
                .spawn(cx),
            popup_below: PopupMenu::new("popup-menu-below-example")
                .template(popup_template.clone())
                .label("Below popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::BelowStart)
                .spawn(cx),
            popup_above: PopupMenu::new("popup-menu-above-example")
                .template(popup_template.clone())
                .label("Above popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::AboveStart)
                .spawn(cx),
            popup_centered: PopupMenu::new("popup-menu-centered-example")
                .template(popup_template.clone())
                .label("Centered popup")
                .items(menu_items())
                .placement(PopupMenuPlacement::CenteredOnTrigger)
                .spawn(cx),
            state_preview: cx.new(|_| PopupMenuStatePreview::new(radix_theme.clone(), popup_template)),
            selection: "none".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.popup_smart, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.popup_below, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.popup_above, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.popup_centered, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.popup_menu.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage(
            "Popup Menu",
            "Popup Menu",
            div()
                .w_full()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(self.popup_below.clone())
                                .child(self.popup_above.clone()),
                        )
                        .child(div().flex().items_center().gap_3().child(self.popup_centered.clone()))
                        .child(div().text_color(chrome.body_text).child(format!("Selected: {}", self.selection)))
                        .child(self.state_preview.clone()),
                )
                .child(self.popup_smart.clone())
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.popup_smart, cx);
        notify_entity(&self.popup_below, cx);
        notify_entity(&self.popup_above, cx);
        notify_entity(&self.popup_centered, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &PopupMenuEvent, cx: &mut Context<GalleryApp>) {
        match event {
            PopupMenuEvent::Select { label, .. } => {
                self.handle_selection(label, cx);
            }
        }
    }

    fn handle_selection(&mut self, label: &gpui::SharedString, cx: &mut Context<GalleryApp>) {
        self.selection = label.to_string();
        cx.notify();
    }
}

#[derive(Clone)]
struct PopupMenuStatePreview {
    radix_theme: Arc<RadixTheme>,
    template: Arc<dyn PopupMenuTemplate>,
}

struct PopupMenuStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
    focus: ControlFocusState,
}

impl PopupMenuStatePreview {
    fn new(radix_theme: Arc<RadixTheme>, template: Arc<dyn PopupMenuTemplate>) -> Self {
        Self { radix_theme, template }
    }
}

impl Render for PopupMenuStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let samples = [
            PopupMenuStateSample {
                id: "default",
                label: "Standard",
                state: InteractionState::default(),
                focus: ControlFocusState::default(),
            },
            PopupMenuStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
                focus: ControlFocusState::default(),
            },
            PopupMenuStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
                focus: ControlFocusState { focused: true, focus_visible: true },
            },
            PopupMenuStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
                focus: ControlFocusState { focused: true, focus_visible: true },
            },
            PopupMenuStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
                focus: ControlFocusState::default(),
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_trigger_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_trigger_sample(
    template: &Arc<dyn PopupMenuTemplate>,
    sample: PopupMenuStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("popup-menu-preview-trigger-{}", sample.id));
    let label = SharedString::from("Popup");
    let items = menu_items().into_iter().collect::<Vec<_>>();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        open_submenu: None,
        active_path: None,
        enabled: !sample.state.disabled,
        focus: sample.focus,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, popup_menu_preview_handlers(items.len(), 0), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn popup_menu_preview_handlers(root_count: usize, submenu_click_count: usize) -> PopupMenuTemplateHandlers {
    let click_count = root_count + submenu_click_count;

    PopupMenuTemplateHandlers {
        trigger_bounds: Box::new(noop_bounds),
        trigger_click: Box::new(noop_click),
        trigger_hover: Box::new(noop_hover),
        trigger_mouse_down: Box::new(noop_mouse_down),
        trigger_mouse_up: Box::new(noop_mouse_up),
        trigger_mouse_up_out: Box::new(noop_mouse_up),
        root_mouse_down_out: Box::new(noop_mouse_down),
        item_hovers: (0..root_count).map(|_| Box::new(noop_hover) as _).collect(),
        item_clicks: (0..click_count).map(|_| Box::new(noop_click) as _).collect(),
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}
