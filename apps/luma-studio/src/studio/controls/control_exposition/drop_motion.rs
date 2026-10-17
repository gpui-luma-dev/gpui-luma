//! Shared SDK-backed controls for inspecting committed drop-layout motion.
use std::{hash::Hash, sync::Arc, time::Duration};
use gpui::{AppContext, Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::{controls::button::Button, infra::presenter::HasPresenter, motion::DropLayoutMotion};
use gpui_luma_look_shadcn::{self as shadcn, ShadcnLook};

pub(super) struct DropMotionControls<K> {
    look: Arc<ShadcnLook>,
    motions: Vec<DropLayoutMotion<K>>,
    buttons: Vec<Entity<Button>>,
}
impl<K: Clone + Eq + Hash + 'static> DropMotionControls<K> {
    pub(super) fn spawn<M: 'static>(
        look: Arc<ShadcnLook>,
        motions: Vec<DropLayoutMotion<K>>,
        cx: &mut Context<M>,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let buttons = [
                ("theme", "Theme", None),
                ("slow", "Slow (800 ms)", Some(Duration::from_millis(800))),
                ("off", "Off", Some(Duration::ZERO)),
            ]
            .into_iter()
            .map(|(id, label, duration)| {
                shadcn::Button::new(format!("drop-motion-{id}"))
                    .look(&look)
                    .outline()
                    .size(shadcn::ShadcnSize::Sm)
                    .label(label)
                    .with_template_modifier(move |root, _| root.debug_selector(move || format!("drop-motion-{id}")))
                    .on_click(cx, move |this: &mut Self, cx| {
                        for motion in &this.motions {
                            motion.set_duration(duration);
                        }
                        cx.refresh_windows();
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
            Self { look, motions, buttons }
        })
    }
}
impl<K: Clone + Eq + Hash + 'static> Render for DropMotionControls<K> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ms = self.motions.first().map_or(0, |m| m.resolved().duration.as_millis());
        div()
            .w_full()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .child(div().text_sm().text_color(self.look.chrome().muted_text).child(format!("Drop motion · {ms} ms")))
            .children(self.buttons.iter().cloned())
    }
}
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    #[test]
    fn sdk_buttons_switch_slow_off_and_theme_for_all_samples() {
        struct Page(Entity<DropMotionControls<u32>>);
        impl Render for Page {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                self.0.clone()
            }
        }
        let motion = DropLayoutMotion::<u32>::default();
        let second = motion.clone();
        let mut app = TestAppContext::single();
        let (_, cx) = app.add_window_view(|_, cx| {
            Page(DropMotionControls::spawn(Arc::new(ShadcnLook::built_in()), vec![second], cx))
        });
        cx.run_until_parked();
        for (selector, ms) in [("drop-motion-slow", 800), ("drop-motion-off", 0), ("drop-motion-theme", 200)] {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            assert_eq!(motion.resolved().duration.as_millis(), ms);
        }
    }
}
