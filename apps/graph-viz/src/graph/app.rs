use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{Context, Entity, FocusHandle, MouseButton, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::{LumaThemeSyncExt, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole};
use lucide_icons::Icon as LucideIcon;

use crate::theme::GraphVizThemeChoice;

use super::activity::load_sample_ride;
use super::content_pane::ContentPaneHost;
use super::controls::workbench_layout::{WorkbenchLayout, WorkbenchSidebar};
use super::theme_sidebar::ThemeSidebar;

pub struct GraphVizApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    active_theme_id: String,
    theme_sidebar: Entity<ThemeSidebar>,
    content_pane: Entity<ContentPaneHost>,
    workbench: WorkbenchLayout,
    _left_sidebar_width_px: Rc<Cell<f32>>,
    sidebar_collapsed: bool,
    _subscriptions: Vec<Subscription>,
}

impl GraphVizApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, theme_choice: GraphVizThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let active_theme_id = theme_choice.id();
        let look = Self::load_theme(&active_theme_id);
        look.set_mode(ThemeMode::Dark);

        let theme_sidebar = cx.new(|cx| ThemeSidebar::new(look.clone(), active_theme_id.clone(), cx));
        let ride = load_sample_ride().expect("bundled sample FIT ride should parse");
        let content_pane = cx.new(|cx| ContentPaneHost::new(look.clone(), ride, cx));

        let left_sidebar_entity = theme_sidebar.clone();
        let content_pane_entity = content_pane.clone();
        let left_sidebar_width_px = Rc::new(Cell::new(360.0));
        let left_sidebar_width_state = left_sidebar_width_px.clone();
        let workbench = WorkbenchLayout::new(
            "graph-viz",
            look.clone(),
            WorkbenchSidebar::new(move || left_sidebar_entity.clone().into_any_element())
                .width(px(360.0))
                .min(px(360.0))
                .max(px(460.0))
                .on_width_changed(move |width| left_sidebar_width_state.set(width.as_f32())),
            move || content_pane_entity.clone().into_any_element(),
            WorkbenchSidebar::new(|| div().into_any_element()),
            cx,
        );

        let mut subscriptions = Vec::new();
        let theme_selector = theme_sidebar.read(cx).theme_selector();
        subscriptions.push(cx.subscribe(
            &theme_selector,
            |app, _, event: &gpui_luma::controls::selector::SelectorEvent, cx| {
                let gpui_luma::controls::selector::SelectorEvent::Change { item_id, .. } = event;
                app.change_theme(item_id.as_ref(), cx);
            },
        ));

        Self {
            focus_scope,
            look,
            active_theme_id,
            theme_sidebar,
            content_pane,
            workbench,
            _left_sidebar_width_px: left_sidebar_width_px,
            sidebar_collapsed: false,
            _subscriptions: subscriptions,
        }
    }

    fn load_theme(theme_id: &str) -> Arc<ShadcnLook> {
        GraphVizThemeChoice::from_id(theme_id).shadcn_look()
    }

    pub fn change_theme(&mut self, theme_id: &str, cx: &mut Context<Self>) {
        if self.active_theme_id == theme_id {
            return;
        }

        self.active_theme_id = theme_id.to_string();
        let base = Self::load_theme(&self.active_theme_id);
        base.set_mode(self.look.mode());
        self.look.replace_theme(base.as_ref());
        cx.bump_luma_theme_revision();
        self.sync_split_themes(cx);

        let look = self.look.clone();
        let active_theme_id = self.active_theme_id.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| {
            sidebar.apply_theme_snapshot(look, cx);
            sidebar.sync_theme_selector(active_theme_id, cx);
        });
        self.content_pane.update(cx, |pane, cx| {
            pane.sync_look(self.look.clone(), cx);
            pane.notify_tabs(cx);
        });
        cx.notify();
    }

    fn sync_split_themes(&self, cx: &mut Context<Self>) {
        let theme = self.look.dock_splitter_theme();
        self.workbench.sync_theme(&theme, cx);
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }
}

impl Render for GraphVizApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sans = self.look.mode_tokens().typography.font.sans.family.clone();
        let active_mode = self.look.mode();
        let toggle_icon = match active_mode {
            ThemeMode::Light => LucideIcon::Moon,
            ThemeMode::Dark => LucideIcon::Sun,
        };
        let title_style = self.look.typography_role(ShadcnTextRole::H4);
        let sidebar_toggle_icon = if self.sidebar_collapsed {
            LucideIcon::PanelLeftOpen
        } else {
            LucideIcon::PanelLeft
        };

        let main_shell = self.workbench.render_body(self.sidebar_collapsed, true);

        let title_bar = TitleBar::new().background_color(chrome.panel_background).border_color(chrome.border).child(
            div()
                .id("graph-viz-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .text_color(chrome.title_text)
                .font_family(sans.clone())
                .child(div().typography_style(title_style).child("Graph Viz"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(
                            div()
                                .id("graph-viz-sidebar-toggle")
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
                                .id("graph-viz-mode-toggle")
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
                                    let look = this.look.clone();
                                    this.theme_sidebar.update(cx, |sidebar, cx| {
                                        sidebar.apply_theme_snapshot(look, cx);
                                    });
                                    this.sync_split_themes(cx);
                                    this.content_pane.update(cx, |pane, cx| {
                                        pane.sync_look(this.look.clone(), cx);
                                        pane.notify_tabs(cx);
                                    });
                                    cx.notify();
                                }))
                                .child(char::from(toggle_icon).to_string()),
                        ),
                ),
        );

        WorkbenchLayout::render_shell(
            &self.focus_scope,
            sans.into(),
            chrome.app_background,
            title_bar,
            main_shell,
            div()
                .id("graph-viz-bottom-app-bar")
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
