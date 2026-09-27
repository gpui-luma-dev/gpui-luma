use gpui::{Context, Entity, FocusHandle, Render, Subscription, Window, div, prelude::*, px};
use luma::controls::tabs::{Tabs, TabsEvent, TabsItem};
use luma_color::style::{ColorControlTheme, set_active_color_control_theme};
use luma::focus::LumaFocusScopeExt;
use luma::shell::TitleBar;
use luma::theme::ThemeMode;
use luma_look_shadcn::ShadcnLook;
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
        let tabs = luma_look_shadcn::Tabs::new("color-viz-workspace-tabs")
            .look(look.as_ref())
            .items([
                TabsItem::new("gradients").label("Gradient Builder"),
                TabsItem::new("compositions").label("Color Compositions"),
            ])
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

        let title_bar = TitleBar::new()
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
                    .child(div().child(if self.show_compositions {
                        "Color Viz - Color Compositions"
                    } else {
                        "Color Viz - Gradients"
                    })),
            );

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .font_family(sans_family)
            .bg(chrome.app_background)
            .child(title_bar)
            .child(div().w_full().flex_shrink_0().p(px(10.0)).bg(chrome.panel_background).child(self.tabs.clone()))
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
        app.update(|cx| luma::init(cx).expect("initialize SDK"));
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
