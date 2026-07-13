use std::sync::Arc;

use gpui::{Context, Entity, FocusHandle, IntoElement, Render, Window, div, prelude::*, px};
use gpui_luma::controls::resizable_panels::{
    ResizeHandleSize, ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation,
};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_shell_common::theme::{ShellThemeChoice, sync_color_control_theme};

use crate::column::SplitColumn;

pub struct SplitTitlebarShellApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    panels: Entity<ResizablePanels>,
}

impl SplitTitlebarShellApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, theme_choice: ShellThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);

        let chrome = look.chrome();
        let left_background = chrome.panel_background;
        let right_background = chrome.content_background;

        let left_column = cx.new(|_| SplitColumn::new(left_background, true));
        let right_column = cx.new(|_| SplitColumn::new(right_background, false));

        let panels = look
            .resizable_panels("shell-full-window")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .show_border(false)
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Md)
            .handle_grip(true)
            .panels([
                ResizablePanelSpec::new_render({
                    let left_column = left_column.clone();
                    move || left_column.clone()
                })
                .weight(3.0)
                .bg(left_background)
                .min(px(180.0)),
                ResizablePanelSpec::new_render({
                    let right_column = right_column.clone();
                    move || right_column.clone()
                })
                .weight(7.0)
                .bg(right_background)
                .min(px(180.0)),
            ])
            .spawn(cx);

        Self { focus_scope, look, panels }
    }
}

impl Render for SplitTitlebarShellApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let sans_family = self.look.mode_tokens().typography.font.sans.family.clone();

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .font_family(sans_family)
            .child(self.panels.clone())
    }
}
