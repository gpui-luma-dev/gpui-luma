use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui::{
    AnyElement, App, Context, Entity, FontFeatures, FontWeight, MouseButton, Pixels, Render, ScrollWheelEvent, Size,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowDismissPolicy, OverlayWindowMode};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::theme::{ControlSize, LumaTextStyle};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnRadius, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::{ControlDocEntry, catalog_entry as controls_catalog_entry, ControlExposition};
use super::catalog::{ComponentCatalogEntry, ComponentCatalogGroup, controls_exposition_id, groups_for_column};

const COLUMN_COUNT: usize = 4;
const CATEGORY_ICON_SIZE: f32 = 14.0;
const DETAIL_DIALOG_WIDTH: f32 = 1000.0;
const DETAIL_DIALOG_VERTICAL_PAD: f32 = 300.0;

pub struct ComponentsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    detail_dialogs: HashMap<&'static str, ComponentDetailDialog>,
    _subscriptions: Vec<Subscription>,
}

struct ComponentDetailDialog {
    overlay: OverlayWindow,
    /// `(width, max_height)` — height follows content up to `max_height`.
    bounds: Arc<Mutex<(f32, f32)>>,
}

impl ComponentsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let expositions = ControlExposition::spawn_all(look.clone(), cx);
        let mut detail_dialogs = HashMap::new();
        let mut subscriptions = Vec::new();

        for exposition in &expositions {
            let exposition_id = exposition.id(cx);
            let Some(entry) = controls_catalog_entry(exposition_id) else {
                continue;
            };

            let (dialog, dialog_subscriptions) = spawn_detail_dialog(look.clone(), *entry, exposition, cx);
            detail_dialogs.insert(exposition_id, dialog);
            subscriptions.extend(dialog_subscriptions);
        }

        Self { look, expositions, detail_dialogs, _subscriptions: subscriptions }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for exposition in &self.expositions {
            exposition.sync_look(look.clone(), cx);
        }
        for dialog in self.detail_dialogs.values() {
            dialog.overlay.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    fn open_detail(&self, gallery_id: &'static str, window: &Window, cx: &mut Context<Self>) {
        let Some(exposition_id) = controls_exposition_id(gallery_id) else {
            return;
        };
        let Some(dialog) = self.detail_dialogs.get(exposition_id) else {
            return;
        };
        *dialog.bounds.lock().expect("detail dialog bounds") = detail_dialog_bounds(window.viewport_size(), &self.look);
        dialog.overlay.update(cx, |overlay, cx| overlay.open(cx));
    }
}

impl Render for ComponentsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let category_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let item_style = self.look.typography_scale(ShadcnTextSize::Sm);

            div()
                .id("luma-studio-components")
                .size_full()
                .min_h_0()
                .overflow_y_scroll()
                .bg(chrome.content_background)
                .p_cn(6.0)
                .flex()
                .flex_col()
                .items_start()
                .child(
                    div()
                        .id("components-catalog-panel")
                        .flex_shrink_0()
                        .border_1()
                        .border_color(chrome.border)
                        .rounded_cn(ShadcnRadius::Lg)
                        .bg(chrome.panel_background)
                        .p_cn(4.0)
                        .child(div().flex_shrink_0().flex().items_start().gap_cn(7.0).children(
                            (0..COLUMN_COUNT).map(|column| self.render_column(column, category_style, item_style, cx)),
                        )),
                )
                .children(self.detail_dialogs.values().map(|dialog| dialog.overlay.clone()))
        })
    }
}

impl ComponentsPanel {
    fn render_column(
        &self,
        column: usize,
        category_style: LumaTextStyle,
        item_style: LumaTextStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_cn(3.0)
            .children(
                groups_for_column(column)
                    .map(|group| self.render_category_section(group, category_style, item_style, cx)),
            )
            .into_any_element()
    }

    fn render_category_section(
        &self,
        group: &ComponentCatalogGroup,
        category_style: LumaTextStyle,
        item_style: LumaTextStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_cn(0.5)
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_cn(1.5)
                    .whitespace_nowrap()
                    .child(render_category_icon(group.icon))
                    .child(
                        div()
                            .typography_style(category_style)
                            .font_weight(FontWeight::SEMIBOLD)
                            .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1), ("c2sc".into(), 1)])))
                            .child(group.label),
                    ),
            )
            .children(group.entries.iter().map(|entry| self.render_component_item(entry, item_style, cx)))
            .into_any_element()
    }

    fn render_component_item(
        &self,
        entry: &ComponentCatalogEntry,
        item_style: LumaTextStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let chrome = self.look.chrome();
        let gallery_id = entry.id;
        let has_dialog = controls_exposition_id(gallery_id)
            .is_some_and(|exposition_id| self.detail_dialogs.contains_key(exposition_id));

        div()
            .id(format!("components-item-{gallery_id}"))
            .flex_shrink_0()
            .whitespace_nowrap()
            .px_cn(0.75)
            .py_cn(0.5)
            .rounded_cn(ShadcnRadius::Sm)
            .typography_style(item_style)
            .font_weight(FontWeight::NORMAL)
            .when(has_dialog, |item| item.cursor_pointer())
            .when(has_dialog, |item| {
                item.hover(move |style| {
                    style.bg(gpui::hsla(chrome.muted_text.h, chrome.muted_text.s, chrome.muted_text.l, 0.10))
                })
            })
            .when(has_dialog, |item| {
                item.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                        this.open_detail(gallery_id, window, cx);
                    }),
                )
            })
            .child(entry.label)
            .into_any_element()
    }
}

fn spawn_detail_dialog(
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    exposition: &ControlExposition,
    cx: &mut Context<ComponentsPanel>,
) -> (ComponentDetailDialog, Vec<Subscription>) {
    let close = look.ghost_button(format!("components-detail-close-{}", entry.id)).label("Close").spawn(cx);
    let title = entry.title;
    let bounds = Arc::new(Mutex::new((DETAIL_DIALOG_WIDTH, DETAIL_DIALOG_WIDTH)));

    let overlay = match exposition {
        ControlExposition::Button(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
        ControlExposition::Checkbox(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
        ControlExposition::RadioButton(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
        ControlExposition::Switch(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
        ControlExposition::ColorSlider(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
        ControlExposition::TextField(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
        ControlExposition::ModalOverlay(entity) => spawn_component_detail_overlay(
            look.clone(),
            entry.id,
            title,
            entity.clone(),
            close.clone(),
            bounds.clone(),
            cx,
            |entity| entity.clone().into_any_element(),
        ),
    };

    let mut subscriptions = Vec::new();
    subscriptions.push(cx.subscribe(&close, {
        let overlay = overlay.clone();
        move |_, _, event: &ButtonEvent, cx| {
            if !event.is_click() {
                return;
            }
            overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
        }
    }));

    (ComponentDetailDialog { overlay, bounds }, subscriptions)
}

fn spawn_component_detail_overlay<T: 'static>(
    look: Arc<ShadcnLook>,
    entry_id: &'static str,
    title: &'static str,
    exposition_entity: Entity<T>,
    close: Entity<Button>,
    bounds: Arc<Mutex<(f32, f32)>>,
    cx: &mut Context<ComponentsPanel>,
    render_exposition: impl Fn(&Entity<T>) -> AnyElement + Send + Sync + 'static,
) -> OverlayWindow {
    let render_exposition = Arc::new(render_exposition);
    look.overlay_window(format!("components-detail-{entry_id}"))
        .mode(OverlayWindowMode::Modal)
        .size(ControlSize::Lg)
        .dismiss_policy(OverlayWindowDismissPolicy::CloseOnClickAway)
        .with_template_modifier({
            let bounds = bounds.clone();
            move |shell, _| {
                let (width, max_height) = *bounds.lock().expect("detail dialog bounds");
                shell
                    .w(px(width))
                    .min_w(px(width))
                    .max_w(px(width))
                    .max_h(px(max_height))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_scroll_wheel(block_dialog_scroll_wheel)
            }
        })
        .content({
            let look = look.clone();
            let bounds = bounds.clone();
            let exposition_entity = exposition_entity.clone();
            let close = close.clone();
            let render_exposition = render_exposition.clone();
            move |_, window, _| {
                let dialog_bounds = detail_dialog_bounds(window.viewport_size(), &look);
                *bounds.lock().expect("detail dialog bounds") = dialog_bounds;
                render_detail_dialog_content(
                    look.clone(),
                    title,
                    render_exposition(&exposition_entity),
                    close.clone(),
                    dialog_bounds.1,
                )
            }
        })
        .theme_child(exposition_entity.clone())
        .theme_child(close.clone())
        .spawn(cx)
}

fn detail_dialog_bounds(viewport: Size<Pixels>, look: &ShadcnLook) -> (f32, f32) {
    let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
    let margin = spacing * 6.0;
    let viewport_w = viewport.width.as_f32();
    let viewport_h = viewport.height.as_f32();
    let available_w = (viewport_w - margin * 2.0).max(DETAIL_DIALOG_WIDTH);
    let width = DETAIL_DIALOG_WIDTH.min(available_w);
    let max_height = (viewport_h - DETAIL_DIALOG_VERTICAL_PAD * 2.0).max(spacing * 35.0);
    (width, max_height)
}

fn render_detail_dialog_content(
    look: Arc<ShadcnLook>,
    _title: &'static str,
    exposition: AnyElement,
    close: Entity<Button>,
    max_height: f32,
) -> AnyElement {
    let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
    let chrome_allowance = spacing * 22.0;
    let max_body_height = (max_height - chrome_allowance).max(spacing * 10.0);

    with_look(&look, || {
        div()
            .id("components-detail-content")
            .w_full()
            .flex()
            .flex_col()
            .gap_cn(4.0)
            .p_cn(5.0)
            .on_scroll_wheel(block_dialog_scroll_wheel)
            .child(div().w_full().flex_shrink_0().flex().items_center().justify_end().child(close))
            .child(
                div()
                    .id("components-detail-body")
                    .w_full()
                    .max_h(px(max_body_height))
                    .min_h_0()
                    .overflow_y_scroll()
                    .on_scroll_wheel(stop_dialog_scroll_wheel)
                    .child(exposition),
            )
            .into_any_element()
    })
}

fn stop_dialog_scroll_wheel(_: &ScrollWheelEvent, _: &mut Window, cx: &mut App) {
    cx.stop_propagation();
}

fn block_dialog_scroll_wheel(_: &ScrollWheelEvent, window: &mut Window, cx: &mut App) {
    window.prevent_default();
    cx.stop_propagation();
}

fn render_category_icon(icon: LucideIcon) -> AnyElement {
    div()
        .flex_shrink_0()
        .font_family("lucide")
        .text_size(px(CATEGORY_ICON_SIZE))
        .line_height(px(CATEGORY_ICON_SIZE))
        .child(char::from(icon).to_string())
        .into_any_element()
}
