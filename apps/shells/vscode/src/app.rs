use std::sync::Arc;

use gpui::{Context, Entity, FocusHandle, Focusable, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ControlIcon, ControlPresenter};
use gpui_luma::controls::control_group::ControlGroupEvent;
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::overlay_window::OverlayWindowEvent;
use gpui_luma::controls::resizable_panels::ResizablePanelsEvent;
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::{TITLE_BAR_HEIGHT, TitleBar};
use gpui_luma::theme::{ControlSize, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextRole};
use gpui_luma_shell_common::{
    chrome::{HasShellTheme, handle_theme_toggle, spawn_theme_toggle_button},
    theme::{ShellThemeChoice, sync_color_control_theme},
};
use lucide_icons::Icon as LucideIcon;

gpui::actions!(vscode_layout, [TogglePrimarySideBar, TogglePanel]);

use crate::customize_layout_dialog::CustomizeLayoutDialog;
use crate::layout_config::{LayoutConfig, LayoutRegion, PanelAlignment, PrimarySideBarPosition};
use crate::workspace_layout::{PrimarySideBar, SecondarySideBar, WorkspaceLayout, WorkspaceLayoutEvent};

const STATUS_BAR_H: f32 = 32.0;

pub struct VscodeShellApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    layout_config: LayoutConfig,
    workspace_layout: Entity<WorkspaceLayout>,
    customize_layout_toggle: IconButton,
    primary_side_bar_toggle: IconButton,
    secondary_side_bar_toggle: IconButton,
    theme_toggle_button: IconButton,
    customize_layout_dialog: Entity<CustomizeLayoutDialog>,
    applying_layout_config: bool,
    _subscriptions: Vec<Subscription>,
}

impl VscodeShellApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let layout_config = LayoutConfig::default();

        let workspace_layout = cx.new(|cx| {
            WorkspaceLayout::new(
                "shell-vscode",
                look.clone(),
                PrimarySideBar::new("primary-side-bar", look.clone()).width(px(360.0)).min(px(240.0)).max(px(460.0)),
                SecondarySideBar::new("secondary-side-bar", look.clone())
                    .width(px(360.0))
                    .min(px(240.0))
                    .max(px(460.0)),
                cx,
            )
        });

        let customize_layout_toggle = look
            .content_only_icon_button("shell-vscode-customize-layout-toggle", LucideIcon::LayoutPanelLeft)
            .size(ControlSize::Sm)
            .spawn(cx);
        let primary_side_bar_toggle = look
            .content_only_icon_button("shell-vscode-primary-side-bar-toggle", LucideIcon::PanelLeft)
            .size(ControlSize::Sm)
            .spawn(cx);
        let secondary_side_bar_toggle = look
            .content_only_icon_button("shell-vscode-secondary-side-bar-toggle", LucideIcon::PanelRight)
            .size(ControlSize::Sm)
            .spawn(cx);
        let theme_toggle_button = spawn_theme_toggle_button("shell-vscode-theme-toggle", &look, cx);

        let customize_layout_dialog = cx.new(|cx| CustomizeLayoutDialog::new(look.clone(), cx));

        let mut subscriptions = Vec::new();
        wire_customize_layout_subscriptions(&customize_layout_dialog, cx, &mut subscriptions);

        let workspace_panels = workspace_layout.read(cx).panels();
        subscriptions.push(cx.subscribe(&workspace_panels, |this, _, event: &ResizablePanelsEvent, cx| {
            this.handle_workbench_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&workspace_layout, |this, _, event: &WorkspaceLayoutEvent, cx| {
            let WorkspaceLayoutEvent::PanelHeightChanged { height_px } = event;
            this.layout_config.panel_height_px = *height_px;
            this.refresh_customize_layout_dialog(cx);
        }));
        subscriptions.push(cx.subscribe(&customize_layout_toggle, |this, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.open_customize_layout(cx);
            }
        }));
        subscriptions.push(cx.subscribe(&primary_side_bar_toggle, |this, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.toggle_primary_side_bar(cx);
            }
        }));
        subscriptions.push(cx.subscribe(&secondary_side_bar_toggle, |this, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.toggle_secondary_side_bar(cx);
            }
        }));
        subscriptions.push(cx.subscribe(&theme_toggle_button, |this, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                handle_theme_toggle(this, cx);
                this.sync_workbench_theme(cx);
            }
        }));

        let mut app = Self {
            focus_scope,
            look,
            layout_config,
            workspace_layout,
            customize_layout_toggle,
            primary_side_bar_toggle,
            secondary_side_bar_toggle,
            theme_toggle_button,
            customize_layout_dialog,
            applying_layout_config: false,
            _subscriptions: subscriptions,
        };
        app.sync_workbench_theme(cx);
        app.apply_layout_config(cx);
        app
    }

    pub fn open_customize_layout(&mut self, cx: &mut Context<Self>) {
        self.customize_layout_dialog.update(cx, |dialog, cx| {
            dialog.sync_from_config(&self.layout_config, cx);
        });
        let focus = self.customize_layout_toggle.read(cx).focus_handle(cx);
        self.customize_layout_dialog.update(cx, |dialog, cx| dialog.open(Some(focus), cx));
    }

    pub fn dismiss_customize_layout(&mut self, cx: &mut Context<Self>) {
        self.customize_layout_dialog.update(cx, |dialog, cx| dialog.dismiss(cx));
    }

    pub fn reset_layout_config(&mut self, cx: &mut Context<Self>) {
        self.layout_config = LayoutConfig::default();
        self.customize_layout_dialog.update(cx, |dialog, cx| {
            dialog.sync_from_config(&self.layout_config, cx);
        });
        self.apply_layout_config(cx);
    }

    pub fn set_layout_region_visible(&mut self, region: LayoutRegion, visible: bool, cx: &mut Context<Self>) {
        if self.layout_config.region_visible(region) == visible {
            return;
        }
        self.layout_config.set_region_visible(region, visible);
        self.workspace_layout.update(cx, |layout, cx| match region {
            LayoutRegion::ActivityBar => layout.set_activity_bar_visible(visible, cx),
            LayoutRegion::SecondaryActivityBar => layout.set_secondary_activity_bar_visible(visible, cx),
            LayoutRegion::PrimarySideBar => layout.set_primary_side_bar_visible(visible, cx),
            LayoutRegion::SecondarySideBar => layout.set_secondary_side_bar_visible(visible, cx),
            LayoutRegion::Panel => layout.set_panel_visible(visible, cx),
            LayoutRegion::StatusBar => layout.set_status_bar_visible(visible, cx),
        });
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    pub fn set_primary_side_bar_position(&mut self, position: PrimarySideBarPosition, cx: &mut Context<Self>) {
        if self.layout_config.primary_side_bar_position == position {
            return;
        }
        self.layout_config.primary_side_bar_position = position;
        self.apply_layout_config(cx);
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    pub fn set_panel_alignment(&mut self, alignment: PanelAlignment, cx: &mut Context<Self>) {
        if self.layout_config.panel_alignment == alignment {
            return;
        }
        self.layout_config.panel_alignment = alignment;
        self.apply_layout_config(cx);
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    fn sync_workbench_theme(&self, cx: &mut Context<Self>) {
        let theme = self.look.resizable_panels_theme();
        self.workspace_layout.update(cx, |layout, cx| {
            layout.sync_chrome(cx);
            layout.sync_theme(&theme, cx);
        });
    }

    fn refresh_customize_layout_dialog(&self, cx: &mut Context<Self>) {
        self.customize_layout_dialog.update(cx, |dialog, cx| {
            dialog.sync_from_config(&self.layout_config, cx);
        });
    }

    fn apply_layout_config(&mut self, cx: &mut Context<Self>) {
        self.applying_layout_config = true;
        self.workspace_layout.update(cx, |layout, cx| {
            layout.set_activity_bar_visible(self.layout_config.activity_bar_visible, cx);
            layout.set_secondary_activity_bar_visible(self.layout_config.secondary_activity_bar_visible, cx);
            layout.set_primary_side_bar_visible(self.layout_config.primary_side_bar_visible, cx);
            layout.set_secondary_side_bar_visible(self.layout_config.secondary_side_bar_visible, cx);
            layout.set_panel_visible(self.layout_config.panel_visible, cx);
            layout.set_status_bar_visible(self.layout_config.status_bar_visible, cx);
            layout.set_primary_side_bar_position(self.layout_config.primary_side_bar_position, cx);
            layout.set_panel_alignment(self.layout_config.panel_alignment, cx);
            layout.set_panel_height(self.layout_config.panel_height_px, cx);
        });
        self.applying_layout_config = false;
        self.sync_side_bar_toggle_icons(cx);
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    fn handle_workbench_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<Self>) {
        if self.applying_layout_config {
            return;
        }
        if let ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden } = event {
            let primary_index = self.workspace_layout.read(cx).primary_side_bar_panel_index();
            let secondary_index = self.workspace_layout.read(cx).secondary_side_bar_panel_index();
            if *panel_index == primary_index {
                self.layout_config.primary_side_bar_visible = !hidden;
            } else if *panel_index == secondary_index {
                self.layout_config.secondary_side_bar_visible = !hidden;
            }
            self.sync_side_bar_toggle_icons(cx);
            self.refresh_customize_layout_dialog(cx);
        }
        cx.notify();
    }

    fn toggle_primary_side_bar(&mut self, cx: &mut Context<Self>) {
        let visible = !self.layout_config.primary_side_bar_visible;
        self.set_layout_region_visible(LayoutRegion::PrimarySideBar, visible, cx);
    }

    fn toggle_secondary_side_bar(&mut self, cx: &mut Context<Self>) {
        let visible = !self.layout_config.secondary_side_bar_visible;
        self.set_layout_region_visible(LayoutRegion::SecondarySideBar, visible, cx);
    }

    fn sync_side_bar_toggle_icons(&mut self, cx: &mut Context<Self>) {
        let primary_icon =
            match (self.layout_config.primary_side_bar_position, self.layout_config.primary_side_bar_visible) {
                (PrimarySideBarPosition::Left, true) => LucideIcon::PanelLeft,
                (PrimarySideBarPosition::Left, false) => LucideIcon::PanelLeftOpen,
                (PrimarySideBarPosition::Right, true) => LucideIcon::PanelRight,
                (PrimarySideBarPosition::Right, false) => LucideIcon::PanelRightOpen,
            };
        let secondary_icon =
            match (self.layout_config.primary_side_bar_position, self.layout_config.secondary_side_bar_visible) {
                (PrimarySideBarPosition::Left, true) => LucideIcon::PanelRight,
                (PrimarySideBarPosition::Left, false) => LucideIcon::PanelRightOpen,
                (PrimarySideBarPosition::Right, true) => LucideIcon::PanelLeft,
                (PrimarySideBarPosition::Right, false) => LucideIcon::PanelLeftOpen,
            };

        self.primary_side_bar_toggle.update(cx, |button, cx| {
            button.set_presenter(titlebar_icon_presenter(ControlIcon::Lucide(primary_icon)), cx);
        });
        self.secondary_side_bar_toggle.update(cx, |button, cx| {
            button.set_presenter(titlebar_icon_presenter(ControlIcon::Lucide(secondary_icon)), cx);
        });
    }
}

impl HasShellTheme for VscodeShellApp {
    fn look(&self) -> &Arc<ShadcnLook> {
        &self.look
    }

    fn theme_toggle_button(&self) -> IconButton {
        self.theme_toggle_button.clone()
    }
}

impl Render for VscodeShellApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sans = self.look.mode_tokens().typography.font.sans.family.clone();
        let title_style = self.look.typography_role(ShadcnTextRole::H4);
        let status_style = self.look.typography_role(ShadcnTextRole::P);
        let status_bar_visible = self.layout_config.status_bar_visible;
        let viewport = window.viewport_size();
        let status_height = if status_bar_visible { STATUS_BAR_H } else { 0.0 };
        let workspace_height = px((viewport.height.as_f32() - TITLE_BAR_HEIGHT.as_f32() - status_height).max(1.0));
        self.workspace_layout.update(cx, |layout, cx| {
            layout.set_frame_size(viewport.width, workspace_height, cx);
        });

        let title_bar = TitleBar::new().background_color(chrome.panel_background).border_color(chrome.border).child(
            div()
                .id("shell-vscode-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .text_color(chrome.title_text)
                .font_family(sans.clone())
                .child(div().typography_style(title_style).child("Shell: VS Code"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(self.customize_layout_toggle.clone())
                        .child(self.primary_side_bar_toggle.clone())
                        .child(self.secondary_side_bar_toggle.clone())
                        .child(self.theme_toggle_button.clone()),
                ),
        );

        div()
            .luma_focus_scope(&self.focus_scope)
            .on_action(cx.listener(|this, _: &TogglePrimarySideBar, _window, cx| {
                this.toggle_primary_side_bar(cx);
            }))
            .on_action(cx.listener(|this, _: &TogglePanel, _window, cx| {
                this.set_layout_region_visible(LayoutRegion::Panel, !this.layout_config.panel_visible, cx);
            }))
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .font_family(sans)
            .bg(chrome.app_background)
            .child(title_bar)
            .child(div().flex_1().min_h_0().w_full().child(self.workspace_layout.clone()))
            .when(status_bar_visible, |root| {
                root.child(
                    div()
                        .id("status-bar")
                        .h(px(STATUS_BAR_H))
                        .w_full()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .bg(chrome.panel_background)
                        .border_t_1()
                        .border_color(chrome.border)
                        .px_3()
                        .text_color(chrome.muted_text)
                        .child(div().typography_style(status_style).child("Status Bar")),
                )
            })
            .child(self.customize_layout_dialog.clone())
    }
}

fn wire_customize_layout_subscriptions(
    dialog: &Entity<CustomizeLayoutDialog>,
    cx: &mut Context<VscodeShellApp>,
    subscriptions: &mut Vec<Subscription>,
) {
    subscriptions.push(cx.subscribe(dialog, |_app, _, event: &OverlayWindowEvent, cx| {
        if matches!(event, OverlayWindowEvent::Dismissed) {
            cx.notify();
        }
    }));

    let close = dialog.read(cx).close_button();
    subscriptions.push(cx.subscribe(&close, |app, _, event, cx| {
        if matches!(event, ButtonEvent::Click) {
            app.dismiss_customize_layout(cx);
        }
    }));

    let reset = dialog.read(cx).reset_button();
    subscriptions.push(cx.subscribe(&reset, |app, _, event, cx| {
        if matches!(event, ButtonEvent::Click) {
            app.reset_layout_config(cx);
        }
    }));

    let visibility_group = dialog.read(cx).visibility_group();
    subscriptions.push(cx.subscribe(&visibility_group, |app, _, event: &ControlGroupEvent, cx| {
        let ControlGroupEvent::Change { changed_id, selected, .. } = event else {
            return;
        };
        let region = match changed_id.as_ref() {
            "activity-bar" => LayoutRegion::ActivityBar,
            "secondary-activity-bar" => LayoutRegion::SecondaryActivityBar,
            "primary-side-bar" => LayoutRegion::PrimarySideBar,
            "secondary-side-bar" => LayoutRegion::SecondarySideBar,
            "panel" => LayoutRegion::Panel,
            "status-bar" => LayoutRegion::StatusBar,
            _ => return,
        };
        app.set_layout_region_visible(region, *selected, cx);
    }));

    let position_group = dialog.read(cx).primary_side_bar_position_group();
    subscriptions.push(cx.subscribe(&position_group, |app, _, event: &ControlGroupEvent, cx| {
        let ControlGroupEvent::Change { changed_id, .. } = event else {
            return;
        };
        let position = match changed_id.as_ref() {
            "right" => PrimarySideBarPosition::Right,
            _ => PrimarySideBarPosition::Left,
        };
        app.set_primary_side_bar_position(position, cx);
    }));

    let alignment_group = dialog.read(cx).panel_alignment_group();
    subscriptions.push(cx.subscribe(&alignment_group, |app, _, event: &ControlGroupEvent, cx| {
        let ControlGroupEvent::Change { changed_id, .. } = event else {
            return;
        };
        let alignment = match changed_id.as_ref() {
            "left" => PanelAlignment::Left,
            "right" => PanelAlignment::Right,
            "justify" => PanelAlignment::Justify,
            _ => PanelAlignment::Center,
        };
        app.set_panel_alignment(alignment, cx);
    }));
}

fn titlebar_icon_presenter(icon: ControlIcon) -> ControlPresenter<ButtonRenderModel<()>> {
    Arc::new(move |_, _| match &icon {
        ControlIcon::Lucide(lucide) => div()
            .font_family("lucide")
            .text_size(px(14.0))
            .child(char::from(*lucide).to_string())
            .into_any_element(),
        ControlIcon::SvgPath(path) => gpui::svg().size(px(14.0)).path(path.clone()).into_any_element(),
    })
}
