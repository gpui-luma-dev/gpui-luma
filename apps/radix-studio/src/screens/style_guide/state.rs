//! Persistent guide controls and subscriptions, created on the first visit.
use std::sync::Arc;
use gpui::{AnyElement, App, Context, Entity, Subscription, Window, prelude::*};
use gpui_luma::controls::{
    tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode},
    tree_view::TreeView,
};
use gpui_luma_look_radix::Look;
use super::{PreviewTabs, TabsExamples};

pub struct State {
    tabs: PreviewTabs,
    tree: TreeView<()>,
    disabled_tree: TreeView<()>,
    _subscriptions: Vec<Subscription>,
}

impl State {
    pub fn new<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        let mut subscriptions = Vec::new();
        let mut navigation = |name| preview_navigation(name, look, &mut subscriptions, cx);
        let tabs = PreviewTabs {
            avatars: navigation("avatars"),
            badges: navigation("badges"),
            buttons: navigation("buttons"),
            checkboxes: navigation("checkboxes"),
            radios: navigation("radios"),
            switches: navigation("switches"),
            textfields: navigation("textfields"),
            textareas: navigation("textareas"),
            sliders: navigation("sliders"),
            progress: navigation("progress"),
            examples: TabsExamples::spawn(look, cx),
            tooltips: super::tooltips::TooltipExamples::new(look, cx),
        };
        let tree = super::super::shared::tree_view::spawn("guide-tree", look, true, cx);
        let disabled_tree = super::super::shared::tree_view::spawn("guide-tree-disabled", look, false, cx);
        for tree in [&tree, &disabled_tree] {
            tree.update(cx, |tree, cx| {
                tree.set_wheel_scroll_policy(gpui_luma::interaction::WheelScrollPolicy::PassThrough, cx);
            });
        }
        Self { tabs, tree, disabled_tree, _subscriptions: subscriptions }
    }

    pub fn render(&self, look: &Arc<Look>, window: &mut Window, cx: &mut App) -> AnyElement {
        super::page(look, self.tabs.clone(), self.tree.clone(), self.disabled_tree.clone(), window, cx)
    }
}

fn preview_navigation<M: 'static>(
    name: &str,
    look: &Look,
    subscriptions: &mut Vec<Subscription>,
    cx: &mut Context<M>,
) -> Entity<Tabs> {
    let tabs = gpui_luma_look_radix::Tabs::new(format!("radix-studio-{name}-preview-tabs"))
        .look(look)
        .items([
            TabsItem::new("template-preview").label("Template Preview"),
            TabsItem::new("colors").label("Colors"),
            TabsItem::new("all-sizes").label("All Sizes"),
        ])
        .active("template-preview")
        .width_mode(TabsWidthMode::Intrinsic)
        .with_template_modifier(|root, _| root.w_full())
        .spawn(cx);
    subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| cx.notify()));
    tabs
}
