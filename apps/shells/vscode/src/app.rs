use std::sync::Arc;

use gpui::{Context, Div, Entity, FocusHandle, Focusable, Render, Stateful, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ControlIcon, ControlPresenter};
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::overlay_window::OverlayWindowEvent;
use gpui_luma::controls::resizable_panels::{PanelHideMode, ResizablePanelsEvent};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::{TITLE_BAR_HEIGHT, TitleBar};
use gpui_luma::theme::{ControlSize, ThemeMode};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextRole};
use gpui_luma_shell_common::{
    chrome::{HasShellTheme, handle_theme_toggle, spawn_theme_toggle_button},
    theme::{ShellThemeChoice, sync_color_control_theme},
};
use lucide_icons::Icon as LucideIcon;

use crate::customize_layout_dialog::CustomizeLayoutDialog;
use crate::layout_config::{LayoutConfig, LayoutRegion, PanelAlignment, PrimarySideBarPosition};
use crate::workbench_layout::{
    LEFT_EDGE_PANEL_INDEX, RIGHT_EDGE_PANEL_INDEX, PrimarySideBar, SecondarySideBar, WORKBENCH_PANEL_H, WorkbenchLayout,
};

const ACTIVITY_BAR_W: f32 = 48.0;
const STATUS_BAR_H: f32 = 32.0;

pub struct VscodeShellApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    layout_config: LayoutConfig,
    workbench: WorkbenchLayout,
    activity_bar_buttons: [IconButton; 5],
    secondary_activity_bar_buttons: [IconButton; 3],
    customize_layout_toggle: IconButton,
    primary_side_bar_toggle: IconButton,
    secondary_side_bar_toggle: IconButton,
    theme_toggle_button: IconButton,
    customize_layout_dialog: Entity<CustomizeLayoutDialog>,
    _subscriptions: Vec<Subscription>,
}

impl VscodeShellApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let layout_config = LayoutConfig::default();

        let workbench = WorkbenchLayout::new(
            "shell-vscode",
            look.clone(),
            PrimarySideBar::new("primary-side-bar", look.clone()).width(px(360.0)),
            SecondarySideBar::new("secondary-side-bar", look.clone()).width(px(360.0)),
            cx,
        );

        let customize_layout_toggle = look
            .content_only_icon_button("shell-vscode-customize-layout-toggle", LucideIcon::LayoutPanelLeft)
            .size(ControlSize::Sm)
            .spawn(cx);
        let activity_bar_buttons = spawn_activity_rail_buttons(
            "shell-vscode-activity-bar",
            [LucideIcon::Files, LucideIcon::Search, LucideIcon::GitBranch, LucideIcon::Bug, LucideIcon::Blocks],
            &look,
            cx,
        );
        let secondary_activity_bar_buttons = spawn_activity_rail_buttons(
            "shell-vscode-secondary-activity-bar",
            [LucideIcon::User, LucideIcon::Bell, LucideIcon::Settings],
            &look,
            cx,
        );
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

        subscriptions.push(cx.subscribe(&workbench.panels(), |this, _, event: &ResizablePanelsEvent, cx| {
            this.handle_workbench_event(event, cx);
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
                this.refresh_customize_layout_dialog_theme(cx);
            }
        }));

        let mut app = Self {
            focus_scope,
            look,
            layout_config,
            workbench,
            activity_bar_buttons,
            secondary_activity_bar_buttons,
            customize_layout_toggle,
            primary_side_bar_toggle,
            secondary_side_bar_toggle,
            theme_toggle_button,
            customize_layout_dialog,
            _subscriptions: subscriptions,
        };
        app.sync_workbench_theme(cx);
        app.apply_layout_config(cx);
        app
    }

    pub fn layout_config(&self) -> &LayoutConfig {
        &self.layout_config
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
        self.workbench.set_primary_side_bar_position(self.layout_config.primary_side_bar_position, cx);
        self.customize_layout_dialog.update(cx, |dialog, cx| {
            dialog.sync_from_config(&self.layout_config, cx);
        });
        self.apply_layout_config(cx);
    }

    pub fn toggle_layout_region(&mut self, region: LayoutRegion, cx: &mut Context<Self>) {
        self.layout_config.toggle_region(region);
        match region {
            LayoutRegion::ActivityBar
            | LayoutRegion::SecondaryActivityBar
            | LayoutRegion::Panel
            | LayoutRegion::StatusBar => {
                cx.notify();
            }
            LayoutRegion::PrimarySideBar => {
                self.set_panel_visible(
                    self.primary_side_bar_panel_index(),
                    self.layout_config.primary_side_bar_visible,
                    cx,
                );
                self.sync_side_bar_toggle_icons(cx);
                cx.notify();
            }
            LayoutRegion::SecondarySideBar => {
                self.set_panel_visible(
                    self.secondary_side_bar_panel_index(),
                    self.layout_config.secondary_side_bar_visible,
                    cx,
                );
                self.sync_side_bar_toggle_icons(cx);
                cx.notify();
            }
        }
        self.refresh_customize_layout_dialog(cx);
    }

    pub fn set_primary_side_bar_position(&mut self, position: PrimarySideBarPosition, cx: &mut Context<Self>) {
        if self.layout_config.primary_side_bar_position == position {
            return;
        }
        self.layout_config.primary_side_bar_position = position;
        self.workbench.set_primary_side_bar_position(position, cx);
        self.apply_layout_config(cx);
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    pub fn set_panel_alignment(&mut self, alignment: PanelAlignment, cx: &mut Context<Self>) {
        if self.layout_config.panel_alignment == alignment {
            return;
        }
        self.layout_config.panel_alignment = alignment;
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    fn sync_workbench_theme(&self, cx: &mut Context<Self>) {
        let theme = self.look.resizable_panels_theme();
        self.workbench.sync_chrome(&self.look, cx);
        self.workbench.sync_theme(&theme, cx);
    }

    fn refresh_customize_layout_dialog(&self, cx: &mut Context<Self>) {
        self.customize_layout_dialog.update(cx, |dialog, cx| {
            dialog.sync_from_config(&self.layout_config, cx);
        });
    }

    fn refresh_customize_layout_dialog_theme(&self, cx: &mut Context<Self>) {
        self.customize_layout_dialog.update(cx, |dialog, cx| {
            dialog.sync_from_config(&self.layout_config, cx);
            dialog.refresh_theme(cx);
        });
    }

    fn apply_layout_config(&mut self, cx: &mut Context<Self>) {
        self.set_panel_visible(self.primary_side_bar_panel_index(), self.layout_config.primary_side_bar_visible, cx);
        self.set_panel_visible(
            self.secondary_side_bar_panel_index(),
            self.layout_config.secondary_side_bar_visible,
            cx,
        );
        self.sync_side_bar_toggle_icons(cx);
        self.refresh_customize_layout_dialog(cx);
        cx.notify();
    }

    fn set_panel_visible(&self, panel_index: usize, visible: bool, cx: &mut Context<Self>) {
        self.workbench.panels().update(cx, |panels, cx| {
            let hidden = panels.is_panel_hidden(panel_index);
            if hidden == visible {
                panels.toggle_panel_hidden(panel_index, PanelHideMode::Completely, cx);
            }
        });
    }

    fn handle_workbench_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<Self>) {
        if let ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden } = event {
            if *panel_index == self.primary_side_bar_panel_index() {
                self.layout_config.primary_side_bar_visible = !hidden;
            } else if *panel_index == self.secondary_side_bar_panel_index() {
                self.layout_config.secondary_side_bar_visible = !hidden;
            }
            self.sync_side_bar_toggle_icons(cx);
            self.refresh_customize_layout_dialog(cx);
        }
        cx.notify();
    }

    fn primary_side_bar_panel_index(&self) -> usize {
        match self.layout_config.primary_side_bar_position {
            PrimarySideBarPosition::Left => LEFT_EDGE_PANEL_INDEX,
            PrimarySideBarPosition::Right => RIGHT_EDGE_PANEL_INDEX,
        }
    }

    fn secondary_side_bar_panel_index(&self) -> usize {
        match self.layout_config.primary_side_bar_position {
            PrimarySideBarPosition::Left => RIGHT_EDGE_PANEL_INDEX,
            PrimarySideBarPosition::Right => LEFT_EDGE_PANEL_INDEX,
        }
    }

    fn toggle_primary_side_bar(&mut self, cx: &mut Context<Self>) {
        self.layout_config.toggle_region(LayoutRegion::PrimarySideBar);
        self.apply_layout_config(cx);
    }

    fn toggle_secondary_side_bar(&mut self, cx: &mut Context<Self>) {
        self.layout_config.toggle_region(LayoutRegion::SecondarySideBar);
        self.apply_layout_config(cx);
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
        let status_bar_visible = self.layout_config.status_bar_visible;
        let panel_visible = self.layout_config.panel_visible;
        let activity_bar_visible = self.layout_config.activity_bar_visible;
        let secondary_activity_bar_visible = self.layout_config.secondary_activity_bar_visible;
        let (left_activity_rail, right_activity_rail) = match self.layout_config.primary_side_bar_position {
            PrimarySideBarPosition::Left => (
                ActivityRailSpec {
                    id: "activity-bar",
                    visible: activity_bar_visible,
                    buttons: &self.activity_bar_buttons,
                },
                ActivityRailSpec {
                    id: "secondary-activity-bar",
                    visible: secondary_activity_bar_visible,
                    buttons: &self.secondary_activity_bar_buttons,
                },
            ),
            PrimarySideBarPosition::Right => (
                ActivityRailSpec {
                    id: "secondary-activity-bar",
                    visible: secondary_activity_bar_visible,
                    buttons: &self.secondary_activity_bar_buttons,
                },
                ActivityRailSpec {
                    id: "activity-bar",
                    visible: activity_bar_visible,
                    buttons: &self.activity_bar_buttons,
                },
            ),
        };
        let viewport = window.viewport_size();
        let rails_width = [activity_bar_visible, secondary_activity_bar_visible]
            .into_iter()
            .filter(|visible| *visible)
            .count() as f32
            * ACTIVITY_BAR_W;
        let workbench_width = px((viewport.width.as_f32() - rails_width).max(1.0));
        let status_height = if status_bar_visible { STATUS_BAR_H } else { 0.0 };
        let floating_panel_visible =
            panel_visible && matches!(self.layout_config.panel_alignment, PanelAlignment::Left | PanelAlignment::Right);
        let external_panel_visible = panel_visible && self.layout_config.panel_alignment == PanelAlignment::Justify;
        let panel_height = if external_panel_visible { WORKBENCH_PANEL_H } else { 0.0 };
        let workbench_height =
            px((viewport.height.as_f32() - TITLE_BAR_HEIGHT.as_f32() - panel_height - status_height).max(1.0));
        self.workbench.set_panel_layout(panel_visible, self.layout_config.panel_alignment, cx);
        self.workbench.set_frame_size(workbench_width, workbench_height, cx);
        let panel_sizes = self.workbench.panel_sizes_px(cx);
        let left_edge_width = panel_sizes.get(LEFT_EDGE_PANEL_INDEX).copied().unwrap_or(0.0);
        let right_edge_width = panel_sizes.get(RIGHT_EDGE_PANEL_INDEX).copied().unwrap_or(0.0);
        let left_pair_width = if left_activity_rail.visible {
            ACTIVITY_BAR_W
        } else {
            0.0
        } + left_edge_width;
        let right_pair_width = if right_activity_rail.visible {
            ACTIVITY_BAR_W
        } else {
            0.0
        } + right_edge_width;
        let viewport_width = viewport.width.as_f32();
        let external_panel_geometry = ExternalPanelGeometry {
            left_span_width: px((viewport_width - right_pair_width).max(1.0)),
            right_span_left: px(left_pair_width),
            right_span_width: px((viewport_width - left_pair_width).max(1.0)),
        };

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
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .font_family(sans)
            .bg(chrome.app_background)
            .child(title_bar)
            .child(
                div()
                    .id("workbench-body")
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .when(left_activity_rail.visible, |body| {
                        body.child(activity_bar_rail(
                            left_activity_rail.id,
                            &self.look,
                            left_activity_rail.buttons,
                            ActivityRailEdge::Left,
                        ))
                    })
                    .child(div().flex_1().min_w_0().min_h_0().child(self.workbench.panels()))
                    .when(right_activity_rail.visible, |body| {
                        body.child(activity_bar_rail(
                            right_activity_rail.id,
                            &self.look,
                            right_activity_rail.buttons,
                            ActivityRailEdge::Right,
                        ))
                    })
                    .when(floating_panel_visible, |body| {
                        body.child(floating_output_panel(
                            &self.look,
                            self.layout_config.panel_alignment,
                            external_panel_geometry,
                        ))
                    }),
            )
            .when(external_panel_visible, |root| {
                root.child(output_panel(&self.look, self.layout_config.panel_alignment, external_panel_geometry))
            })
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
                        .border_color(chrome.border),
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

    for region in [
        LayoutRegion::ActivityBar,
        LayoutRegion::SecondaryActivityBar,
        LayoutRegion::PrimarySideBar,
        LayoutRegion::SecondarySideBar,
        LayoutRegion::Panel,
        LayoutRegion::StatusBar,
    ] {
        let row = dialog.read(cx).visibility_row(region);
        subscriptions.push(cx.subscribe(&row, move |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.toggle_layout_region(region, cx);
            }
        }));
    }

    for position in [PrimarySideBarPosition::Left, PrimarySideBarPosition::Right] {
        let row = dialog.read(cx).primary_side_bar_position_row(position);
        subscriptions.push(cx.subscribe(&row, move |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.set_primary_side_bar_position(position, cx);
            }
        }));
    }

    for alignment in [PanelAlignment::Left, PanelAlignment::Right, PanelAlignment::Center, PanelAlignment::Justify] {
        let row = dialog.read(cx).panel_alignment_row(alignment);
        subscriptions.push(cx.subscribe(&row, move |app, _, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.set_panel_alignment(alignment, cx);
            }
        }));
    }
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

fn spawn_activity_rail_buttons<const N: usize>(
    prefix: &'static str,
    icons: [LucideIcon; N],
    look: &Arc<ShadcnLook>,
    cx: &mut Context<VscodeShellApp>,
) -> [IconButton; N] {
    std::array::from_fn(|index| {
        look.content_only_icon_button(format!("{prefix}-{index}"), icons[index])
            .size(ControlSize::Md)
            .spawn(cx)
    })
}

#[derive(Clone, Copy)]
enum ActivityRailEdge {
    Left,
    Right,
}

struct ActivityRailSpec<'a> {
    id: &'static str,
    visible: bool,
    buttons: &'a [IconButton],
}

#[derive(Clone, Copy)]
struct ExternalPanelGeometry {
    left_span_width: gpui::Pixels,
    right_span_left: gpui::Pixels,
    right_span_width: gpui::Pixels,
}

fn activity_bar_rail(
    id: &'static str,
    look: &ShadcnLook,
    buttons: &[IconButton],
    edge: ActivityRailEdge,
) -> impl IntoElement {
    let chrome = look.chrome();
    let mut rail = div()
        .id(id)
        .w(px(ACTIVITY_BAR_W))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .justify_between()
        .py(px(8.0))
        .bg(chrome.panel_background)
        .border_color(chrome.border);

    rail = match edge {
        ActivityRailEdge::Left => rail.border_r_1(),
        ActivityRailEdge::Right => rail.border_l_1(),
    };

    let mut primary = div().flex().flex_col().items_center().gap(px(4.0));
    for button in buttons.iter().take(5) {
        primary = primary.child(button.clone());
    }

    rail.child(primary)
}

fn output_panel(look: &ShadcnLook, alignment: PanelAlignment, geometry: ExternalPanelGeometry) -> impl IntoElement {
    let panel = panel_surface(look);

    let panel = match alignment {
        PanelAlignment::Justify => panel.w_full(),
        PanelAlignment::Left => panel.w(geometry.left_span_width),
        PanelAlignment::Right => panel.w(geometry.right_span_width),
        PanelAlignment::Center => panel.w(geometry.right_span_width),
    };

    let wrapper = div().h(px(WORKBENCH_PANEL_H)).w_full().flex_shrink_0().flex().bg(look.chrome().content_background);

    match alignment {
        PanelAlignment::Left | PanelAlignment::Justify => wrapper.justify_start().child(panel),
        PanelAlignment::Right => wrapper.justify_start().child(panel.ml(geometry.right_span_left)),
        PanelAlignment::Center => wrapper.justify_center().child(panel),
    }
}

fn floating_output_panel(
    look: &ShadcnLook,
    alignment: PanelAlignment,
    geometry: ExternalPanelGeometry,
) -> impl IntoElement {
    let panel = panel_surface(look).absolute().bottom(px(0.0));

    match alignment {
        PanelAlignment::Left => panel.left(px(0.0)).w(geometry.left_span_width),
        PanelAlignment::Right => panel.left(geometry.right_span_left).w(geometry.right_span_width).border_l_1(),
        PanelAlignment::Center | PanelAlignment::Justify => panel.left(px(0.0)).w_full(),
    }
}

fn panel_surface(look: &ShadcnLook) -> Stateful<Div> {
    let chrome = look.chrome();
    let label_style = look.typography_role(ShadcnTextRole::P);

    div()
        .id("panel")
        .h(px(WORKBENCH_PANEL_H))
        .bg(chrome.panel_background)
        .border_t_1()
        .border_color(chrome.border)
        .p(px(12.0))
        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("Panel"))
}
