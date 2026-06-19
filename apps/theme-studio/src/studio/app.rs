use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    AnyElement, Context, DragMoveEvent, Entity, FocusHandle, MouseButton, MouseDownEvent, Point, Pixels, Render, Size,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::{ControlSize, LumaThemeSyncExt, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use crate::theme::StudioThemeChoice;

use super::controls::workbench_layout::{WorkbenchLayout, WorkbenchSidebar};
use super::demo_controls::DemoControls;
use super::inspectable::InspectableId;
use super::overrides::{
    StudioOverrides, ThemePaletteHslOverride, ThemeShadowOverride, clamp_palette_hue_deg,
    clamp_palette_lightness_multiplier, clamp_palette_saturation_multiplier, clamp_shadow_blur, clamp_shadow_offset_x,
    clamp_shadow_offset_y, clamp_shadow_opacity, clamp_shadow_spread, default_shadow_override,
};
use super::panel_layout::{DemoPanelDrag, default_panel_position, offset_panel_position};
use super::panel_layout_config::{load_panel_positions, load_window_size, save_studio_layout};
use super::content::{BoardSnapshot, ContentPaneHost};
use super::theme_sidebar::{ThemeSidebar, palette_tokens};

struct PanelDragState {
    id: InspectableId,
    mouse_origin: Point<Pixels>,
    panel_origin: Point<Pixels>,
}

pub struct ThemeStudioApp {
    focus_scope: FocusHandle,
    pub(super) look: Arc<ShadcnLook>,
    pub(super) control_size: ControlSize,
    pub(super) demos: DemoControls,
    pub(super) panel_positions: HashMap<InspectableId, Point<Pixels>>,
    pub(super) panel_z_order: HashMap<InspectableId, u32>,
    next_panel_z: u32,
    panel_drag: Option<PanelDragState>,
    pub(super) selected: Option<InspectableId>,
    pub(super) overrides: StudioOverrides,
    last_window_size: Size<Pixels>,
    active_theme_id: String,
    theme_sidebar: Entity<ThemeSidebar>,
    content_pane: Entity<ContentPaneHost>,
    workbench: WorkbenchLayout,
    sidebar_collapsed: bool,
    right_sidebar_collapsed: bool,
    /// Suppresses sidebar `Change` handlers while programmatically syncing sidebar values.
    syncing_sidebar_tokens: bool,
    _subscriptions: Vec<Subscription>,
}

impl ThemeStudioApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, theme_choice: StudioThemeChoice) -> Self {
        let app = cx.entity();
        let focus_scope = cx.focus_handle();
        let active_theme_id = theme_choice.id();
        let mode = ThemeMode::Dark;
        let look = Self::load_theme(&active_theme_id);
        look.set_mode(mode);
        let control_size = ControlSize::Md;
        let demos = DemoControls::spawn(cx, look.clone(), control_size);

        let overrides = StudioOverrides::default();
        let panel_positions = load_panel_positions();
        let panel_z_order: HashMap<InspectableId, u32> =
            InspectableId::all().iter().enumerate().map(|(index, &id)| (id, index as u32)).collect();
        let next_panel_z = InspectableId::all().len() as u32;
        let theme_sidebar =
            cx.new(|cx| ThemeSidebar::new(app.clone(), look.clone(), active_theme_id.clone(), &overrides, cx));

        let board_snapshot = BoardSnapshot {
            selected: None,
            panel_positions: panel_positions.clone(),
            panel_z_order: panel_z_order.clone(),
            demos: demos.clone(),
            look: look.clone(),
            overrides: overrides.clone(),
        };
        let content_pane = cx.new(|cx| ContentPaneHost::new(app.clone(), board_snapshot, cx));
        let left_sidebar_entity = theme_sidebar.clone();
        let content_pane_entity = content_pane.clone();
        let right_sidebar_look = look.clone();
        let workbench = WorkbenchLayout::new(
            "theme-studio",
            look.clone(),
            WorkbenchSidebar::new(move || left_sidebar_entity.clone().into_any_element())
                .width(px(360.0))
                .min(px(360.0))
                .max(px(460.0)),
            move || content_pane_entity.clone().into_any_element(),
            WorkbenchSidebar::new(move || render_right_sidebar(right_sidebar_look.clone()))
                .width(px(360.0))
                .min(px(360.0))
                .max(px(460.0)),
            cx,
        );

        let mut subscriptions = Vec::new();
        ThemeSidebar::wire_subscriptions(&theme_sidebar, cx, &mut subscriptions);
        demos.subscribe(cx, &mut subscriptions);
        subscriptions.push(cx.observe_window_bounds(window, |this, window, cx| {
            this.on_window_bounds_changed(window, cx);
        }));

        let last_window_size = load_window_size();

        Self {
            focus_scope,
            look,
            control_size,
            demos,
            panel_positions,
            panel_z_order,
            next_panel_z,
            panel_drag: None,
            selected: None,
            overrides,
            last_window_size,
            active_theme_id,
            theme_sidebar,
            content_pane,
            workbench,
            sidebar_collapsed: false,
            right_sidebar_collapsed: true,
            syncing_sidebar_tokens: false,
            _subscriptions: subscriptions,
        }
    }

    fn sync_sidebar(&mut self, sync_tokens: bool, cx: &mut Context<Self>) {
        let theme = self.look.clone();
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

    fn load_theme(theme_id: &str) -> Arc<ShadcnLook> {
        StudioThemeChoice::from_id(theme_id).shadcn_look()
    }

    fn apply_theme_overrides(&mut self, cx: &mut Context<Self>) {
        let base = Self::load_theme(&self.active_theme_id);
        base.set_mode(self.look.mode());
        let (light_palette_color_overrides, dark_palette_color_overrides) =
            self.derived_palette_color_overrides(base.as_ref());
        self.look.replace_theme(base.as_ref());
        if let Err(err) =
            self.look.apply_mode_color_overrides(&light_palette_color_overrides, &dark_palette_color_overrides)
        {
            tracing::warn!("failed to apply studio color overrides: {err:?}");
        }
        let token_overrides = self.overrides.token_overrides();
        if let Err(err) = self.look.apply_token_overrides(&token_overrides) {
            tracing::warn!("failed to apply studio token overrides: {err:?}");
        }
        cx.bump_luma_theme_revision();
        self.sync_split_themes(cx);
        self.refresh_content_pane(cx);
    }

    fn derived_palette_color_overrides(
        &self,
        base: &ShadcnLook,
    ) -> (HashMap<String, gpui::Hsla>, HashMap<String, gpui::Hsla>) {
        let derive_for_mode = |mode: ThemeMode| {
            let palette_hsl = self.overrides.palette_hsl(mode).clone();
            let mode_tokens = match mode {
                ThemeMode::Light => base.light_tokens(),
                ThemeMode::Dark => base.dark_tokens(),
            };

            palette_tokens()
                .into_iter()
                .filter_map(|token| {
                    let css_name = token_css_name(token);
                    self.overrides
                        .global_color_override(&css_name)
                        .or_else(|| mode_tokens.catalog.color(token).ok())
                        .map(|color| (css_name, palette_hsl.apply(color)))
                })
                .collect::<HashMap<_, _>>()
        };

        (derive_for_mode(ThemeMode::Light), derive_for_mode(ThemeMode::Dark))
    }

    fn board_snapshot(&self) -> BoardSnapshot {
        BoardSnapshot {
            selected: self.selected,
            panel_positions: self.panel_positions.clone(),
            panel_z_order: self.panel_z_order.clone(),
            demos: self.demos.clone(),
            look: self.look.clone(),
            overrides: self.overrides.clone(),
        }
    }

    fn refresh_content_pane(&self, cx: &mut Context<Self>) {
        let board = self.board_snapshot();
        self.content_pane.update(cx, |pane, cx| {
            pane.sync_board_snapshot(board, cx);
            pane.notify_tabs(cx);
        });
    }

    fn sync_split_themes(&self, cx: &mut Context<Self>) {
        let theme = self.look.resizable_panels_theme();
        self.workbench.sync_theme(theme, cx);
    }

    fn sync_split_measured_sizes(&self, size: Size<Pixels>, cx: &mut Context<Self>) {
        self.workbench.sync_measured_size(size, cx);
    }

    pub fn change_theme(&mut self, theme_id: &str, cx: &mut Context<Self>) {
        if self.active_theme_id == theme_id {
            return;
        }
        self.active_theme_id = theme_id.to_string();
        self.overrides.clear_metric_overrides();
        self.overrides.clear_shadow_override();
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

    pub fn set_palette_hue_deg(&mut self, hue_deg: f32, cx: &mut Context<Self>) {
        self.update_palette_hsl_override(|palette_hsl| palette_hsl.hue_deg = clamp_palette_hue_deg(hue_deg), cx);
    }

    pub fn set_palette_saturation_multiplier(&mut self, multiplier: f32, cx: &mut Context<Self>) {
        self.update_palette_hsl_override(
            |palette_hsl| palette_hsl.saturation_multiplier = clamp_palette_saturation_multiplier(multiplier),
            cx,
        );
    }

    pub fn set_palette_lightness_multiplier(&mut self, multiplier: f32, cx: &mut Context<Self>) {
        self.update_palette_hsl_override(
            |palette_hsl| palette_hsl.lightness_multiplier = clamp_palette_lightness_multiplier(multiplier),
            cx,
        );
    }

    pub fn set_radius_rem(&mut self, rem: f32, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let rem = crate::studio::overrides::clamp_radius_rem(rem);
        if self.overrides.radius_rem() == Some(rem) {
            return;
        }

        self.overrides.set_radius_rem(rem);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    pub fn set_spacing_rem(&mut self, rem: f32, cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let rem = crate::studio::overrides::clamp_spacing_rem(rem);
        if self.overrides.spacing_rem() == Some(rem) {
            return;
        }

        self.overrides.set_spacing_rem(rem);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    pub fn reload_active_theme(&mut self, cx: &mut Context<Self>) {
        self.overrides.clear_all_overrides();
        self.apply_theme_overrides(cx);
        self.sync_sidebar(true, cx);
        cx.notify();
    }

    pub fn set_shadow_color(&mut self, color: gpui::Hsla, cx: &mut Context<Self>) {
        self.update_shadow_override(
            |shadow| {
                shadow.color.h = color.h;
                shadow.color.s = color.s;
                shadow.color.l = color.l;
            },
            cx,
        );
    }

    pub fn set_shadow_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.set_opacity(clamp_shadow_opacity(opacity)), cx);
    }

    pub fn set_shadow_blur(&mut self, blur_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.blur_px = clamp_shadow_blur(blur_px), cx);
    }

    pub fn set_shadow_spread(&mut self, spread_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.spread_px = clamp_shadow_spread(spread_px), cx);
    }

    pub fn set_shadow_offset_x(&mut self, offset_x_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.offset_x_px = clamp_shadow_offset_x(offset_x_px), cx);
    }

    pub fn set_shadow_offset_y(&mut self, offset_y_px: f32, cx: &mut Context<Self>) {
        self.update_shadow_override(|shadow| shadow.offset_y_px = clamp_shadow_offset_y(offset_y_px), cx);
    }

    fn update_palette_hsl_override(
        &mut self,
        update: impl FnOnce(&mut ThemePaletteHslOverride),
        cx: &mut Context<Self>,
    ) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let mode = self.look.mode();
        let mut palette_hsl = self.overrides.palette_hsl(mode).clone();
        update(&mut palette_hsl);

        if self.overrides.palette_hsl(mode) == &palette_hsl {
            return;
        }

        self.overrides.set_palette_hsl_override(mode, palette_hsl);
        self.apply_theme_overrides(cx);
        self.syncing_sidebar_tokens = true;
        let overrides = self.overrides.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| sidebar.sync_global_overrides(&overrides, cx));
        self.syncing_sidebar_tokens = false;
        cx.notify();
    }

    fn update_shadow_override(&mut self, update: impl FnOnce(&mut ThemeShadowOverride), cx: &mut Context<Self>) {
        if self.syncing_sidebar_tokens {
            return;
        }

        let mut shadow =
            self.overrides.shadow_override().cloned().unwrap_or_else(|| default_shadow_override(&self.look));
        update(&mut shadow);

        if self.overrides.shadow_override() == Some(&shadow) {
            return;
        }

        self.overrides.set_shadow_override(shadow);
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
        self.sync_split_measured_sizes(size, cx);
        self.persist_layout();
    }

    fn persist_layout(&self) {
        if let Err(err) = save_studio_layout(self.last_window_size, &self.panel_positions) {
            tracing::warn!("failed to save studio layout: {err:?}");
        }
    }

    pub fn panel_position(&self, id: InspectableId) -> Point<Pixels> {
        self.panel_positions.get(&id).copied().unwrap_or_else(|| default_panel_position(id))
    }

    pub fn begin_panel_drag(&mut self, id: InspectableId, event: &MouseDownEvent, cx: &mut Context<Self>) {
        if self.selected != Some(id) {
            self.selected = Some(id);
        }
        self.bring_panel_to_front(id);
        self.panel_drag =
            Some(PanelDragState { id, mouse_origin: event.position, panel_origin: self.panel_position(id) });
        cx.notify();
    }

    fn bring_panel_to_front(&mut self, id: InspectableId) {
        let z = self.next_panel_z;
        self.next_panel_z += 1;
        self.panel_z_order.insert(id, z);
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

    pub fn set_control_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.control_size == size {
            return;
        }
        self.control_size = size;
        self.demos = DemoControls::spawn(cx, self.look.clone(), self.control_size);
        self.refresh_content_pane(cx);
        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sync_split_measured_sizes(self.last_window_size, cx);
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }

    fn toggle_right_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sync_split_measured_sizes(self.last_window_size, cx);
        self.right_sidebar_collapsed = !self.right_sidebar_collapsed;
        cx.notify();
    }
}

use super::export::token_css_name;

impl Render for ThemeStudioApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sans = self.look.mode_tokens().typography.font.sans.family.clone();
        let active_mode = self.look.mode();
        let toggle_icon = match active_mode {
            ThemeMode::Light => LucideIcon::Moon,
            ThemeMode::Dark => LucideIcon::Sun,
        };
        let size = self.control_size;
        let title_style = self.look.typography_role(ShadcnTextRole::H4);
        let toggle_label_style = self.look.typography_scale(ShadcnTextSize::Xs);
        let sidebar_toggle_icon = if self.sidebar_collapsed {
            LucideIcon::PanelLeftOpen
        } else {
            LucideIcon::PanelLeft
        };
        let right_sidebar_toggle_icon = if self.right_sidebar_collapsed {
            LucideIcon::PanelRightOpen
        } else {
            LucideIcon::PanelRight
        };

        let main_shell = self.workbench.render_body(self.sidebar_collapsed, self.right_sidebar_collapsed);

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
                .child(div().typography_style(title_style).child("Luma Theme Studio"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(render_size_toggle(size, chrome, toggle_label_style, cx))
                        .child(
                            div()
                                .id("theme-studio-sidebar-toggle")
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
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_sidebar(cx);
                                }))
                                .child(char::from(sidebar_toggle_icon).to_string()),
                        )
                        .child(
                            div()
                                .id("theme-studio-right-sidebar-toggle")
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
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_right_sidebar(cx);
                                }))
                                .child(char::from(right_sidebar_toggle_icon).to_string()),
                        )
                        .child(
                            div()
                                .id("theme-studio-reset-theme")
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
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.reload_active_theme(cx);
                                }))
                                .child(char::from(LucideIcon::RefreshCcw).to_string()),
                        )
                        .child(
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
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let mode = match this.look.mode() {
                                        ThemeMode::Light => ThemeMode::Dark,
                                        ThemeMode::Dark => ThemeMode::Light,
                                    };
                                    this.look.set_mode(mode);
                                    cx.bump_luma_theme_revision();
                                    let theme = this.look.clone();
                                    let overrides = this.overrides.clone();
                                    this.theme_sidebar.update(cx, |sidebar, cx| {
                                        sidebar.apply_theme_snapshot(theme, &overrides, cx);
                                    });
                                    this.sync_split_themes(cx);
                                    this.refresh_content_pane(cx);
                                    cx.notify();
                                }))
                                .child(char::from(toggle_icon).to_string()),
                        ),
                ),
        );

        self.workbench.render_shell(
            &self.focus_scope,
            sans.into(),
            chrome.app_background,
            title_bar,
            main_shell,
            div()
                .id("theme-studio-bottom-app-bar")
                .h(px(32.0))
                .w_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .bg(chrome.panel_background)
                .border_t_1()
                .border_color(chrome.border),
        )
    }
}

fn render_right_sidebar(look: Arc<ShadcnLook>) -> AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .id("theme-studio-right-sidebar")
        .size_full()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(chrome.panel_background)
        .border_l_1()
        .border_color(chrome.border)
        .child(
            div()
                .id("theme-studio-right-sidebar-header")
                .w_full()
                .h(px(48.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .px(px(16.0))
                .border_b_1()
                .border_color(chrome.border)
                .child(div().typography_style(title_style).text_color(chrome.title_text).child("Inspector")),
        )
        .child(div().flex_1().min_h_0().w_full())
        .into_any_element()
}

fn render_size_toggle(
    active: ControlSize,
    chrome: gpui_luma::theme::LumaChrome,
    label_style: gpui_luma::theme::LumaTextStyle,
    cx: &mut Context<ThemeStudioApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("Size:"))
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
                .typography_style(label_style)
                .text_color(if selected { chrome.title_text } else { chrome.muted_text })
                .bg(if selected {
                    gpui::hsla(0.0, 0.0, 1.0, 0.12)
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                })
                .cursor_pointer()
                .child(label)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |app, _, _, cx| {
                        cx.stop_propagation();
                        app.set_control_size(size, cx);
                    }),
                )
        }))
}
