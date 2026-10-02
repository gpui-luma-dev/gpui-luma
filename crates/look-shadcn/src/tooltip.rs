//! Tooltip presentation bound to the control's mutable Shadcn look.
use std::sync::Arc;
use gpui_luma::controls::tooltip::{TooltipLook, TooltipTheme};
use crate::{ShadcnLook, ShadcnToken, ShadcnRadius};
struct ShadcnTooltipTheme(ShadcnLook);
impl TooltipTheme for ShadcnTooltipTheme {
    fn resolve(&self) -> TooltipLook {
        TooltipLook {
            background: self.0.color(ShadcnToken::Foreground),
            foreground: self.0.color(ShadcnToken::Background),
            padding: 8.0,
            padding_y: 4.0,
            radius: self.0.radius(ShadcnRadius::Md),
            max_width: 260.0,
            text_size: 12.0,
            line_height: 16.0,
            shadow: Vec::new(),
        }
    }
}
/// Resolve tooltip tokens on each presentation, including mode changes.
pub fn tooltip_theme(look: &ShadcnLook) -> Arc<dyn TooltipTheme> {
    Arc::new(ShadcnTooltipTheme(look.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::ThemeMode;
    #[test]
    fn tooltip_tracks_bound_look_mode() {
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
        let theme = tooltip_theme(&look);
        let light = theme.resolve();
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve();
        assert_ne!(light.background, dark.background);
        assert_eq!(dark.background, look.color(ShadcnToken::Foreground));
        assert_eq!(dark.foreground, look.color(ShadcnToken::Background));
    }
}

#[cfg(all(test, feature = "test-support"))]
mod coverage_tests {
    use super::*;
    use gpui::{AnyView, Context, IntoElement, Render, TestAppContext, Window};
    use gpui_luma::{
        controls::tooltip::{Tooltip, bubble},
        infra::attachments::TooltipEntityExt,
    };
    use gpui_luma::theme::ThemeMode;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    struct Page {
        target: AnyView,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.target.clone()
        }
    }
    macro_rules! coverage {
        ($name:ident, $builder:expr) => {
            #[test]
            fn $name() {
                let mut app = TestAppContext::single();
                let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
                let colors = Arc::new(Mutex::new(Vec::new()));
                let log = colors.clone();
                let (_, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    let target = $builder.look(&look).spawn(cx).tooltip(
                        Tooltip::new("Help").delay(Duration::ZERO).template(move |model| {
                            log.lock().unwrap().push((model.background, model.foreground));
                            bubble(model)
                        }),
                        cx,
                    );
                    Page { target: target.into() }
                });
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
                    ..Default::default()
                });
                cx.run_until_parked();
                assert_eq!(
                    colors.lock().unwrap().last(),
                    Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
                );
                look.set_mode(ThemeMode::Dark);
                cx.update(|window, _| window.refresh());
                cx.run_until_parked();
                // A theme change may resize the target and dismiss its current tip.
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(600.0), gpui::px(600.0)),
                    ..Default::default()
                });
                cx.executor().advance_clock(Duration::from_millis(200));
                cx.run_until_parked();
                cx.simulate_event(gpui::MouseMoveEvent {
                    position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
                    ..Default::default()
                });
                cx.run_until_parked();
                assert_eq!(
                    colors.lock().unwrap().last(),
                    Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
                );
            }
        };
    }
    coverage!(button_help_tracks_look, crate::Button::new("button"));
    coverage!(icon_button_help_tracks_look, crate::Button::icon_button("icon", lucide_svg_static::Icon::Plus));
    coverage!(typed_button_help_tracks_look, crate::Button::new("typed").typed(42u32));
    coverage!(checkbox_help_tracks_look, crate::Checkbox::new("checkbox"));
    coverage!(switch_help_tracks_look, crate::Switch::new("switch"));
    coverage!(radio_help_tracks_look, crate::Radio::new("radio"));
    coverage!(toggle_help_tracks_look, crate::Toggle::new("toggle"));
    coverage!(textfield_help_tracks_look, crate::TextField::new("textfield"));
    #[test]
    fn sidebar_item_help_tracks_look() {
        let mut app = TestAppContext::single();
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
        let colors = Arc::new(Mutex::new(Vec::new()));
        let log = colors.clone();
        let (_, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            let target = crate::Sidebar::new("sidebar")
                .look(&look)
                .animated(false)
                .sidebar(crate::Sidebar::panel("content").content(crate::Sidebar::content().group(
                    crate::Sidebar::group().menu(crate::Sidebar::menu("main").item(
                        crate::Sidebar::menu_item("first", "First").icon(lucide_svg_static::Icon::House).tooltip(
                            Tooltip::new("First help").delay(Duration::ZERO).template(move |model| {
                                log.lock().unwrap().push((model.background, model.foreground));
                                bubble(model)
                            }),
                        ),
                    )),
                )))
                .spawn(cx);
            Page { target: target.into() }
        });
        cx.simulate_event(gpui::MouseMoveEvent {
            position: gpui::point(gpui::px(10.0), gpui::px(10.0)),
            ..Default::default()
        });
        cx.run_until_parked();
        assert_eq!(
            colors.lock().unwrap().last(),
            Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
        );
        look.set_mode(ThemeMode::Dark);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert_eq!(
            colors.lock().unwrap().last(),
            Some(&(look.color(ShadcnToken::Foreground), look.color(ShadcnToken::Background)))
        );
    }
}
