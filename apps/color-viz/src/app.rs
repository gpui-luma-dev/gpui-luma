use gpui::{Context, Entity, FocusHandle, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::tabs::{Tabs, TabsEvent, TabsItem};
use gpui_luma_color::style::{ColorControlTheme, set_active_color_control_theme};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use std::sync::Arc;

use crate::gradient_builder::GradientBuilder;
use crate::compositions::ColorCompositions;
use crate::theme::ColorVizThemeChoice;

pub struct ColorVizApp {
    focus_scope: FocusHandle,
    look: Arc<ShadcnLook>,
    gradient_builder: Entity<GradientBuilder>,
    compositions: Option<Entity<ColorCompositions>>,
    tabs: Entity<Tabs>,
    show_compositions: bool,
    _subscriptions: Vec<Subscription>,
}

impl ColorVizApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>, theme_choice: ColorVizThemeChoice) -> Self {
        let focus_scope = cx.focus_handle();
        let look = theme_choice.shadcn_look();
        look.set_mode(ThemeMode::Dark);
        sync_color_control_theme(&look);
        let gradient_builder = cx.new(|cx| GradientBuilder::new(look.clone(), cx));
        let tabs = gpui_luma_look_shadcn::Tabs::new("color-viz-workspace-tabs")
            .look(look.as_ref())
            .template(crate::studio_tabs::template(look.clone(), false))
            .items([TabsItem::new("gradients").label("Gradients"), TabsItem::new("compositions").label("Compositions")])
            .size(gpui_luma_look_shadcn::ShadcnSize::Sm)
            .active("gradients")
            .spawn(cx);
        let subscriptions = vec![cx.subscribe(&tabs, |this, _, event: &TabsEvent, cx| {
            if let TabsEvent::Activate { tab_id, .. } = event {
                this.show_compositions = tab_id == "compositions";
                if this.show_compositions && this.compositions.is_none() {
                    this.compositions = Some(cx.new(|cx| ColorCompositions::new(this.look.clone(), cx)));
                }
                cx.notify();
            }
        })];

        Self {
            focus_scope,
            look,
            gradient_builder,
            compositions: None,
            tabs,
            show_compositions: false,
            _subscriptions: subscriptions,
        }
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

        let export_button = self.gradient_builder.read(_cx).export_button();
        let title_bar = TitleBar::new()
            .height(px(48.0))
            .background_color(chrome.panel_background)
            .border_color(chrome.border)
            .text_color(chrome.title_text)
            .child(
                div()
                    .id("color-viz-titlebar")
                    .h_full()
                    .w_full()
                    .flex()
                    .items_center()
                    .px_2()
                    .text_color(chrome.title_text)
                    .font_family(sans_family.clone())
                    .child(div().text_size(px(14.0)).font_weight(gpui::FontWeight::SEMIBOLD).child("Color Viz"))
                    .child(
                        div()
                            .ml(px(28.0))
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(self.tabs.clone()),
                    )
                    .child(div().flex_1())
                    .when(!self.show_compositions, |this| {
                        this.child(
                            div()
                                .mr(px(12.0))
                                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .child(export_button),
                        )
                    }),
            );

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .font_family(sans_family)
            .bg(chrome.app_background)
            .text_color(chrome.body_text)
            .child(title_bar)
            .child(div().flex_1().min_h_0().bg(chrome.content_background).child(
                match (self.show_compositions, &self.compositions) {
                    (true, Some(view)) => view.clone().into_any_element(),
                    _ => self.gradient_builder.clone().into_any_element(),
                },
            ))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn workspace_navigation_loads_compositions_once_and_retains_both_workspaces() {
        let mut app = TestAppContext::single();
        app.update(|cx| gpui_luma::init(cx).expect("initialize SDK"));
        let (view, cx) = app.add_window_view(|window, cx| ColorVizApp::new(window, cx, ColorVizThemeChoice::Default));
        cx.run_until_parked();
        let (tabs, gradients) = cx.update(|_, cx| {
            let view = view.read(cx);
            assert!(view.compositions.is_none());
            (view.tabs.clone(), view.gradient_builder.clone())
        });
        let mut first_compositions = None;
        for id in ["compositions", "gradients", "compositions"] {
            tabs.update(cx, |tabs, cx| {
                tabs.set_active(id, cx);
                cx.emit(TabsEvent::Activate { tab_id: id.into(), label: id.into() });
            });
            cx.run_until_parked();
            cx.update(|_, cx| {
                let view = view.read(cx);
                assert_eq!(view.show_compositions, id == "compositions");
                assert_eq!(view.gradient_builder, gradients);
                let compositions = view.compositions.as_ref().expect("loaded compositions");
                if let Some(first) = &first_compositions {
                    assert_eq!(compositions, first);
                } else {
                    first_compositions = Some(compositions.clone());
                }
            });
        }
    }
}
