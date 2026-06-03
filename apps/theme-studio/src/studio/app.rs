use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    Context, Div, DragMoveEvent, Entity, FocusHandle, MouseButton, MouseDownEvent, Overflow, Point, Pixels, Render,
    Size, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::resizable_panels::{ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation};
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
use super::studio_board::StudioBoardHost;
use super::theme_sidebar::ThemeSidebar;

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
    active_theme_id: String,
    theme_sidebar: Entity<ThemeSidebar>,
    board_host: Entity<StudioBoardHost>,
    main_split: Entity<ResizablePanels>,
    /// Suppresses hex field `Change` handlers while programmatically syncing sidebar values.
    syncing_sidebar_tokens: bool,
    _subscriptions: Vec<Subscription>,
}

impl ThemeStudioApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: StudioThemeChoice) -> Self {
        let app = cx.entity();
        let focus_scope = cx.focus_handle();
        let active_theme_id = theme_choice.id();
        let mode = ThemeMode::Dark;
        let radix_theme = Self::load_theme(&active_theme_id);
        radix_theme.set_mode(mode);
        let control_size = ControlSize::Md;
        let demos = DemoControls::spawn(cx, radix_theme.clone(), control_size);

        let overrides = StudioOverrides::default();
        let panel_positions = load_panel_positions();
        let theme_sidebar =
            cx.new(|cx| ThemeSidebar::new(app.clone(), radix_theme.clone(), active_theme_id.clone(), &overrides, cx));

        let board_host = cx.new(|_| StudioBoardHost::new(app.clone()));
        let board_host_for_split = board_host.clone();
        let sidebar_entity = theme_sidebar.clone();

        let main_split = radix_theme
            .resizable_panels("theme-studio-main-split")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .show_handle(true)
            .handle_size(px(8.0))
            .handle_grip(true)
            .show_border(false)
            .panels([
                ResizablePanelSpec::new_render(move || {
                    div()
                        .size_full()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(div().flex_1().min_h_0().w_full().child(sidebar_entity.clone()))
                        .into_any_element()
                })
                .default_size(28.0)
                .min_size(18.0)
                .max_size(45.0),
                ResizablePanelSpec::new_render(move || {
                    div()
                        .size_full()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(scrollable_panel().items_center().p(px(24.0)).child(board_host_for_split.clone()))
                        .into_any_element()
                })
                .default_size(72.0)
                .min_size(55.0)
                .max_size(82.0),
            ])
            .spawn(cx);

        let mut subscriptions = Vec::new();
        ThemeSidebar::wire_subscriptions(&theme_sidebar, cx, &mut subscriptions);
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
            panel_positions,
            panel_drag: None,
            selected: None,
            overrides,
            export_status: String::new(),
            last_window_size,
            active_theme_id,
            theme_sidebar,
            board_host,
            main_split,
            syncing_sidebar_tokens: false,
            _subscriptions: subscriptions,
        }
    }

    fn sync_sidebar(&mut self, sync_tokens: bool, cx: &mut Context<Self>) {
        let theme = self.radix_theme.clone();
        let overrides = self.overrides.clone();
        let active_theme_id = self.active_theme_id.clone();

        self.syncing_sidebar_tokens = sync_tokens;
        self.theme_sidebar.update(cx, |sidebar, cx| {
            if sync_tokens {
                sidebar.apply_theme_snapshot(theme, &overrides, cx);
            } else {
                sidebar.sync_global_overrides(&overrides, cx);
            }
            sidebar.sync_theme_selector(active_theme_id, cx);
        });
        self.syncing_sidebar_tokens = false;
    }

    fn load_theme(theme_id: &str) -> Arc<RadixTheme> {
        StudioThemeChoice::from_id(theme_id).radix_theme()
    }

    fn apply_theme_overrides(&mut self, cx: &mut Context<Self>) {
        let base = Self::load_theme(&self.active_theme_id);
        base.set_mode(self.radix_theme.mode());
        self.radix_theme = Arc::new(base.with_color_overrides(&self.overrides.global_color_overrides));
        self.refresh_demos(cx);
    }

    fn refresh_demos(&mut self, cx: &mut Context<Self>) {
        self.demos = DemoControls::spawn(cx, self.radix_theme.clone(), self.control_size);
        self.demos.subscribe(cx, &mut self._subscriptions);
    }

    pub fn change_theme(&mut self, theme_id: &str, cx: &mut Context<Self>) {
        if self.active_theme_id == theme_id {
            return;
        }
        self.active_theme_id = theme_id.to_string();
        self.apply_theme_overrides(cx);
        self.sync_sidebar(true, cx);
        cx.notify();
    }

    pub fn set_global_color(&mut self, token: &str, color: gpui::Hsla, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let css_name = token_css_name(token);
        if self.overrides.global_color_override(&css_name) == Some(color) {
            return;
        }

        self.overrides.set_global_color(css_name, color);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
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
        self.refresh_demos(cx);
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

use super::export::token_css_name;

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
                    .id("theme-studio-body")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(div().flex_1().min_h_0().size_full().child(self.main_split.clone())),
            );

        if let Some(panel) = inspector {
            root = root.child(panel);
        }

        root
    }
}

fn scrollable_panel() -> Div {
    let mut panel = div().size_full().flex().flex_col();
    panel.style().overflow.y = Some(Overflow::Scroll);
    panel
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
