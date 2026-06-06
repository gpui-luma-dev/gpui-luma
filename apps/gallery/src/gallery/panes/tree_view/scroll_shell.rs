use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*};
use gpui_luma::controls::scroll_container::ScrollContainer;
use gpui_luma::controls::scrollbar::ScrollbarEvent;
use gpui_luma::controls::tree_view::TreeViewControl;
use gpui_luma_look_shadcn::ShadcnLook;

/// Gallery-only wrapper that clips the tree in a bounded viewport with a themed scrollbar.
pub(in crate::gallery) struct TreeViewScrollShell {
    scroll: ScrollContainer,
    tree: Entity<TreeViewControl<SharedString>>,
    _subscriptions: Vec<Subscription>,
}

impl TreeViewScrollShell {
    pub fn new(look: Arc<ShadcnLook>, tree: Entity<TreeViewControl<SharedString>>, cx: &mut Context<Self>) -> Self {
        let scroll = ScrollContainer::new("gallery-tree-scroll", look.scrollbar_template(), cx);
        let scrollbar = scroll.scrollbar();
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| match event {
            ScrollbarEvent::Change { value } => {
                this.scroll.set_vertical_offset(*value, cx);
            }
        })];

        Self { scroll, tree, _subscriptions: subscriptions }
    }

    pub fn tree(&self) -> Entity<TreeViewControl<SharedString>> {
        self.tree.clone()
    }
}

impl Render for TreeViewScrollShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scroll.sync_scrollbar(cx);

        self.scroll
            .render(div().id("gallery-tree-scroll-content").size_full().child(self.tree.clone()).into_any_element())
    }
}
