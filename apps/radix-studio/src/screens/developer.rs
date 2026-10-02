//! Developer tab screen: live tuning tools that are not part of the design docs.

use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, App, Subscription, div, prelude::*, px};
use gpui_luma::controls::tabs::{Tabs, TabsItem, TabsEvent};
use super::tooltip_playground::TooltipPlayground;
use gpui_luma::vstack;
use gpui_luma_look_radix::{Look, SemanticRole};

use super::section::section;
use crate::controls::ClassicShadowEditor;

pub struct State {
    tabs: Entity<Tabs>,
    tooltips: Entity<TooltipPlayground>,
    _subscription: Subscription,
}
impl State {
    pub fn new<M: 'static>(look: &Arc<Look>, cx: &mut Context<M>) -> Self {
        let tabs = gpui_luma_look_radix::Tabs::new("developer-tabs")
            .look(look)
            .line()
            .with_template_modifier(|root, _| root.flex_none().debug_selector(|| "developer-tabs".into()))
            .items([TabsItem::new("buttons").label("Buttons"), TabsItem::new("tooltips").label("Tooltips")])
            .active("buttons")
            .spawn(cx);
        let subscription = cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| cx.notify());
        let tooltips = cx.new(|cx| TooltipPlayground::new(look, cx));
        Self { tabs, tooltips, _subscription: subscription }
    }
    pub fn render(&self, look: &Arc<Look>, shadow_editor: Entity<ClassicShadowEditor>, cx: &App) -> AnyElement {
        let content = if self.tabs.read(cx).active_id().map(|id| id.as_ref()) == Some("tooltips") {
            self.tooltips.clone().into_any_element()
        } else {
            button_page(look, shadow_editor)
        };
        div()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(24.0))
            .w_full()
            .child(self.tabs.clone())
            .child(content)
            .into_any_element()
    }
}

fn button_page(look: &Arc<Look>, shadow_editor: Entity<ClassicShadowEditor>) -> AnyElement {
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();

    vstack! {
        gap=28;
        section(
            "Classic Button",
            "Retunes the raised bubble live. Every Classic button in the app follows.",
            fg,
            muted,
            border,
            shadow_editor.into_any_element(),
        ),
    }
    .w_full()
    .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use gpui::{Bounds, Pixels, Render, TestAppContext, Window};

    struct Page {
        look: Arc<Look>,
        state: State,
        shadow: Entity<ClassicShadowEditor>,
        bounds: HashMap<String, Bounds<Pixels>>,
        _bounds_subscription: Subscription,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().flex().flex_col().child(
                div()
                    .id("developer-test-scroll")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .child(self.state.render(&self.look, self.shadow.clone(), cx)),
            )
        }
    }
    #[test]
    fn developer_tabs_switch_to_and_from_tooltip_preview() {
        let mut app = TestAppContext::single();
        let look = Arc::new(Look::built_in());
        let (page, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            let state = State::new(&look, cx);
            let bounds_subscription = cx.subscribe(&state.tabs, |this: &mut Page, _, event: &TabsEvent, _| {
                if let TabsEvent::ItemBoundsChanged { tab_id, bounds } = event {
                    this.bounds.insert(tab_id.to_string(), *bounds);
                }
            });
            Page {
                look: look.clone(),
                state,
                shadow: cx.new(|cx| ClassicShadowEditor::new(&look, cx)),
                bounds: HashMap::new(),
                _bounds_subscription: bounds_subscription,
            }
        });
        cx.simulate_resize(gpui::size(px(800.0), px(600.0)));
        cx.run_until_parked();
        assert!(cx.debug_bounds("tooltip-preview").is_none());
        for tab in ["tooltips", "buttons", "tooltips"] {
            let bounds = cx.update(|_, app| page.read(app).bounds[tab]);
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            assert_eq!(
                cx.update(|_, app| page.read(app).state.tabs.read(app).active_id().cloned()),
                Some(tab.into()),
                "clicked {tab} at {bounds:?}"
            );
            assert_eq!(cx.debug_bounds("tooltip-preview").is_some(), tab == "tooltips");
        }
    }
}
