use gpui::{Context, Entity, FocusHandle, Render, Window, div, prelude::*};
use gpui_luma::controls::color::style::{ColorControlTheme, set_active_color_control_theme};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use std::sync::Arc;

use crate::gradient_builder::GradientBuilder;
use crate::theme::ColorVizThemeChoice;

pub struct ColorVizApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    gradient_builder: Entity<GradientBuilder>,
}

impl ColorVizApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, theme_choice: ColorVizThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);
        let gradient_builder = cx.new(|cx| GradientBuilder::new(look.clone(), cx));

        Self { focus_scope, look, gradient_builder }
    }
}

fn sync_color_control_theme(look: &ShadcnLook) {
    let chrome = look.chrome();

    set_active_color_control_theme(ColorControlTheme::new(
        chrome.border,
        chrome.panel_background,
        matches!(look.mode(), ThemeMode::Dark),
    ));
}

impl Render for ColorVizApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sans_family = self.look.mode_tokens().typography.font.sans.family.clone();

        let title_bar = TitleBar::new().background_color(chrome.panel_background).border_color(chrome.border).child(
            div()
                .id("color-viz-titlebar")
                .h_full()
                .w_full()
                .flex()
                .items_center()
                .px_2()
                .text_color(chrome.title_text)
                .font_family(sans_family.clone())
                .child(div().child("Color Viz - Gradients")),
        );

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .font_family(sans_family)
            .bg(chrome.app_background)
            .child(title_bar)
            .child(div().flex_1().min_h_0().bg(chrome.content_background).child(self.gradient_builder.clone()))
    }
}
