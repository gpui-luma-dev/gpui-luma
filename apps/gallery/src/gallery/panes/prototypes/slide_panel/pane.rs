use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, Focusable, FontWeight, Hsla, IntoElement, MouseDownEvent, Overflow,
    Render, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent, HasPresenter};
use gpui_luma::{flow, hstack, vstack};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::control::{
    SlidePanelEdge, SlidePanelOverlayHandlers, SlidePanelState, SlidePanelTopAnchor, render_slide_panel_overlay,
};

const PANEL_SIDE_TOP_PADDING: f32 = 42.0;
const PANEL_VERTICAL_TOP_PADDING: f32 = 22.0;
const PANEL_CONTENT_INSET: f32 = 22.0;
const PANEL_SECTION_GAP: f32 = 18.0;
const PANEL_ACTION_GAP: f32 = 10.0;
const PANEL_HEADER_GAP: f32 = 16.0;
const PANEL_HEADER_TEXT_GAP: f32 = 6.0;

const PANE_PADDING: f32 = 28.0;
const PANE_SECTION_GAP: f32 = 24.0;
const PANE_HEADER_GAP: f32 = 4.0;
const PANE_DESCRIPTION_MAX_WIDTH: f32 = 760.0;

const STAGE_MAX_WIDTH: f32 = 960.0;
const STAGE_MIN_HEIGHT: f32 = 420.0;
const STAGE_RADIUS: f32 = 20.0;
const STAGE_PADDING: f32 = 24.0;
const STAGE_DESCRIPTION_MAX_WIDTH: f32 = 700.0;
const STAGE_TRIGGER_GAP: f32 = 12.0;

const STATUS_RADIUS: f32 = 16.0;
const STATUS_PADDING: f32 = 18.0;
const STATUS_BORDER_OPACITY: f32 = 0.78;
const STATUS_BACKGROUND_OPACITY: f32 = 0.36;

const INFO_CARD_RADIUS: f32 = 12.0;
const INFO_CARD_PADDING: f32 = 14.0;
const STATS_BORDER_OPACITY: f32 = 0.88;
const STATS_BACKGROUND_OPACITY: f32 = 0.38;
const NOTE_BORDER_OPACITY: f32 = 0.72;
const NOTE_BACKGROUND_OPACITY: f32 = 0.22;

const TITLE_TEXT_SIZE: f32 = 20.0;
const TITLE_LINE_HEIGHT: f32 = 28.0;
const PANEL_TITLE_TEXT_SIZE: f32 = 22.0;
const PANEL_TITLE_LINE_HEIGHT: f32 = 28.0;
const STAGE_TITLE_TEXT_SIZE: f32 = 24.0;
const STAGE_TITLE_LINE_HEIGHT: f32 = 30.0;
const BODY_TEXT_SIZE: f32 = 15.0;
const BODY_LINE_HEIGHT: f32 = 22.0;
const DETAIL_TEXT_SIZE: f32 = 13.0;
const DETAIL_LINE_HEIGHT: f32 = 18.0;
const STAGE_DETAIL_LINE_HEIGHT: f32 = 19.0;
const LABEL_TEXT_SIZE: f32 = 12.0;
const LABEL_LINE_HEIGHT: f32 = 16.0;
const NOTE_LINE_HEIGHT: f32 = 18.0;

const PROGRESS_PERCENT_SCALE: f32 = 100.0;

#[derive(Clone)]
pub(in crate::gallery) struct SlidePanelPane {
    demo: Entity<SlidePanelDemo>,
}

impl SlidePanelPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self { demo: cx.new(|cx| SlidePanelDemo::new(look, cx)) }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        let (
            trigger_left,
            trigger_right,
            trigger_top,
            trigger_bottom,
            close_button,
            primary_action,
            secondary_action,
            archive_action,
            backdrop_toggle,
        ) = {
            let demo = self.demo.read(cx);
            (
                demo.trigger_left.clone(),
                demo.trigger_right.clone(),
                demo.trigger_top.clone(),
                demo.trigger_bottom.clone(),
                demo.close_button.clone(),
                demo.primary_action.clone(),
                demo.secondary_action.clone(),
                demo.archive_action.clone(),
                demo.backdrop_toggle.clone(),
            )
        };

        subscriptions.push(cx.subscribe(&trigger_left, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_trigger_open(SlidePanelEdge::Left, cx);
        }));
        subscriptions.push(cx.subscribe(&trigger_right, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_trigger_open(SlidePanelEdge::Right, cx);
        }));
        subscriptions.push(cx.subscribe(&trigger_top, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_trigger_open(SlidePanelEdge::Top, cx);
        }));
        subscriptions.push(cx.subscribe(&trigger_bottom, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_trigger_open(SlidePanelEdge::Bottom, cx);
        }));
        subscriptions.push(cx.subscribe(&close_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_close_button(cx);
        }));
        subscriptions.push(cx.subscribe(&primary_action, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_primary_action(cx);
        }));
        subscriptions.push(cx.subscribe(&secondary_action, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_secondary_action(cx);
        }));
        subscriptions.push(cx.subscribe(&archive_action, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_archive_action(cx);
        }));
        subscriptions.push(cx.subscribe(&backdrop_toggle, |app, _, _: &ButtonEvent, cx| {
            app.panes.slide_panel.handle_backdrop_toggle(cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, _look: &ShadcnLook) -> AnyElement {
        self.demo.clone().into_any_element()
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.notify_controls(cx);
            cx.notify();
        });
    }

    fn handle_trigger_open(&self, edge: SlidePanelEdge, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.open_from_trigger(edge, cx);
            cx.notify();
        });
    }

    fn handle_close_button(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.last_action = "Closed from the panel close button.".to_string();
            demo.state.request_close();
            cx.notify();
        });
    }

    fn handle_primary_action(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.last_action = "Primary action pressed while the panel stayed open.".to_string();
            cx.notify();
        });
    }

    fn handle_secondary_action(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.last_action = "Secondary review action fired.".to_string();
            cx.notify();
        });
    }

    fn handle_archive_action(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            demo.last_action = "Ghost archive action fired.".to_string();
            cx.notify();
        });
    }

    fn handle_backdrop_toggle(&self, cx: &mut Context<GalleryApp>) {
        self.demo.update(cx, |demo, cx| {
            let enabled = !demo.state.backdrop_click_closes();
            demo.state.set_backdrop_click_closes(enabled);
            demo.backdrop_toggle.update(cx, |button, cx| button.set_data(enabled, cx));
            demo.last_action = if enabled {
                "Backdrop click dismissal enabled.".to_string()
            } else {
                "Backdrop click dismissal disabled.".to_string()
            };
            cx.notify();
        });
    }
}

struct SlidePanelDemo {
    look: Arc<ShadcnLook>,
    trigger_left: Entity<Button>,
    trigger_right: Entity<Button>,
    trigger_top: Entity<Button>,
    trigger_bottom: Entity<Button>,
    close_button: Entity<Button>,
    primary_action: Entity<Button>,
    secondary_action: Entity<Button>,
    archive_action: Entity<Button>,
    backdrop_toggle: Entity<Button<bool>>,
    state: SlidePanelState,
    last_action: String,
}

impl SlidePanelDemo {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let trigger_left = look.primary_button("slide-panel-open-left").label("Open Left").spawn(cx);
        let trigger_right = look.primary_button("slide-panel-open-right").label("Open Right").spawn(cx);
        let trigger_top = look.primary_button("slide-panel-open-top").label("Open Top").spawn(cx);
        let trigger_bottom = look.primary_button("slide-panel-open-bottom").label("Open Bottom").spawn(cx);
        let close_button = look.ghost_icon_button("slide-panel-close", LucideIcon::X).spawn(cx);
        let primary_action = look.primary_button("slide-panel-primary").label("Apply Changes").spawn(cx);
        let secondary_action = look.secondary_button("slide-panel-secondary").label("Review Draft").spawn(cx);
        let archive_action = look.ghost_button("slide-panel-archive").label("Archive").spawn(cx);
        let backdrop_toggle = look
            .secondary_toggle("slide-panel-backdrop-toggle")
            .with_data(true)
            .content(|model, _| {
                let label = if model.data {
                    "Backdrop click closes: On"
                } else {
                    "Backdrop click closes: Off"
                };
                div().child(label).into_any_element()
            })
            .spawn(cx);

        Self {
            look,
            trigger_left,
            trigger_right,
            trigger_top,
            trigger_bottom,
            close_button,
            primary_action,
            secondary_action,
            archive_action,
            backdrop_toggle,
            state: SlidePanelState::new(SlidePanelTopAnchor::BelowTopBar),
            last_action: "Panel closed. Use one of the edge triggers to open a drawer.".to_string(),
        }
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
        for button in [
            &self.trigger_left,
            &self.trigger_right,
            &self.trigger_top,
            &self.trigger_bottom,
            &self.close_button,
            &self.primary_action,
            &self.secondary_action,
            &self.archive_action,
        ] {
            notify_button(button, cx);
        }
        notify_button(&self.backdrop_toggle, cx);
    }

    fn open_from_trigger(&mut self, edge: SlidePanelEdge, cx: &mut Context<Self>) {
        let opener = match edge {
            SlidePanelEdge::Left => self.trigger_left.read(cx).focus_handle(cx),
            SlidePanelEdge::Right => self.trigger_right.read(cx).focus_handle(cx),
            SlidePanelEdge::Top => self.trigger_top.read(cx).focus_handle(cx),
            SlidePanelEdge::Bottom => self.trigger_bottom.read(cx).focus_handle(cx),
        };
        self.state.open(edge, opener);
        self.last_action = format!("Opened the {} slide panel.", edge.label());
    }

    fn extra_focus_handles(&self, cx: &App) -> Vec<FocusHandle> {
        vec![
            self.primary_action.read(cx).focus_handle(cx),
            self.secondary_action.read(cx).focus_handle(cx),
            self.archive_action.read(cx).focus_handle(cx),
            self.backdrop_toggle.read(cx).focus_handle(cx),
        ]
    }

    fn handle_overlay_key_down(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" => {
                self.last_action = "Closed with Escape and restored focus to the trigger.".to_string();
                if self.state.handle_escape(window, cx) {
                    cx.notify();
                }
            }
            "tab" => {
                let close_focus = self.close_button.read(cx).focus_handle(cx);
                let extra_focuses = self.extra_focus_handles(cx);
                if self.state.handle_tab_navigation(close_focus, &extra_focuses, event, window, cx) {
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn handle_backdrop_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.state.backdrop_click_closes() {
            return;
        }

        self.last_action = "Closed by clicking the backdrop.".to_string();
        if self.state.request_close() {
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn render_panel_content(&self, edge: SlidePanelEdge, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let panel_foreground = look.token_color("foreground").unwrap_or(chrome.title_text);
        let muted = look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
        let border = look.token_color("border").unwrap_or(chrome.border);
        let content_padding_top = px(match edge {
            SlidePanelEdge::Left | SlidePanelEdge::Right => PANEL_SIDE_TOP_PADDING,
            SlidePanelEdge::Top | SlidePanelEdge::Bottom => PANEL_VERTICAL_TOP_PADDING,
        });

        let mut panel_content = vstack! {
            gap=PANEL_SECTION_GAP justify=between;
            vstack! {
                gap=PANEL_SECTION_GAP;
                render_panel_header(edge, self.close_button.clone().into_any_element(), panel_foreground, muted),
                render_panel_stats_card(
                    edge,
                    self.state.backdrop_click_closes(),
                    panel_foreground,
                    muted,
                    border,
                    chrome.content_background,
                ),
                flow! {
                    gap=PANEL_ACTION_GAP;
                    self.primary_action.clone(),
                    self.secondary_action.clone(),
                    self.archive_action.clone(),
                    self.backdrop_toggle.clone(),
                }
                .items_center(),
            },
            render_panel_focus_note(border, chrome.content_background, muted),
        }
        .size_full()
        .min_h_0()
        .pt(content_padding_top)
        .pr(px(PANEL_CONTENT_INSET))
        .pb(px(PANEL_CONTENT_INSET))
        .pl(px(PANEL_CONTENT_INSET));

        if matches!(edge, SlidePanelEdge::Top | SlidePanelEdge::Bottom) {
            panel_content = panel_content.justify_start().gap(px(PANEL_SECTION_GAP));
            panel_content.style().overflow.y = Some(Overflow::Scroll);
        }

        panel_content.into_any_element()
    }
}

impl Render for SlidePanelDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.state.sync_animation();
        self.state.schedule_animation_frame(window, cx);

        let close_focus = self.close_button.read(cx).focus_handle(cx);
        self.state.schedule_pending_focus(window, cx, close_focus);

        let chrome = self.look.chrome();
        let title = self.look.token_color("foreground").unwrap_or(chrome.title_text);
        let body = self.look.token_color("foreground").unwrap_or(chrome.body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
        let border = self.look.token_color("border").unwrap_or(chrome.border);
        let stage_background = self.look.token_color("card").unwrap_or(chrome.panel_background);
        let viewport = window.viewport_size();

        let stage = vstack! {
            gap=PANE_SECTION_GAP justify=between;
            render_stage_intro(title, muted),
            flow! {
                gap=STAGE_TRIGGER_GAP;
                self.trigger_left.clone(),
                self.trigger_right.clone(),
                self.trigger_top.clone(),
                self.trigger_bottom.clone(),
            }
            .items_center(),
            render_status_card(
                self.last_action.clone(),
                self.state.open_progress(),
                self.state.active_edge(),
                body,
                muted,
                border,
                chrome.content_background,
            ),
        }
        .w_full()
        .max_w(px(STAGE_MAX_WIDTH))
        .min_h(px(STAGE_MIN_HEIGHT))
        .rounded(px(STAGE_RADIUS))
        .border_1()
        .border_color(border)
        .bg(stage_background)
        .p(px(STAGE_PADDING));

        let mut root = div()
            .id("slide-panel-prototype-pane")
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(chrome.content_background)
            .p(px(PANE_PADDING))
            .child(
                vstack! {
                    gap=PANE_SECTION_GAP;
                    render_pane_header(
                        "Slide Panel",
                        "Open drawers from each edge and verify close behavior, focus trap, and focus restore without touching Luma Studio.",
                        title,
                        muted,
                    ),
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(stage),
                }
                .size_full(),
            );

        if let Some(edge) = self.state.active_edge() {
            let handlers = SlidePanelOverlayHandlers {
                key_down: Box::new(cx.listener(Self::handle_overlay_key_down)),
                backdrop_mouse_down: Box::new(cx.listener(Self::handle_backdrop_mouse_down)),
            };
            let overlay = render_slide_panel_overlay(
                &self.look,
                &self.state,
                viewport,
                self.render_panel_content(edge, &self.look),
                handlers,
            );
            root = root.child(gpui::deferred(overlay).with_priority(10));
        }

        root
    }
}

fn render_pane_header(
    title: &'static str,
    description: &'static str,
    title_color: Hsla,
    description_color: Hsla,
) -> AnyElement {
    vstack! {
        gap=PANE_HEADER_GAP;
        div()
            .text_size(px(TITLE_TEXT_SIZE))
            .line_height(px(TITLE_LINE_HEIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(title_color)
            .child(title),
        div()
            .max_w(px(PANE_DESCRIPTION_MAX_WIDTH))
            .text_size(px(DETAIL_TEXT_SIZE))
            .line_height(px(DETAIL_LINE_HEIGHT))
            .text_color(description_color)
            .child(description),
    }
    .w_full()
    .into_any_element()
}

fn render_stage_intro(title_color: Hsla, muted: Hsla) -> AnyElement {
    vstack! {
        gap=PANEL_ACTION_GAP;
        div()
            .text_size(px(STAGE_TITLE_TEXT_SIZE))
            .line_height(px(STAGE_TITLE_LINE_HEIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(title_color)
            .child("Slide Panels"),
        div()
            .max_w(px(STAGE_DESCRIPTION_MAX_WIDTH))
            .text_size(px(DETAIL_TEXT_SIZE))
            .line_height(px(STAGE_DETAIL_LINE_HEIGHT))
            .text_color(muted)
            .child("Gallery-only drawer prototype using a deferred window overlay, focus restore, Escape dismissal, and local Tab trapping."),
    }
    .into_any_element()
}

fn render_status_card(
    last_action: String,
    open_progress: f32,
    active_edge: Option<SlidePanelEdge>,
    body: Hsla,
    muted: Hsla,
    border: Hsla,
    content_background: Hsla,
) -> AnyElement {
    vstack! {
        gap=PANEL_ACTION_GAP;
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(LABEL_LINE_HEIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(muted)
            .child("Status"),
        div()
            .text_size(px(BODY_TEXT_SIZE))
            .line_height(px(BODY_LINE_HEIGHT))
            .text_color(body)
            .child(last_action),
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(NOTE_LINE_HEIGHT))
            .text_color(muted)
            .child(format!(
                "Open progress: {:>3.0}%{}",
                open_progress * PROGRESS_PERCENT_SCALE,
                active_edge
                    .map(|edge| format!(" | active edge: {}", edge.label()))
                    .unwrap_or_else(|| " | panel closed".to_string())
            )),
    }
    .rounded(px(STATUS_RADIUS))
    .border_1()
    .border_color(border.opacity(STATUS_BORDER_OPACITY))
    .bg(content_background.opacity(STATUS_BACKGROUND_OPACITY))
    .p(px(STATUS_PADDING))
    .into_any_element()
}

fn render_panel_header(edge: SlidePanelEdge, close_button: AnyElement, foreground: Hsla, muted: Hsla) -> AnyElement {
    hstack! {
        justify=between align=start gap=PANEL_HEADER_GAP;
        vstack! {
            gap=PANEL_HEADER_TEXT_GAP;
            div()
                .text_size(px(LABEL_TEXT_SIZE))
                .line_height(px(LABEL_LINE_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(muted)
                .child(format!("{} Edge", edge.label())),
            div()
                .text_size(px(PANEL_TITLE_TEXT_SIZE))
                .line_height(px(PANEL_TITLE_LINE_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(foreground)
                .child("Slide Panel Prototype"),
            div()
                .text_size(px(DETAIL_TEXT_SIZE))
                .line_height(px(DETAIL_LINE_HEIGHT))
                .text_color(muted)
                .child(edge.description()),
        },
        close_button,
    }
    .into_any_element()
}

fn render_panel_stats_card(
    edge: SlidePanelEdge,
    backdrop_click_closes: bool,
    foreground: Hsla,
    muted: Hsla,
    border: Hsla,
    content_background: Hsla,
) -> AnyElement {
    vstack! {
        gap=PANEL_ACTION_GAP;
        render_stat_row("Edge", edge.label(), foreground, muted),
        render_stat_row(
            "Backdrop click",
            if backdrop_click_closes { "Enabled" } else { "Disabled" },
            foreground,
            muted,
        ),
        render_stat_row("Focus", "Trapped inside panel while open", foreground, muted),
        render_stat_row("Close", "Escape, close button, or backdrop", foreground, muted),
    }
    .rounded(px(INFO_CARD_RADIUS))
    .border_1()
    .border_color(border.opacity(STATS_BORDER_OPACITY))
    .bg(content_background.opacity(STATS_BACKGROUND_OPACITY))
    .p(px(INFO_CARD_PADDING))
    .into_any_element()
}

fn render_panel_focus_note(border: Hsla, content_background: Hsla, muted: Hsla) -> AnyElement {
    div()
        .rounded(px(INFO_CARD_RADIUS))
        .border_1()
        .border_color(border.opacity(NOTE_BORDER_OPACITY))
        .bg(content_background.opacity(NOTE_BACKGROUND_OPACITY))
        .p(px(INFO_CARD_PADDING))
        .text_size(px(LABEL_TEXT_SIZE))
        .line_height(px(NOTE_LINE_HEIGHT))
        .text_color(muted)
        .child("Tab cycles through the panel actions and returns to the opener when the panel closes.")
        .into_any_element()
}

fn render_stat_row(label: &'static str, value: &'static str, foreground: Hsla, muted: Hsla) -> AnyElement {
    hstack! {
        justify=between align=center gap=STAGE_TRIGGER_GAP;
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(LABEL_LINE_HEIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(muted)
            .child(label),
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(LABEL_LINE_HEIGHT))
            .text_color(foreground)
            .child(value),
    }
    .into_any_element()
}

fn notify_button<T: 'static>(entity: &Entity<T>, cx: &mut Context<SlidePanelDemo>) {
    entity.update(cx, |_, cx| cx.notify());
}
