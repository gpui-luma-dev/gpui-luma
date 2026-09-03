use std::sync::Arc;

use gpui::{AnyElement, AppContext, Context, Entity, Pixels, ScrollWheelEvent, SharedString, div, prelude::*, px};
use crate::controls::scroll_container::ScrollContainer;
use crate::controls::scrollbar::{Scrollbar, ScrollbarTemplate};

#[derive(Clone)]
pub struct PopupScrollSurface {
    container: ScrollContainer,
    viewport_height: Pixels,
    row_height: Pixels,
    content_top_padding: Pixels,
    item_count: usize,
}

impl PopupScrollSurface {
    pub fn new(
        id: impl Into<SharedString>,
        scrollbar_template: Arc<dyn ScrollbarTemplate>,
        cx: &mut impl AppContext,
    ) -> Self {
        Self {
            container: ScrollContainer::new(id, scrollbar_template, cx),
            viewport_height: px(220.0),
            row_height: px(28.0),
            content_top_padding: px(0.0),
            item_count: 0,
        }
    }

    pub fn scrollbar(&self) -> Entity<Scrollbar> {
        self.container.scrollbar()
    }

    pub fn set_vertical_offset<T: 'static>(&self, value: f32, cx: &mut Context<T>) {
        self.container.set_vertical_offset(value, cx);
    }

    pub fn configure(
        &mut self,
        item_count: usize,
        row_height: Pixels,
        content_top_padding: Pixels,
        viewport_height: Pixels,
    ) {
        self.item_count = item_count;
        self.row_height = row_height.max(px(1.0));
        self.content_top_padding = content_top_padding.max(px(0.0));
        self.viewport_height = viewport_height.max(px(1.0));
    }

    pub fn ensure_item_visible<T: 'static>(&self, item_index: usize, cx: &mut Context<T>) {
        if self.item_count == 0 {
            return;
        }

        let index = item_index.min(self.item_count.saturating_sub(1));
        let top = self.content_top_padding + (self.row_height * index as f32);
        let bottom = top + self.row_height;
        self.container.ensure_vertical_bounds_visible(top, bottom, cx);
    }

    pub fn sync<T: 'static>(&self, cx: &mut Context<T>) {
        self.container.sync_scrollbar(cx);
    }

    pub fn scroll_wheel<T: 'static>(&mut self, event: &ScrollWheelEvent, cx: &mut Context<T>) -> bool {
        let delta = event.delta.pixel_delta(self.row_height).y.as_f32();
        if !delta.is_finite() || delta.abs() <= f32::EPSILON {
            return false;
        }

        let current = self.container.vertical_offset().as_f32();
        let max = self.container.max_vertical_offset().as_f32();
        let target = (current + delta).clamp(0.0, max);
        if (target - current).abs() <= f32::EPSILON {
            return false;
        }

        self.container.set_vertical_offset(target, cx);
        true
    }

    pub fn render(&self, content: AnyElement) -> AnyElement {
        div().h(self.viewport_height).w_full().child(self.container.render(content)).into_any_element()
    }
}
