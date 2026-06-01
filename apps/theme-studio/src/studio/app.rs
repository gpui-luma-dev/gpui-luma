use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    Context, DragMoveEvent, FocusHandle, MouseButton, MouseDownEvent, Point, Pixels, Render, Size, Subscription,
    Window, div, prelude::*, px,
};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::{ControlSize, RadixTheme, ThemeMode};
use lucide_icons::Icon as LucideIcon;

use crate::theme::StudioThemeChoice;

use super::demo_controls::DemoControls;
use super::inspectable::InspectableId;
use super::inspector::{render_inspector, run_export};
use super::overrides::StudioOverrides;
use super::panel_layout::{DemoPanelDrag, default_panel_position, offset_panel_position};
use super::panel_layout_config::{load_panel_positions, load_window_size, save_studio_layout};
use super::panels::render_demo_board;

struct PanelDragState {
    id: InspectableId,
    mouse_origin: Point<Pixels>,
    panel_origin: Point<Pixels>,
}

pub struct ThemeStudioApp {
    focus_scope: FocusHandle,
    pub(super) radix_theme: Arc<RadixTheme>,
    pub(super) control_size: ControlSize,
    pub(super) demos: DemoControls,
    pub(super) panel_positions: HashMap<InspectableId, Point<Pixels>>,
    panel_drag: Option<PanelDragState>,
    pub(super) selected: Option<InspectableId>,
    pub(super) overrides: StudioOverrides,
    pub(super) export_status: String,
    last_window_size: Size<Pixels>,
    _subscriptions: Vec<Subscription>,
}

impl ThemeStudioApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: StudioThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let radix_theme = theme_choice.radix_theme();
        radix_theme.set_mode(ThemeMode::Dark);
        let control_size = ControlSize::Md;
        let demos = DemoControls::spawn(cx, radix_theme.clone(), control_size);

        let mut subscriptions = Vec::new();
        demos.subscribe(cx, &mut subscriptions);
        subscriptions.push(cx.observe_window_bounds(window, |this, window, cx| {
            this.on_window_bounds_changed(window, cx);
        }));

        let last_window_size = load_window_size();

        Self {
            focus_scope,
            radix_theme,
            control_size,
            demos,
            panel_positions: load_panel_positions(),
            panel_drag: None,
            selected: None,
            overrides: StudioOverrides::default(),
            export_status: String::new(),
            last_window_size,
            _subscriptions: subscriptions,
        }
    }

    fn on_window_bounds_changed(&mut self, window: &Window, cx: &mut Context<Self>) {
        let size = window.bounds().size;
        if size == self.last_window_size {
            return;
        }
        self.last_window_size = size;
        self.persist_layout();
        let _ = cx;
    }

    fn persist_layout(&self) {
        if let Err(err) = save_studio_layout(self.last_window_size, &self.panel_positions) {
            tracing::warn!("failed to save studio layout: {err:?}");
        }
    }

    pub fn panel_position(&self, id: InspectableId) -> Point<Pixels> {
        self.panel_positions.get(&id).copied().unwrap_or_else(|| default_panel_position(id))
    }

    pub fn begin_panel_drag(&mut self, id: InspectableId, event: &MouseDownEvent, _cx: &mut Context<Self>) {
        self.panel_drag =
            Some(PanelDragState { id, mouse_origin: event.position, panel_origin: self.panel_position(id) });
    }

    pub fn handle_panel_drag_move(
        &mut self,
        event: &DragMoveEvent<DemoPanelDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ref state) = self.panel_drag else {
            return;
        };
        if event.drag(cx).id != state.id {
            return;
        }
        let delta = event.event.position.relative_to(&state.mouse_origin);
        let position = offset_panel_position(state.panel_origin, delta);
        self.panel_positions.insert(state.id, position);
        cx.notify();
    }

    pub fn end_panel_drag(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.panel_drag.is_some() {
            self.panel_drag = None;
            self.last_window_size = window.bounds().size;
            self.persist_layout();
            cx.notify();
        }
    }

    pub fn select_inspectable(&mut self, id: InspectableId, cx: &mut Context<Self>) {
        self.selected = Some(id);
        cx.notify();
    }

    pub fn close_inspector(&mut self, cx: &mut Context<Self>) {
        self.selected = None;
        cx.notify();
    }

    pub fn set_control_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.control_size == size {
            return;
        }
        self.control_size = size;
        self.demos = DemoControls::spawn(cx, self.radix_theme.clone(), self.control_size);
        self._subscriptions.clear();
        self.demos.subscribe(cx, &mut self._subscriptions);
        cx.notify();
    }

    pub fn adjust_scale(&mut self, id: InspectableId, key: &str, delta: f32, cx: &mut Context<Self>) {
        if !matches!(id, InspectableId::CookieSettings) {
            return;
        }
        let metrics = &self.radix_theme.mode_tokens().metrics;
        let base = gpui_luma::controls::switch::SwitchScale::compute(self.control_size, metrics, 1.0);
        let current = self.overrides.effective_switch_scale(base);
        let value = match key {
            "track_width" => current.track_width + delta,
            "track_height" => current.track_height + delta,
            "thumb_size" => current.thumb_size + delta,
            _ => return,
        };
        self.overrides.set_scale(id, key.to_string(), value.max(8.0));
        self.demos.cookies.update(cx, |_, cx| cx.notify());
        cx.notify();
    }

    pub fn apply_overrides(&mut self, cx: &mut Context<Self>) {
        self.export_status = "Adjustments saved in session.".to_string();
        cx.notify();
    }

    pub fn export_theme(&mut self, cx: &mut Context<Self>) {
        match run_export(self) {
            Ok(path) => {
                self.export_status = format!("Exported to {}", path.display());
                tracing::info!("theme studio export: {}", path.display());
            }
            Err(err) => {
                self.export_status = format!("Export failed: {err}");
                tracing::error!("theme studio export failed: {err:?}");
            }
        }
        cx.notify();
    }
}

impl Render for ThemeStudioApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let sans = self.radix_theme.mode_tokens().typography.font.sans.family.clone();
        let active_mode = self.radix_theme.mode();
        let toggle_icon = match active_mode {
            ThemeMode::Light => LucideIcon::Moon,
            ThemeMode::Dark => LucideIcon::Sun,
        };
        let size = self.control_size;

        let title_bar = TitleBar::new().background_color(chrome.panel_background).border_color(chrome.border).child(
            div()
                .id("theme-studio-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .text_color(chrome.title_text)
                .font_family(sans.clone())
                .child(div().text_size(px(14.0)).font_weight(gpui::FontWeight::SEMIBOLD).child("Luma Theme Studio"))
                .child(
                    div().flex().items_center().gap(px(12.0)).child(render_size_toggle(size, chrome, cx)).child(
                        div()
                            .id("theme-studio-mode-toggle")
                            .size(px(28.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(6.0))
                            .font_family("lucide")
                            .text_size(px(14.0))
                            .line_height(px(14.0))
                            .text_color(chrome.title_text)
                            .cursor_pointer()
                            .hover(|style| style.bg(gpui::hsla(0.0, 0.0, 1.0, 0.10)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                let mode = match this.radix_theme.mode() {
                                    ThemeMode::Light => ThemeMode::Dark,
                                    ThemeMode::Dark => ThemeMode::Light,
                                };
                                this.radix_theme.set_mode(mode);
                                cx.notify();
                            }))
                            .child(char::from(toggle_icon).to_string()),
                    ),
                ),
        );

        let scale_factor = window.scale_factor();
        let inspector = render_inspector(self, scale_factor, cx);
        let board = render_demo_board(self.selected, &self.panel_positions, &self.demos, chrome, cx);

        let mut root = div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .font_family(sans)
            .bg(chrome.app_background)
            .child(title_bar)
            .child(
                div()
                    .id("theme-studio-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .justify_center()
                    .p(px(24.0))
                    .child(board),
            );

        if let Some(panel) = inspector {
            root = root.child(panel);
        }

        root
    }
}

fn render_size_toggle(
    active: ControlSize,
    chrome: gpui_luma::theme::LumaChrome,
    cx: &mut Context<ThemeStudioApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .child(div().text_size(px(11.0)).text_color(chrome.muted_text).child("Size:"))
        .children([ControlSize::Sm, ControlSize::Md, ControlSize::Lg].map(|size| {
            let selected = active == size;
            let label = match size {
                ControlSize::Sm => "SM",
                ControlSize::Md => "MD",
                ControlSize::Lg => "LG",
            };
            div()
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(5.0))
                .text_size(px(11.0))
                .text_color(if selected { chrome.title_text } else { chrome.muted_text })
                .bg(if selected {
                    gpui::hsla(0.0, 0.0, 1.0, 0.12)
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                })
                .cursor_pointer()
                .child(label)
                .on_mouse_down(MouseButton::Left, cx.listener(move |app, _, _, cx| app.set_control_size(size, cx)))
        }))
}
