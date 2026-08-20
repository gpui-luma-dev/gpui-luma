use std::sync::Arc;

use gpui::{Context, Entity, FocusHandle, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ControlIcon, ControlPresenter};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::resizable_panels::{PanelHideMode, ResizablePanelsEvent};
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::{ControlSize, LumaThemeSyncExt, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextRole};
use lucide_svg_static::Icon as LucideIcon;

use crate::theme::GraphVizThemeChoice;

use super::activity::load_sample_ride;
use super::content_pane::ContentPaneHost;
use super::controls::workbench_layout::{LEFT_SIDEBAR_PANEL_INDEX, WorkbenchLayout, WorkbenchSidebar};
use super::theme_sidebar::ThemeSidebar;

pub struct GraphVizApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    active_theme_id: String,
    theme_sidebar: Entity<ThemeSidebar>,
    content_pane: Entity<ContentPaneHost>,
    workbench: WorkbenchLayout,
    sidebar_toggle: IconButton,
    mode_toggle: IconButton,
    sidebar_hidden: bool,
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
        let workbench = WorkbenchLayout::new(
            "graph-viz",
            look.clone(),
            WorkbenchSidebar::new(move || left_sidebar_entity.clone().into_any_element())
                .width(px(360.0))
                .min(px(360.0))
                .max(px(460.0)),
            move || content_pane_entity.clone().into_any_element(),
            cx,
        );

        let sidebar_toggle = look
            .content_only_icon_button("graph-viz-sidebar-toggle", LucideIcon::PanelLeft)
            .size(ControlSize::Sm)
            .spawn(cx);
        let mode_toggle = look
            .content_only_icon_button("graph-viz-mode-toggle", toggle_mode_icon(look.mode()))
            .size(ControlSize::Sm)
            .spawn(cx);
        for (button, icon) in [(&sidebar_toggle, LucideIcon::PanelLeft), (&mode_toggle, toggle_mode_icon(look.mode()))]
        {
            button.update(cx, |button, cx| {
                button.set_presenter(titlebar_icon_presenter(ControlIcon::Lucide(icon), look.chrome().title_text), cx);
            });
        }

        let mut subscriptions = Vec::new();
        let theme_selector = theme_sidebar.read(cx).theme_selector();
        subscriptions.push(cx.subscribe(
            &theme_selector,
            |app, _, event: &gpui_luma::controls::selector::SelectorEvent, cx| {
                if let gpui_luma::controls::selector::SelectorEvent::Change { item_id, .. } = event {
                    app.change_theme(item_id.as_ref(), cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe(&workbench.panels(), |app, _, event: &ResizablePanelsEvent, cx| {
            app.handle_workbench_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&sidebar_toggle, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.toggle_sidebar(cx);
            }
        }));
        subscriptions.push(cx.subscribe(&mode_toggle, |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.toggle_mode(cx);
            }
        }));

        Self {
            focus_scope,
            look,
            active_theme_id,
            theme_sidebar,
            content_pane,
            workbench,
            sidebar_toggle,
            mode_toggle,
            sidebar_hidden: false,
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
        let theme = self.look.resizable_panels_theme();
        self.workbench.sync_theme(&theme, cx);
    }

    fn handle_workbench_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<Self>) {
        if let ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden } = event
            && *panel_index == LEFT_SIDEBAR_PANEL_INDEX
        {
            self.sidebar_hidden = *hidden;
            let icon = if *hidden {
                LucideIcon::PanelLeftOpen
            } else {
                LucideIcon::PanelLeft
            };
            self.sidebar_toggle.update(cx, |button, cx| {
                button.set_presenter(
                    titlebar_icon_presenter(ControlIcon::Lucide(icon), self.look.chrome().title_text),
                    cx,
                );
            });
        }
        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.workbench.panels().update(cx, |panels, cx| {
            panels.toggle_panel_hidden(LEFT_SIDEBAR_PANEL_INDEX, PanelHideMode::Completely, cx);
        });
    }

    fn toggle_mode(&mut self, cx: &mut Context<Self>) {
        let mode = match self.look.mode() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.look.set_mode(mode);
        self.mode_toggle.update(cx, |button, cx| {
            button.set_presenter(
                titlebar_icon_presenter(ControlIcon::Lucide(toggle_mode_icon(mode)), self.look.chrome().title_text),
                cx,
            );
        });
        cx.bump_luma_theme_revision();
        let look = self.look.clone();
        self.theme_sidebar.update(cx, |sidebar, cx| {
            sidebar.apply_theme_snapshot(look, cx);
        });
        self.sync_split_themes(cx);
        self.content_pane.update(cx, |pane, cx| {
            pane.sync_look(self.look.clone(), cx);
            pane.notify_tabs(cx);
        });
        cx.notify();
    }
}

fn toggle_mode_icon(mode: ThemeMode) -> LucideIcon {
    match mode {
        ThemeMode::Light => LucideIcon::Moon,
        ThemeMode::Dark => LucideIcon::Sun,
    }
}

fn titlebar_icon_presenter(icon: ControlIcon, color: gpui::Hsla) -> ControlPresenter<ButtonRenderModel<()>> {
    Arc::new(move |_, _| match &icon {
        ControlIcon::Lucide(lucide) => {
            div().child(gpui_luma::controls::icon::lucide_icon(*lucide, color, 14.0)).into_any_element()
        }
        ControlIcon::SvgPath(path) => gpui::svg().size(px(14.0)).path(path.clone()).into_any_element(),
    })
}

impl Render for GraphVizApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sans = self.look.mode_tokens().typography.font.sans.family.clone();
        let title_style = self.look.typography_role(ShadcnTextRole::H4);

        let main_shell = self.workbench.render_body();

        let title_bar = TitleBar::new()
            .background_color(chrome.panel_background)
            .border_color(chrome.border)
            .text_color(chrome.title_text)
            .child(
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
                            .gap(px(6.0))
                            .child(self.sidebar_toggle.clone())
                            .child(self.mode_toggle.clone()),
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
